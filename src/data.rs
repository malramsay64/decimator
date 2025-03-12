//! Provide the interface to the database
//
// This module provides the interface to the database. We want this interface
// to be properly handled and tested, so we split it into this file to maintain
// the understanding and separation.

use std::io::Cursor;
use std::ops::Not;

use ::entity::{Selection, picture};
use anyhow::Error;
use anyhow::anyhow;
use camino::Utf8PathBuf;
use futures::future::{join_all, try_join_all};
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

/// Search for pictures in the database located within a directory
///
/// The directory searched is specified from the input. The directory is the
/// full directory to the file, subdirectories are not included within the
/// search.
#[tracing::instrument(name = "Querying Picture from directories", skip(db))]
pub(crate) async fn query_directory_pictures(
    db: &DatabaseConnection,
    directory: String,
) -> Result<Vec<PictureThumbnail>, Error> {
    Ok(picture::Entity::find()
        .filter(picture::Column::Directory.eq(directory))
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

pub(crate) async fn update_thumbnails(
    db: &DatabaseConnection,
    update_all: bool,
) -> Result<(), Error> {
    let query = picture::Entity::find().apply_if(update_all.not().then_some(()), |q, _| {
        q.filter(picture::Column::Thumbnail.is_null())
    });
    let num_items = query.clone().count(db).await?;

    tracing::info!("{} {:?}", update_all, num_items);

    let mut paginated_results = query.paginate(db, 8);
    while let Some(results) = paginated_results.fetch_and_next().await? {
        let futures: Vec<_> = results
            .par_iter()
            .filter_map(|picture| {
                let filepath = picture.filepath().clone();
                let _span = tracing::info_span!("Updating thumbnail");
                let thumbnail_buffer: Result<Cursor<Vec<u8>>, Error> = {
                    let mut buffer = Cursor::new(vec![]);
                    tracing::debug!("loading file from {}", &filepath);
                    PictureData::load_thumbnail(&filepath, 240, 240).map(|f| {
                        f.write_to(&mut buffer, ImageFormat::Jpeg)
                            .expect("Error writing image to buffer");
                        buffer
                    })
                };
                match thumbnail_buffer {
                    Ok(buffer) => {
                        let mut picture: picture::ActiveModel = (*picture).clone().into();
                        picture.set(picture::Column::Thumbnail, buffer.into_inner().into());
                        // Updates have to be done to single objects, unlike inserts
                        // where we can insert many items at once
                        Some(picture.update(db))
                    }
                    _ => {
                        tracing::warn!(
                            "Error loading file from {filepath:?}, {thumbnail_buffer:?}"
                        );
                        None
                    }
                }
            })
            .collect();
        try_join_all(futures).await.unwrap();
    }
    Ok(())
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
