use std::io::Cursor;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gdk::Texture;
use glib::{Bytes, Object};
use gtk::gdk_pixbuf::Pixbuf;
use gtk::{gdk, glib};
use image::imageops::FilterType::Nearest;
use image::io::Reader as ImageReader;
use image::{imageops, ImageOutputFormat};

glib::wrapper! {
    pub struct PictureObject(ObjectSubclass<imp::PictureObject>);
}

impl PictureObject {
    pub fn new(path: String) -> Self {
        Object::builder()
            .property("path", path)
            .property::<Option<Texture>>("thumbnail", None)
            .build()
    }
}

#[derive(Default, Clone)]
pub struct PictureData {
    pub path: String,
    pub thumbnail: Option<Texture>,
}

impl std::fmt::Debug for PictureData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let loaded = match &self.thumbnail {
            Some(_) => true,
            None => false,
        };
        f.debug_struct("PictureData")
            .field("path", &self.path)
            .field("thumbnail", &loaded)
            .finish()
    }
}

impl PictureData {
    #[tracing::instrument(name = "Loading thumbnail from file using ImageReader")]
    pub fn get_thumbnail(path: &str) -> Texture {
        let image = Pixbuf::from_file_at_scale(path, 320, 320, true)
            .expect("Image not found.")
            .apply_embedded_orientation()
            .expect("Unable to apply orientation.");
        Texture::for_pixbuf(&image)
        // let image = ImageReader::open(path)
        //     .expect("Error opening file")
        //     .decode()
        //     .expect("Error decoding file");
        // let mut buffer = Cursor::new(Vec::new());
        // imageops::resize(&image, 320, 320, Nearest)
        //     .write_to(&mut buffer, ImageOutputFormat::Png)
        //     .expect("Writing to in memory file failed.");
        // Texture::from_bytes(&Bytes::from(buffer.get_ref()))
        //     .expect("Error parsing bytes to texture.")
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
                    ParamSpecString::builder("path").build(),
                    ParamSpecObject::builder::<Option<Texture>>("thumbnail").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "path" => {
                    let input_value = value
                        .get()
                        .expect("The value needs to be of type `String`.");
                    let mut data = self.data.lock().expect("Mutex is Poisoned.");
                    data.path = input_value;
                    // Reset the thumbnail when the path changes
                    data.thumbnail = None;
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
                    .path
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
