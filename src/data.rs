//! Provide the interface to the database
//
// This module provides the interface to the database. We want this interface
// to be properly handled and tested, so we split it into this file to maintain
// the understanding and separation.

use anyhow::Error;
use camino::Utf8PathBuf;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::directory::DirectoryData;
use crate::picture::{PictureData, Selection};

pub(crate) async fn query_directory_pictures(
    db: &sqlx::SqlitePool,
    directory: String,
) -> Result<Vec<PictureData>, Error> {
    Ok(sqlx::query_as(
        r#"
            SELECT id, directory, filename, selection, rating, flag, hidden
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
) -> Result<Vec<Utf8PathBuf>, Error> {
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
    .await?
    .into_iter()
    .map(|(a, b): (String, String)| {
        let mut path = Utf8PathBuf::from(&a);
        path.push(b);
        path
    })
    .collect::<Vec<Utf8PathBuf>>())
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

pub(crate) async fn update_selection_state(
    db: &SqlitePool,
    id: Uuid,
    state: Selection,
) -> Result<(), Error> {
    let state = state.to_string();
    sqlx::query!(
        r#"
            UPDATE picture
            SET selection = $1
            WHERE id = $2
        "#,
        state,
        id
    )
    .execute(db)
    .await?;
    Ok(())
}
