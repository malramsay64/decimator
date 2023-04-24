use std::fs::File;
use std::io::{BufRead, BufReader, Seek};

use anyhow::Error;
use camino::Utf8PathBuf;
use exif::{In, Tag};
use futures::io::Cursor;
use image::imageops::{flip_horizontal, flip_vertical, rotate180, rotate270, rotate90, FilterType};
use image::io::Reader;
use image::{DynamicImage, ImageBuffer, ImageFormat, RgbaImage};
use relm4::gtk::gdk::Texture;
use relm4::gtk::gdk_pixbuf::Pixbuf;
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};
use time::PrimitiveDateTime;
use uuid::Uuid;
use walkdir::DirEntry;

use crate::picture::{DateTime, Flag, Rating, Selection};

#[derive(Default, Clone)]
pub struct PictureData {
    pub id: Uuid,
    pub filepath: Utf8PathBuf,
    pub raw_extension: Option<String>,
    pub capture_time: Option<DateTime>,
    pub selection: Selection,
    pub rating: Rating,
    pub flag: Flag,
    pub hidden: Option<bool>,
    pub thumbnail: Option<DynamicImage>,
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

    #[tracing::instrument(name = "Updating exif data from file")]
    pub fn update_from_exif(&mut self) -> Result<(), Error> {
        // Get the image capture date
        let file = std::fs::File::open(&self.filepath)?;
        let mut bufreader = BufReader::new(&file);

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

    #[tracing::instrument(
        name = "Loading thumbnail from file using ImageReader",
        level = "trace"
    )]
    pub fn load_thumbnail(
        filepath: &Utf8PathBuf,
        scale_x: u32,
        scale_y: u32,
    ) -> Result<RgbaImage, Error> {
        let file = std::fs::File::open(filepath)?;
        let mut cursor = std::io::BufReader::new(file);
        let exif_data = exif::Reader::new().read_from_container(&mut cursor)?;

        // Reset the buffer to the start to read the image file
        cursor.rewind()?;
        let image = Reader::new(cursor).with_guessed_format()?.decode()?.resize(
            scale_x,
            scale_y,
            FilterType::Triangle,
        );
        // Apply Exif image transformations
        // https://sirv.com/help/articles/rotate-photos-to-be-upright/
        Ok(
            match exif_data
                .get_field(Tag::Orientation, In::PRIMARY)
                .and_then(|e| e.value.get_uint(0))
            {
                Some(1) => image.into_rgba8(),
                Some(2) => flip_horizontal(&image),
                Some(3) => rotate180(&image),
                Some(4) => flip_vertical(&image),
                Some(5) => rotate270(&flip_horizontal(&image)),
                Some(6) => rotate90(&image),
                Some(7) => rotate90(&flip_horizontal(&image)),
                Some(8) => rotate270(&image),
                // Where we can't interpret the exif data, we revert to the base image
                _ => image.into_rgba8(),
            },
        )
    }

    #[tracing::instrument(
        name = "Loading thumbnail from file using gdk::Pixbuf",
        level = "trace"
    )]
    pub fn load_thumbnail_gtk(filepath: Utf8PathBuf, scale_x: i32, scale_y: i32) -> Texture {
        let image = Pixbuf::from_file_at_scale(filepath, scale_x, scale_y, true)
            .expect("Image not found.")
            .apply_embedded_orientation()
            .expect("Unable to apply orientation.");
        Texture::for_pixbuf(&image)
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
impl From<DirEntry> for PictureData {
    fn from(path: DirEntry) -> Self {
        let p = Utf8PathBuf::from(path.into_path().to_str().expect("Invalid UTF-8 Path"));
        Self {
            id: Uuid::new_v4(),
            filepath: p,
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

        let thumbnail_data: Option<Vec<u8>> = row.try_get("thumbnail")?;
        let thumbnail: Option<DynamicImage> = thumbnail_data.map(|data| {
            image::load_from_memory_with_format(&data, ImageFormat::Jpeg)
                .expect("Unable to load image from database")
        });

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
            thumbnail,
        })
    }
}

impl std::fmt::Debug for PictureData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PictureData")
            .field("id", &self.id)
            .field("path", &self.filepath)
            .field("raw_extension", &self.raw_extension)
            .field("capture_time", &self.capture_time)
            .field("selection", &self.selection)
            .field("rating", &self.rating)
            .field("flag", &self.flag)
            .field("hidden", &self.hidden)
            .finish()
    }
}
