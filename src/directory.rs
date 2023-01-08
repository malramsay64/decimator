use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};

pub(crate) struct DirectoryData {
    directory: String,
}

impl From<DirectoryData> for String {
    fn from(d: DirectoryData) -> Self {
        d.directory
    }
}
impl FromRow<'_, SqliteRow> for DirectoryData {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let directory: &str = row.try_get("directory")?;
        Ok(Self {
            directory: directory.to_owned(),
        })
    }
}
