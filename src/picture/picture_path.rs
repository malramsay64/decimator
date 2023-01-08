use camino::Utf8PathBuf;
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PicturePath {
    pub directory: String,
    pub filename: String,
}

impl From<Utf8PathBuf> for PicturePath {
    fn from(path: Utf8PathBuf) -> Self {
        let directory = path.parent().expect("Invalid parent").as_str().to_owned();
        let filename = path.file_name().expect("No valid filename.").to_owned();

        Self {
            directory,
            filename,
        }
    }
}

impl FromRow<'_, SqliteRow> for PicturePath {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let directory: String = row.try_get("directory")?;
        let filename: String = row.try_get("filename")?;
        Ok(Self {
            directory,
            filename,
        })
    }
}
