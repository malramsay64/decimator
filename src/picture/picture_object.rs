use std::fmt::Display;
use std::str::FromStr;

use camino::{Utf8Path, Utf8PathBuf};
use gtk::glib::value::{
    FromValue, GenericValueTypeOrNoneChecker, ToValueOptional, ValueType, ValueTypeOptional,
};
use gtk::glib::Value;
use serde::{Deserialize, Serialize};

use adw::prelude::*;
use adw::subclass::prelude::*;
use anyhow::{anyhow, Error, Result};
use gdk::Texture;
use glib::Object;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::{gdk, glib};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};
use uuid::Uuid;

glib::wrapper! {
    pub struct PictureObject(ObjectSubclass<imp::PictureObject>);
}

impl PictureObject {
    pub fn get_filepath(&self) -> Utf8PathBuf {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .filepath
            .clone()
    }
    pub fn get_id(&self) -> Uuid {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .id
    }
    fn get_picked(&self) -> Option<bool> {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .picked
    }
    fn get_rating(&self) -> Option<Rating> {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .rating
    }
    fn get_flag(&self) -> Option<Flag> {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .flag
    }
    fn get_hidden(&self) -> Option<bool> {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .hidden
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Rating {
    One,
    Two,
    Three,
    Four,
    Five,
}

impl FromStr for Rating {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "one" | "One" => Ok(Rating::One),
            "two" | "Two" => Ok(Rating::Two),
            "three" | "Three" => Ok(Rating::Three),
            "four" | "Four" => Ok(Rating::Four),
            "five" | "Five" => Ok(Rating::Five),
            _ => Err(anyhow!("Invalid value for rating.")),
        }
    }
}

impl Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Rating::One => "One",
            Rating::Two => "Two",
            Rating::Three => "Three",
            Rating::Four => "Four",
            Rating::Five => "Five",
        };
        write!(f, "{}", text)
    }
}

impl ToValue for Rating {
    fn to_value(&self) -> glib::Value {
        <str>::to_value(&self.to_string())
    }

    fn value_type(&self) -> glib::Type {
        String::static_type()
    }
}

impl ValueType for Rating {
    type Type = String;
}
unsafe impl<'a> FromValue<'a> for Rating {
    type Checker = GenericValueTypeOrNoneChecker<Self>;
    unsafe fn from_value(value: &'a Value) -> Self {
        Rating::from_str(<&str>::from_value(value)).unwrap()
    }
}
impl ValueTypeOptional for Rating {}
impl StaticType for Rating {
    fn static_type() -> glib::Type {
        String::static_type()
    }
}
impl ToValueOptional for Rating {
    fn to_value_optional(s: Option<&Self>) -> glib::Value {
        let value = s.map(Rating::to_string);
        <String>::to_value_optional(value.as_ref())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Flag {
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
}

impl FromStr for Flag {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "red" | "Red" => Ok(Flag::Red),
            "green" | "Green" => Ok(Flag::Green),
            "blue" | "Blue" => Ok(Flag::Blue),
            "yellow" | "Yellow" => Ok(Flag::Yellow),
            "purple" | "Purple" => Ok(Flag::Purple),
            _ => Err(anyhow!("Invalid value for Flags.")),
        }
    }
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Flag::Red => "Red",
            Flag::Green => "Green",
            Flag::Blue => "Blue",
            Flag::Yellow => "Yellow",
            Flag::Purple => "Purple",
        };
        write!(f, "{}", text)
    }
}

impl ToValue for Flag {
    fn to_value(&self) -> glib::Value {
        <str>::to_value(&self.to_string())
    }

    fn value_type(&self) -> glib::Type {
        String::static_type()
    }
}

impl ValueType for Flag {
    type Type = String;
}
unsafe impl<'a> FromValue<'a> for Flag {
    type Checker = GenericValueTypeOrNoneChecker<Self>;
    unsafe fn from_value(value: &'a Value) -> Self {
        Flag::from_str(<&str>::from_value(value)).unwrap()
    }
}
impl StaticType for Flag {
    fn static_type() -> glib::Type {
        String::static_type()
    }
}
impl ValueTypeOptional for Flag {}
impl ToValueOptional for Flag {
    fn to_value_optional(s: Option<&Self>) -> glib::Value {
        let value = s.map(Flag::to_string);
        <String>::to_value_optional(value.as_ref())
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PictureData {
    pub id: Uuid,
    pub filepath: Utf8PathBuf,
    pub picked: Option<bool>,
    pub rating: Option<Rating>,
    pub flag: Option<Flag>,
    pub hidden: Option<bool>,
    #[serde(skip)]
    pub thumbnail: Option<Texture>,
    #[serde(skip)]
    pub preview: Option<Texture>,
}

impl PictureData {
    fn path(&self) -> String {
        self.filepath.clone().into()
    }
}

impl From<PictureData> for PictureObject {
    fn from(pic: PictureData) -> Self {
        Object::builder()
            .property("id", pic.id.to_string())
            .property("path", pic.path())
            .property::<Option<Texture>>("thumbnail", None)
            .build()
    }
}

impl<T: AsRef<PictureObject>> From<T> for PictureData {
    fn from(p: T) -> Self {
        let p = p.as_ref();
        Self {
            id: p.get_id(),
            filepath: p.get_filepath(),
            picked: p.get_picked(),
            rating: p.get_rating(),
            flag: p.get_flag(),
            hidden: p.get_hidden(),
            thumbnail: None,
            preview: None,
        }
    }
}

impl FromRow<'_, SqliteRow> for PictureData {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let id = row.try_get("id")?;
        let directory: &str = row.try_get("directory")?;
        let filename: &str = row.try_get("filename")?;
        let filepath = Utf8Path::new(directory).join(filename);
        let picked: Option<bool> = row.try_get("picked")?;
        let rating_string: Option<String> = row.try_get("rating")?;
        let rating: Option<Rating> = rating_string.map(|s| Rating::from_str(&s).unwrap());
        let flag_string: Option<String> = row.try_get("flag")?;
        let flag: Option<Flag> = flag_string.map(|s| Flag::from_str(&s).unwrap());
        let hidden: Option<bool> = row.try_get("hidden")?;
        Ok(Self {
            id,
            filepath,
            picked,
            rating,
            flag,
            hidden,
            thumbnail: None,
            preview: None,
        })
    }
}

impl std::fmt::Debug for PictureData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PictureData")
            .field("id", &self.id)
            .field("path", &self.filepath)
            .field("thumbnail", &self.thumbnail.is_some())
            .field("preview", &self.preview.is_some())
            .finish()
    }
}

impl PictureData {
    #[tracing::instrument(
        name = "Loading thumbnail from file using ImageReader",
        level = "trace"
    )]
    pub fn get_thumbnail(path: &str) -> Texture {
        let image = Pixbuf::from_file_at_scale(path, 320, 320, true)
            .expect("Image not found.")
            .apply_embedded_orientation()
            .expect("Unable to apply orientation.");
        Texture::for_pixbuf(&image)
    }

    #[tracing::instrument(name = "Loading preview from file using ImageReader", level = "trace")]
    pub fn get_preview(path: &str) -> Texture {
        let image = Pixbuf::from_file(path)
            .expect("Image not found.")
            .apply_embedded_orientation()
            .expect("Unable to apply orientation.");
        Texture::for_pixbuf(&image)
    }
}

mod imp {
    use std::sync::{Arc, Mutex};

    use camino::Utf8PathBuf;
    use gdk::Texture;
    use glib::ParamSpecObject;
    use glib::{ParamSpec, ParamSpecString, Value};

    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{gdk, glib};
    use once_cell::sync::Lazy;
    use uuid::Uuid;

    use super::{Flag, PictureData, Rating};

    #[derive(Default)]
    pub struct PictureObject {
        pub data: Arc<Mutex<PictureData>>,
    }

    impl PictureObject {
        fn get_filepath(&self) -> Utf8PathBuf {
            self.data
                .as_ref()
                .lock()
                .expect("Mutex lock is poisoned")
                .filepath
                .clone()
        }
        fn get_id(&self) -> Uuid {
            self.data
                .as_ref()
                .lock()
                .expect("Mutex lock is poisoned")
                .id
        }
        fn get_picked(&self) -> Option<bool> {
            self.data
                .as_ref()
                .lock()
                .expect("Mutex lock is poisoned")
                .picked
        }
        fn get_rating(&self) -> Option<Rating> {
            self.data
                .as_ref()
                .lock()
                .expect("Mutex lock is poisoned")
                .rating
        }
        fn get_flag(&self) -> Option<Flag> {
            self.data
                .as_ref()
                .lock()
                .expect("Mutex lock is poisoned")
                .flag
        }
        fn get_hidden(&self) -> Option<bool> {
            self.data
                .as_ref()
                .lock()
                .expect("Mutex lock is poisoned")
                .hidden
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PictureObject {
        const NAME: &'static str = "PictureObject";
        type Type = super::PictureObject;
    }

    // Trait shared by all GObjects
    impl ObjectImpl for PictureObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("id").build(),
                    ParamSpecString::builder("path").build(),
                    ParamSpecString::builder("picked").build(),
                    ParamSpecString::builder("rating").build(),
                    ParamSpecString::builder("flag").build(),
                    ParamSpecString::builder("hidden").build(),
                    ParamSpecObject::builder::<Option<Texture>>("thumbnail").build(),
                    ParamSpecObject::builder::<Option<Texture>>("preview").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "path" => {
                    let input_value: String = value
                        .get()
                        .expect("The value needs to be of type `String`.");
                    let mut data = self.data.lock().expect("Mutex is Poisoned.");
                    data.filepath = input_value.into();
                    // Reset the thumbnail when the path changes
                    data.thumbnail = None;
                    data.preview = None;
                }
                "id" => {
                    let input_value: String = value
                        .get()
                        .expect("The value needs to be of type `String`.");
                    let mut data = self.data.lock().expect("Mutex is Poisoned.");
                    data.id = Uuid::try_parse(&input_value).expect("Unable to parse uuid");
                }
                "thumbnail" => {
                    let input_value: Option<Texture> = value.get().expect("Needs a texture.");
                    self.data.lock().expect("Mutex is poisoned.").thumbnail = input_value;
                }
                "preview" => {
                    let input_value: Option<Texture> = value.get().expect("Needs a texture.");
                    self.data.lock().expect("Mutex is poisoned.").preview = input_value;
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "path" => self.get_filepath().as_str().to_value(),
                "picked" => self
                    .get_picked()
                    .map_or(String::from("None"), |b| b.to_string())
                    .to_value(),
                "rating" => self.get_rating().to_value(),
                "flag" => self.get_flag().to_value(),
                "hidden" => self.get_hidden().unwrap_or(false).to_value(),
                "thumbnail" => self
                    .data
                    .as_ref()
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .thumbnail
                    .as_ref()
                    .to_value(),
                "preview" => self
                    .data
                    .as_ref()
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .preview
                    .as_ref()
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
