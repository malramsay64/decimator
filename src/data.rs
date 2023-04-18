//! Provide the interface to the database
//
// This module provides the interface to the database. We want this interface
// to be properly handled and tested, so we split it into this file to maintain
// the understanding and separation.

use anyhow::Error;
use camino::Utf8PathBuf;
use futures::stream::StreamExt;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use tracing::Instrument;
use uuid::Uuid;

use crate::directory::DirectoryData;
use crate::picture::{PictureData, Selection};

pub(crate) async fn query_directory_pictures(
    db: &sqlx::SqlitePool,
    directory: String,
) -> Result<Vec<PictureData>, Error> {
    Ok(sqlx::query_as(
        r#"
            SELECT 
                id, 
                directory, 
                filename, 
                capture_time, 
                selection, 
                rating, 
                flag, 
                hidden, 
                thumbnail
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
    .map(|(a, b): (String, String)| [a, b].iter().collect::<Utf8PathBuf>())
    .collect::<Vec<_>>())
}

pub(crate) async fn update_thumbnails(db: &SqlitePool, update_all: bool) -> Result<(), Error> {
    // TODO: Improve the performance of this query. Not entirely sure where the bottlenecks are
    let mut query = QueryBuilder::new(
        "
            SELECT 
                id, 
                directory, 
                filename, 
                capture_time, 
                selection, 
                rating, 
                flag, 
                hidden, 
                thumbnail
            FROM
                picture
        ",
    );
    if !update_all {
        query.push("WHERE thumbnail is NULL");
    }
    query
        .build_query_as::<'_, PictureData>()
        .fetch(db)
        .map(|r| {
            let db = db.clone();
            relm4::spawn(async move {
                let picture = r.unwrap();
                let span = tracing::debug_span!("Updating thumbnail");
                let filepath = picture.filepath.clone();
                async move {
                    tracing::debug!("loading file from {}", &filepath);
                    let thumbnail = PictureData::load_thumbnail(filepath, 240, 240).await;
                    if let Ok(t) = thumbnail {
                        // TODO: Potential optimisations using a join in the SQL query for the update
                        sqlx::query(
                            r#"
                    UPDATE picture
                    SET             
                        thumbnail = ?
                    WHERE
                        id = ?
                "#,
                        )
                        .bind(&t.into_bytes())
                        .bind(picture.id)
                        .execute(&db)
                        .await
                        .expect("Query failed to execute");
                    }
                }
                .instrument(span)
                .await
            })
        })
        .buffer_unordered(num_cpus::get())
        .collect::<Vec<_>>()
        .await;
    Ok(())
}

pub(crate) async fn add_new_images(db: &SqlitePool, images: Vec<PictureData>) -> Result<(), Error> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        "
        INSERT INTO picture(
            id, 
            directory, 
            filename, 
            raw_extension, 
            capture_time, 
            rating, 
            flag, 
            hidden, 
            selection, 
            thumbnail
        )",
    );

    query_builder.push_values(images, |mut b, picture| {
        b.push_bind(Uuid::new_v4())
            .push_bind(picture.directory())
            .push_bind(picture.filename())
            .push_bind(picture.raw_extension)
            .push_bind(picture.capture_time.map(|d| d.datetime()))
            .push_bind(picture.rating.to_string())
            .push_bind(picture.flag.to_string())
            .push_bind(picture.hidden)
            .push_bind(picture.selection.to_string())
            .push_bind(picture.thumbnail.map(|p| p.into_bytes()));
    });

    // Run the database query to update all the files
    let query = query_builder.build();
    query.execute(db).await?;

    Ok(())
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
