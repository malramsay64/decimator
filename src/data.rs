//! Provide the interface to the database
//
// This module provides the interface to the database. We want this interface
// to be properly handled and tested, so we split it into this file to maintain
// the understanding and separation.

use futures::future::join_all;
use futures::future::try_join_all;
use itertools::Itertools;
use rayon::prelude::*;
use std::io::Cursor;
use std::ops::Not;

use anyhow::Error;
use camino::Utf8PathBuf;
use image::ImageFormat;
use sea_orm::{sea_query, DatabaseConnection};
use sea_query::Condition;
use uuid::Uuid;

use sea_orm::query::*;
use sea_orm::*;

use crate::directory::DirectoryData;
use crate::picture::PictureData;
use ::entity::picture;
use ::entity::Selection;

#[tracing::instrument(name = "Querying Picture from directories", skip(db))]
pub(crate) async fn query_directory_pictures(
    db: &DatabaseConnection,
    directory: String,
) -> Result<Vec<PictureData>, Error> {
    dbg!(&directory);
    Ok(picture::Entity::find()
        .filter(picture::Column::Directory.eq(directory))
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

    let mut paginated_results = query.paginate(db, 8);
    while let Some(results) = paginated_results.fetch_and_next().await? {
        let futures: Vec<_> = results
            .par_iter()
            .map(|picture| {
                let filepath = picture.filepath().clone();
                let _span = tracing::info_span!("Updating thumbnail");
                let thumbnail_buffer: Result<Cursor<Vec<u8>>, Error> = {
                    let mut buffer = Cursor::new(vec![]);
                    tracing::info!("loading file from {}", &filepath);
                    PictureData::load_thumbnail(&filepath, 240, 240).map(|f| {
                        f.write_to(&mut buffer, ImageFormat::Jpeg)
                            .expect("Error writing image to buffer");
                        buffer
                    })
                };
                let mut picture: picture::ActiveModel = (*picture).clone().into();
                picture.set(
                    picture::Column::Thumbnail,
                    thumbnail_buffer.unwrap().into_inner().into(),
                );
                picture.update(db)
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
    for group in &images.into_iter().map(PictureData::to_active).chunks(1024) {
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
