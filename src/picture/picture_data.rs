use std::io::{BufReader, Cursor, Seek};

use anyhow::Error;
use camino::Utf8PathBuf;
use exif::{In, Tag};
use image::imageops::{flip_horizontal, flip_vertical, rotate180, rotate270, rotate90, FilterType};
use image::io::Reader;
use image::{ImageFormat, RgbaImage};
use sea_orm::ActiveValue;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::PrimitiveDateTime;
use uuid::Uuid;
use walkdir::DirEntry;

use entity::picture;
use entity::{Flag, Rating, Selection};

const DISPLAY_FORMAT: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

#[derive(Default, Clone, PartialEq)]
pub struct PictureData {
    pub id: Uuid,
    pub filepath: Utf8PathBuf,
    pub raw_extension: Option<String>,
    pub capture_time: Option<PrimitiveDateTime>,
    pub selection: Selection,
    pub rating: Option<Rating>,
    pub flag: Option<Flag>,
    pub hidden: bool,
    pub thumbnail: Option<RgbaImage>,
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
            Some(PrimitiveDateTime::parse(&v, DISPLAY_FORMAT).expect("Unable to parse datetime"))
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
}

impl From<picture::Model> for PictureData {
    fn from(value: picture::Model) -> Self {
        let thumbnail = value.thumbnail.as_ref().and_then(|data| {
            image::load_from_memory_with_format(data, image::ImageFormat::Jpeg)
                .ok()
                .map(|i| i.into_rgba8())
        });
        Self {
            id: value.id,
            filepath: value.filepath(),
            raw_extension: value.raw_extension,
            capture_time: value.capture_time.map(PrimitiveDateTime::from),
            selection: value.selection,
            rating: value.rating,
            flag: value.flag,
            hidden: value.hidden,
            thumbnail,
        }
    }
}

impl PictureData {
    pub fn to_active(self) -> picture::ActiveModel {
        let mut thumbnail = Cursor::new(vec![]);
        if let Some(f) = self.thumbnail.as_ref() {
            f.write_to(&mut thumbnail, ImageFormat::Jpeg).unwrap();
        }
        picture::ActiveModel {
            id: ActiveValue::Unchanged(self.id),
            short_hash: ActiveValue::not_set(),
            full_hash: ActiveValue::not_set(),
            directory: ActiveValue::Set(self.directory()),
            filename: ActiveValue::Set(self.filename()),
            raw_extension: ActiveValue::Set(self.raw_extension),
            capture_time: ActiveValue::Set(self.capture_time),
            selection: ActiveValue::Set(self.selection),
            rating: ActiveValue::Set(self.rating),
            flag: ActiveValue::Set(self.flag),
            hidden: ActiveValue::Set(self.hidden),
            thumbnail: ActiveValue::Set(Some(thumbnail.into_inner())),
        }
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
