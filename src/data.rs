//! Provide the interface to the database
//
// This module provides the interface to the database. We want this interface
// to be properly handled and tested, so we split it into this file to maintain
// the understanding and separation.

use std::io::Cursor;

use anyhow::Error;
use camino::Utf8PathBuf;
use futures::stream::StreamExt;
use image::ImageFormat;
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
    // TODO: Improve the performance of this query. Not entirely sure where the bottlenecks are
    picture::Entity::find()
        .apply_if(update_all.then_some(()), |query, _| {
            query.filter(picture::Column::Thumbnail.is_null())
        })
        .stream(db)
        .await?
        .for_each_concurrent(num_cpus::get(), |r| async move {
            let picture = r.unwrap();
            let filepath = picture.filepath();
            let _span = tracing::info_span!("Updating thumbnail");
            tracing::info!("loading file from {}", &filepath);
            let mut buffer = Cursor::new(vec![]);
            let thumbnail = PictureData::load_thumbnail(&filepath, 240, 240)
                    .map(|f| f.write_to(&mut buffer, ImageFormat::Jpeg)).unwrap();
            if thumbnail.is_ok() {
                let mut picture: picture::ActiveModel = picture.into();
                picture.set(picture::Column::Thumbnail, buffer.into_inner().into());
                tracing::info!("{picture:?}");
                picture.update(db).await.unwrap();
            } else {
                tracing::info!("Unable to read file {}", &picture.filepath());
            }
        })
        .await;
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
