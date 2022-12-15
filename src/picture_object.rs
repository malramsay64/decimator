use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use adw::prelude::*;
use adw::subclass::prelude::*;
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
    fn get_filepath(&self) -> Utf8PathBuf {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .filepath
            .clone()
            .into()
    }
    fn get_id(&self) -> Uuid {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .id
            .into()
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PictureData {
    pub id: Uuid,
    pub filepath: Utf8PathBuf,
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
        Ok(Self {
            id,
            filepath,
            thumbnail: None,
        })
    }
}

impl std::fmt::Debug for PictureData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let loaded = match &self.thumbnail {
            Some(_) => true,
            None => false,
        };
        f.debug_struct("PictureData")
            .field("id", &self.id)
            .field("path", &self.filepath)
            .field("thumbnail", &loaded)
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
}

mod imp {
    use std::sync::{Arc, Mutex};

    use gdk::Texture;
    use glib::ParamSpecObject;
    use glib::{ParamSpec, ParamSpecString, Value};

    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{gdk, glib};
    use once_cell::sync::Lazy;
    use uuid::Uuid;

    use super::PictureData;

    #[derive(Default)]
    pub struct PictureObject {
        pub data: Arc<Mutex<PictureData>>,
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
                "path" => self
                    .data
                    .lock()
                    .expect("Mutex is poisoned.")
                    .filepath
                    .to_string()
                    .to_value(),
                "thumbnail" => self
                    .data
                    .lock()
                    .expect("Mutex is poisoned.")
                    .thumbnail
                    .as_ref()
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
