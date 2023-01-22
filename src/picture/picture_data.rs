use adw::subclass::prelude::*;
use anyhow::Error;
use camino::Utf8PathBuf;
use gdk::Texture;

use gtk::gdk_pixbuf::Pixbuf;
use gtk::{gdk};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::picture::{DateTime, Flag, Rating, Selection};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PictureData {
    pub id: Uuid,
    pub filepath: Utf8PathBuf,
    pub raw_extension: Option<String>,
    pub capture_time: Option<DateTime>,
    pub selection: Selection,
    pub rating: Rating,
    pub flag: Flag,
    pub hidden: Option<bool>,
    #[serde(skip)]
    pub thumbnail: Option<Texture>,
}

impl PictureData {
    pub fn path(&self) -> String {
        self.filepath.clone().into()
    }

    pub fn directory(&self) -> String {
        self.filepath
            .parent()
            .expect("Invalid parent directory")
            .as_str()
            .to_owned()
    }

    pub fn filename(&self) -> String {
        self.filepath
            .file_name()
            .expect("No valid filename.")
            .to_owned()
    }

    pub fn update_from_exif(&mut self) -> Result<(), Error> {
        // Get the image capture date
        let file = std::fs::File::open(&self.filepath)?;
        let mut bufreader = std::io::BufReader::new(&file);

        let exifreader = exif::Reader::new();
        let exif = exifreader.read_from_container(&mut bufreader)?;

        let capture_datetime = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY);

        self.capture_time = if let Some(f) = capture_datetime {
            let v = f.value.display_as(exif::Tag::DateTimeOriginal).to_string();
            Some(v.try_into().expect("Unable to parse datetime"))
        } else {
            None
        };

        Ok(())
    }
}

impl From<Utf8PathBuf> for PictureData {
    fn from(path: Utf8PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            filepath: path,
            ..Default::default()
        }
    }
}

impl FromRow<'_, SqliteRow> for PictureData {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let directory: &str = row.try_get("directory")?;
        let filename: &str = row.try_get("filename")?;
        let filepath: Utf8PathBuf = [directory, filename].iter().collect();
        let capture_time: Option<DateTime> = row
            // We need to ensure we get the Option.
            .try_get::<Option<PrimitiveDateTime>, _>("capture_time")?
            .map(|t| t.into());

        Ok(Self {
            id: row.try_get("id")?,
            filepath,
            raw_extension: None,
            capture_time,
            selection: row
                .try_get::<&str, _>("selection")?
                .try_into()
                .unwrap_or_default(),
            rating: row
                .try_get::<&str, _>("rating")?
                .try_into()
                .unwrap_or_default(),
            flag: row
                .try_get::<&str, _>("flag")?
                .try_into()
                .unwrap_or_default(),
            hidden: row.try_get("hidden")?,
            thumbnail: None,
        })
    }
}

impl std::fmt::Debug for PictureData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PictureData")
            .field("id", &self.id)
            .field("path", &self.filepath)
            .field("thumbnail", &self.thumbnail.is_some())
            .finish()
    }
}

impl PictureData {
    #[tracing::instrument(
        name = "Loading thumbnail from file using ImageReader",
        level = "trace"
    )]
    pub fn thumbnail(path: &str, (scale_x, scale_y): (i32, i32)) -> Texture {
        let image = Pixbuf::from_file_at_scale(path, scale_x, scale_y, true)
            .expect("Image not found.")
            .apply_embedded_orientation()
            .expect("Unable to apply orientation.");
        Texture::for_pixbuf(&image)
    }
}
