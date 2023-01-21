use adw::subclass::prelude::*;
use camino::Utf8PathBuf;
use gdk::Texture;
use glib::Object;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::{gdk, glib};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};
use time::PrimitiveDateTime;
use uuid::Uuid;

use super::PictureData;
use crate::picture::{DateTime, Flag, Rating, Selection};

glib::wrapper! {
    pub struct PictureObject(ObjectSubclass<imp::PictureObject>);
}

impl PictureObject {
    pub fn filepath(&self) -> Utf8PathBuf {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .filepath
            .clone()
    }

    pub fn id(&self) -> Uuid {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .id
    }

    fn capture_time(&self) -> Option<DateTime> {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutext lock is poisoned")
            .capture_time
    }

    fn selection(&self) -> Selection {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .selection
    }
    fn rating(&self) -> Rating {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .rating
    }
    fn flag(&self) -> Flag {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .flag
    }
    fn hidden(&self) -> Option<bool> {
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
            .selection = Selection::Pick
    }
    pub fn ordinary(&self) {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .selection = Selection::Ordinary
    }
    pub fn ignore(&self) {
        self.imp()
            .data
            .as_ref()
            .lock()
            .expect("Mutex lock is poisoned")
            .selection = Selection::Ignore
    }
}
impl<T: AsRef<PictureObject>> From<T> for PictureData {
    fn from(p: T) -> Self {
        let p = p.as_ref();
        Self {
            id: p.id(),
            filepath: p.filepath(),
            raw_extension: None,
            capture_time: p.capture_time(),
            selection: p.selection(),
            rating: p.rating(),
            flag: p.flag(),
            hidden: p.hidden(),
            thumbnail: None,
        }
    }
}

impl From<PictureData> for PictureObject {
    fn from(pic: PictureData) -> Self {
        Object::builder()
            .property("id", pic.id.to_string())
            .property("path", pic.path())
            .property("selection", pic.selection.to_string())
            .property("capture-time", pic.capture_time.map(|c| c.to_string()))
            .property::<Option<Texture>>("thumbnail", None)
            .build()
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

    use crate::picture::{DateTime, Flag, PictureData, Rating, Selection};

    #[derive(Default)]
    pub struct PictureObject {
        pub data: Arc<Mutex<PictureData>>,
    }

    impl PictureObject {
        fn id(&self) -> Uuid {
            self.data
                .as_ref()
                .lock()
                .expect("Mutex lock is poisoned")
                .id
        }

        fn filepath(&self) -> Utf8PathBuf {
            self.data
                .as_ref()
                .lock()
                .expect("Mutex lock is poisoned")
                .filepath
                .clone()
        }

        fn capture_time(&self) -> Option<DateTime> {
            self.data
                .as_ref()
                .lock()
                .expect("Mutex lock is poisoned")
                .capture_time
        }

        fn selection(&self) -> Selection {
            self.data
                .as_ref()
                .lock()
                .expect("Mutex lock is poisoned")
                .selection
        }

        fn rating(&self) -> Rating {
            self.data
                .as_ref()
                .lock()
                .expect("Mutex lock is poisoned")
                .rating
        }
        fn flag(&self) -> Flag {
            self.data
                .as_ref()
                .lock()
                .expect("Mutex lock is poisoned")
                .flag
        }
        fn hidden(&self) -> Option<bool> {
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
                    ParamSpecString::builder("capture-time").build(),
                    ParamSpecString::builder("selection").build(),
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
                "selection" => {
                    let input_value: Selection = value.get().expect("Needs a `Selection`.");
                    self.data.lock().expect("Mutex is poisoned.").selection = input_value;
                }
                "thumbnail" => {
                    let input_value: Option<Texture> = value.get().expect("Needs a texture.");
                    self.data.lock().expect("Mutex is poisoned.").thumbnail = input_value;
                }
                "capture-time" => {
                    self.data.lock().expect("Mutex is Poisoned.").capture_time =
                        value.get().expect("Needs a string.")
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "id" => self.id().to_string().to_value(),
                "path" => self.filepath().as_str().to_value(),
                "capture-time" => self.capture_time().to_value(),
                "selection" => self.selection().to_value(),
                "rating" => self.rating().to_value(),
                "flag" => self.flag().to_value(),
                "hidden" => self.hidden().unwrap_or(false).to_value(),
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
