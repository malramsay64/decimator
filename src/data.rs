//! Provide the interface to the database
//
// This module provides the interface to the database. We want this interface
// to be properly handled and tested, so we split it into this file to maintain
// the understanding and separation.

use std::future;
use std::io::Cursor;
use std::ops::Not;
use std::path::PathBuf;
use std::sync::Arc;

use ::entity::{picture, Selection};
use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use camino::Utf8PathBuf;
use entity::directory;
use futures::future::{join_all, try_join_all};
use futures::{StreamExt, TryStreamExt};
use futures_concurrency::prelude::*;
use iced::task::sipper;
use iced::task::Straw;
use image::ImageFormat;
use itertools::Itertools;
use rayon::prelude::*;
use sea_orm::entity::*;
use sea_orm::prelude::*;
use sea_orm::query::*;
use uuid::Uuid;

use crate::directory::DirectoryData;
use crate::picture::PictureThumbnail;
use crate::picture::{PictureData, ThumbnailData};
use crate::DirectoryDataDB;

/// Search for pictures in the database located within a directory
///
/// The directory searched is specified from the input. The directory is the
/// full directory to the file, subdirectories are not included within the
/// search.
#[tracing::instrument(name = "Querying Picture from directories", skip(db))]
pub(crate) async fn query_directory_pictures(
    db: &DatabaseConnection,
    directory: DirectoryDataDB,
) -> Result<Vec<PictureThumbnail>, Error> {
    let mut ids: Vec<Uuid> = vec![directory.id];
    ids.extend(directory.children.iter());
    Ok(picture::Entity::find()
        .filter(picture::Column::DirectoryId.is_in(ids))
        .all(db)
        .await?
        .into_iter()
        .map(PictureData::from)
        .map(|d| PictureThumbnail {
            data: d,
            handle: None,
        })
        .collect())
}

#[tracing::instrument(
    name = "Querying Picture within directories or subdirectories.",
    skip(db)
)]
pub(crate) async fn query_existing_pictures(
    db: &DatabaseConnection,
    directory: &Utf8PathBuf,
) -> Result<Vec<Utf8PathBuf>, Error> {
    Ok(picture::Entity::find()
        .filter(
            Condition::all()
                // This matches the current directory. There is no slash after
                // the directory so we just use the exact value.
                .add(picture::Column::Directory.eq(format!("{directory}")))
                // This matches all the subdirectories, which are needed since we
                // perform a recursive search when adding new directories.
                .add(picture::Column::Directory.like(format!("{directory}/%"))),
        )
        .all(db)
        .await?
        .iter()
        .map(picture::Model::filepath)
        .collect::<Vec<_>>())
}

fn load_thumbnail_buffer(filepath: &Utf8PathBuf, size: u32) -> Result<Vec<u8>, Error> {
    let _span = tracing::info_span!("Updating thumbnail");
    let thumbnail_buffer: Result<Cursor<Vec<u8>>, Error> = {
        let mut buffer = Cursor::new(vec![]);
        tracing::debug!("loading file from {}", filepath);
        PictureData::load_thumbnail(filepath, size, size).map(|f| {
            f.write_to(&mut buffer, ImageFormat::Jpeg)
                .expect("Error writing image to buffer");
            buffer
        })
    };
    thumbnail_buffer.map(|i| i.into_inner())
}

#[derive(Debug, Clone)]
pub struct Progress {
    pub percent: f32,
}

pub(crate) fn update_thumbnails(
    db: &DatabaseConnection,
    update_all: bool,
) -> impl Straw<(), Progress, ThumbnailError> {
    let db = db.clone();
    sipper(async move |mut progress| {
        let query = picture::Entity::find().apply_if(update_all.not().then_some(()), |q, _| {
            q.filter(picture::Column::Thumbnail.is_null())
        });
        let num_items = query
            .clone()
            .count(&db)
            .await
            .map_err(|e| Into::<ThumbnailError>::into(Error::from(e)))?;
        tracing::info!("{} {:?}", update_all, num_items);
        let mut paginated_results = query
            .stream(&db)
            .await
            .map_err(|e| Into::<ThumbnailError>::into(Error::from(e)))?;

        let mut index = 0;
        while let Some(picture) = paginated_results.next().await {
            // tracing::debug!("Loading Picture {picture:?}");
            let picture = picture.map_err(|e| Into::<ThumbnailError>::into(Error::from(e)))?;
            let filepath = picture.filepath().clone();
            let buffer_result =
                tokio::task::spawn_blocking(move || load_thumbnail_buffer(&filepath, 480))
                    .await
                    .inspect_err(|e| tracing::error!("{e:?}"))
                    .map_err(|e| Into::<ThumbnailError>::into(Error::from(e)))?;
            if let Ok(buffer) = buffer_result {
                let mut picture: picture::ActiveModel = picture.into();
                picture.set(picture::Column::Thumbnail, buffer.into());
                // Updates have to be done to single objects, unlike inserts
                // where we can insert many items at once
                picture
                    .update(&db)
                    .await
                    .map_err(|e| Into::<ThumbnailError>::into(Error::from(e)))?;
                tracing::debug!("Successfully loaded picture into buffer");
            } else {
                tracing::error!("{buffer_result:?}");
                continue;
            };
            index += 1;
            // .inspect_err(|e| tracing::error!("{e:?}"))
            // .map_err(|e| Into::<ThumbnailError>::into(Error::from(e)))?;
            // let buffer = tokio::task::spawn_blocking(move || load_thumbnail_buffer(&filepath, 480))
            //     .await
            //     .inspect_err(|e| tracing::error!("{e:?}"))
            //     .map_err(|e| Into::<ThumbnailError>::into(Error::from(e)))?
            //     .inspect_err(|e| tracing::error!("{e:?}"))
            //     .map_err(|e| Into::<ThumbnailError>::into(Error::from(e)))?;

            let _ = progress
                .send(Progress {
                    percent: 100.0 * index as f32 / num_items as f32,
                })
                .await;
        }
        Ok(())
    })
}

pub(crate) async fn add_new_images(
    db: &DatabaseConnection,
    images: Vec<PictureData>,
) -> Result<(), Error> {
    let mut futures = vec![];
    for group in &images
        .into_iter()
        .map(PictureData::into_active)
        .chunks(1024)
    {
        futures.push(picture::Entity::insert_many(group).exec(db))
    }
    join_all(futures).await;
    Ok(())
}

pub(crate) async fn query_directories(
    db: &DatabaseConnection,
) -> Result<Vec<DirectoryDataDB>, Error> {
    tracing::debug!("Loading directories");
    let directories_with_children: Vec<(directory::Model, Vec<directory::Model>)> =
        directory::Entity::find()
            .order_by_desc(directory::Column::Directory)
            .find_with_linked(directory::SelfReferencingLink)
            .all(db)
            .await?;
    tracing::debug!("Loaded {} directories", directories_with_children.len());
    Ok(directories_with_children
        .into_iter()
        .map(|(m, c)| DirectoryDataDB::new(m, c))
        .collect())
}

pub(crate) async fn query_unique_directories(
    db: &DatabaseConnection,
) -> Result<Vec<DirectoryData>, Error> {
    Ok(picture::Entity::find()
        .select_only()
        .column(picture::Column::Directory)
        .distinct()
        .into_tuple::<String>()
        .all(db)
        .await?
        .into_iter()
        .map(DirectoryData::from)
        .collect())
}

pub(crate) async fn update_picture_data(
    db: &DatabaseConnection,
    data: PictureData,
) -> Result<(), Error> {
    data.into_active().update(db).await?;
    Ok(())
}

/// Modify the state of the selected attribute on an image
///
/// Provides an interface to the database ensuring the state of the selected
/// image is appropriately updated.
pub(crate) async fn update_selection_state(
    db: &DatabaseConnection,
    id: Uuid,
    selection: Selection,
) -> Result<(), Error> {
    picture::ActiveModel {
        id: ActiveValue::Unchanged(id),
        selection: ActiveValue::Set(selection),
        ..Default::default()
    }
    .update(db)
    .await?;
    Ok(())
}

pub async fn load_thumbnail(db: &DatabaseConnection, id: Uuid) -> Result<ThumbnailData, Error> {
    picture::Entity::find_by_id(id)
        .one(db)
        .await?
        .map(ThumbnailData::from)
        .ok_or(anyhow!("ID does not exist within database."))
}

#[derive(Debug, Clone)]
pub enum ThumbnailError {
    ThumbnailFailed(Arc<anyhow::Error>),
}

impl From<anyhow::Error> for ThumbnailError {
    fn from(error: anyhow::Error) -> Self {
        ThumbnailError::ThumbnailFailed(Arc::new(error))
    }
}
