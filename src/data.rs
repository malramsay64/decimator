//! Provide the interface to the database
//
// This module provides the interface to the database. We want this interface
// to be properly handled and tested, so we split it into this file to maintain
// the understanding and separation.

use std::io::Cursor;
use std::ops::Not;

use anyhow::Error;
use camino::Utf8PathBuf;
use futures::{Stream, TryStreamExt};
use image::ImageFormat;
use relm4::gtk;
use sea_orm::{sea_query, DatabaseConnection};
use sea_query::{Condition, Expr};
use uuid::Uuid;

pub mod picture;

use sea_orm::query::*;
use sea_orm::*;

use crate::directory::DirectoryData;
use crate::picture::{PictureData, Selection};

#[tracing::instrument(name = "Querying Picture from directories", skip(db))]
pub(crate) async fn query_directory_pictures(
    db: &DatabaseConnection,
    directories: &[String],
) -> Result<Vec<PictureData>, Error> {
    dbg!(&directories);
    Ok(picture::Entity::find()
        .filter(Expr::col(picture::Column::Directory).is_in(directories))
        .all(db)
        .await?
        .into_iter()
        .map(PictureData::from)
        .collect())
}

pub(crate) async fn query_existing_pictures(
    db: &DatabaseConnection,
    directory: &Utf8PathBuf,
) -> Result<Vec<Utf8PathBuf>, Error> {
    Ok(picture::Entity::find()
        .filter(
            Condition::all()
                // This matches the current directory. There is no slash after
                // the directory so we just use the exact value.
                .add(picture::Column::Directory.eq(&format!("{directory}")))
                // This matches all the subdirectories, which are needed since we
                // perform a recursive search when adding new directories.
                .add(picture::Column::Directory.like(&format!("{directory}/%"))),
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

    // TODO: Improve the performance of this query. Not entirely sure where the bottlenecks are
    let stream = query.stream(db).await?;

    stream
        .try_for_each_concurrent(num_cpus::get(), |picture| async move {
            let filepath = picture.filepath().clone();
            let _span = tracing::info_span!("Updating thumbnail");
            let thumbnail_buffer: Result<Cursor<Vec<u8>>, Error> =
                relm4::spawn_blocking(move || {
                    let mut buffer = Cursor::new(vec![]);
                    tracing::info!("loading file from {}", &filepath);
                    PictureData::load_thumbnail(&filepath, 240, 240).map(|f| {
                        f.write_to(&mut buffer, ImageFormat::Jpeg)
                            .expect("Error writing image to buffer");
                        buffer
                    })
                })
                .await
                .expect("Unable to join");
            if let Ok(thumbnail) = thumbnail_buffer {
                let mut picture: picture::ActiveModel = picture.into();
                picture.set(picture::Column::Thumbnail, thumbnail.into_inner().into());
                tracing::trace!("{picture:?}");
                picture.update(db).await?;
            } else {
                tracing::warn!("Unable to read file {}", &picture.filepath());
            }
            Ok(())
        })
        .await
        .expect("Error updating thumbnails");
    Ok(())
}

pub(crate) async fn add_new_images(
    db: &DatabaseConnection,
    images: Vec<PictureData>,
) -> Result<(), Error> {
    picture::Entity::insert_many(images.into_iter().map(picture::ActiveModel::from))
        .exec(db)
        .await?;
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
