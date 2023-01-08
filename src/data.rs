//! Provide the interface to the database
//
// This module provides the interface to the database. We want this interface
// to be properly handled and tested, so we split it into this file to maintain
// the understanding and separation.

use anyhow::Error;
use sqlx::SqlitePool;

use crate::directory::DirectoryData;
use crate::picture::{PictureData, PicturePath};

pub(crate) async fn query_directory_pictures(
    db: &sqlx::SqlitePool,
    directory: String,
) -> Result<Vec<PictureData>, Error> {
    Ok(sqlx::query_as(
        r#"
            SELECT id, directory, filename, picked, rating, flag, hidden
            FROM picture
            WHERE directory == $1
            ORDER BY capture_time DESC, filename DESC
        "#,
    )
    .bind(directory)
    .fetch_all(db)
    .await?)
}

pub(crate) async fn query_existing_pictures(
    db: &SqlitePool,
    directory: String,
) -> Result<Vec<PicturePath>, Error> {
    Ok(sqlx::query_as(
        r#"
            SELECT directory, filename
            FROM picture
            WHERE directory == ? OR directory like ?
        "#,
    )
    // This matches the current directory. There is no slash after
    // the directory so we just use the exact value.
    .bind(&directory)
    // This matches all the subdirectories, which are needed since we
    // perform a recursive search when adding new directories.
    .bind(format!("{directory}/%"))
    .fetch_all(db)
    .await?)
}
pub(crate) async fn query_unique_directories(db: &SqlitePool) -> Result<Vec<String>, Error> {
    Ok(sqlx::query_as(
        r#"
            SELECT DISTINCT directory
            FROM picture
            ORDER BY directory
        "#,
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(DirectoryData::into)
    .collect())
}
