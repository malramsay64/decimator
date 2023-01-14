use std::str::FromStr;

use adw::subclass::prelude::*;
use camino::{Utf8Path, Utf8PathBuf};
use gdk::Texture;
use glib::Object;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::{gdk, glib};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};
use uuid::Uuid;

use crate::picture::{Flag, Rating, Selection};

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
    fn get_picked(&self) -> Selection {
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

    pub fn pick(&self) {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .picked = Selection::Picked
    }
    pub fn deselect(&self) {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .picked = Selection::None
    }
    pub fn reject(&self) {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .picked = Selection::Rejected
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PictureData {
    pub id: Uuid,
    pub filepath: Utf8PathBuf,
    pub picked: Selection,
    pub rating: Option<Rating>,
    pub flag: Option<Flag>,
    pub hidden: Option<bool>,
    #[serde(skip)]
    pub thumbnail: Option<Texture>,
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
        }
    }
}

impl FromRow<'_, SqliteRow> for PictureData {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let id = row.try_get("id")?;
        let directory: &str = row.try_get("directory")?;
        let filename: &str = row.try_get("filename")?;
        let filepath = Utf8Path::new(directory).join(filename);
        let picked_string: Option<String> = row.try_get("picked")?;
        let picked: Selection =
            picked_string.map_or(Selection::None, |s| Selection::from_str(&s).unwrap());
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
    pub fn get_thumbnail(path: &str, (scale_x, scale_y): (i32, i32)) -> Texture {
        let image = Pixbuf::from_file_at_scale(path, scale_x, scale_y, true)
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
    use glib::{ParamSpec, ParamSpecObject, ParamSpecString, Value};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{gdk, glib};
    use once_cell::sync::Lazy;
    use uuid::Uuid;

    use crate::picture::pick::Selection;

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
        fn get_picked(&self) -> Selection {
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
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "path" => self.get_filepath().as_str().to_value(),
                "picked" => self.get_picked().to_value(),
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
                _ => unimplemented!(),
            }
        }
    }
}
