use glib::Object;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct PictureObject(ObjectSubclass<imp::PictureObject>);
}

impl PictureObject {
    pub fn new(path: String) -> Self {
        Object::builder()
            .property("path", path)
            .property::<Option<Pixbuf>>("thumbnail", None)
            .build()
    }
}

#[derive(Default)]
pub struct PictureData {
    pub path: String,
    pub thumbnail: Option<Pixbuf>,
}

impl PictureData {
    fn update_thumbnail(&mut self) {
        let thumbnail = Pixbuf::from_file_at_scale(&self.path, 320, 320, true)
            .expect("Image not found")
            .apply_embedded_orientation()
            .expect("Unable to apply image orientation.");

        self.thumbnail = Some(thumbnail);
    }
}

mod imp {
    use std::cell::RefCell;
    use std::rc::Rc;

    use glib::ParamSpecObject;
    use glib::{ParamSpec, ParamSpecString, Value};
    use gtk::gdk_pixbuf::Pixbuf;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use once_cell::sync::Lazy;

    #[derive(Default)]
    pub struct PictureObject {
        pub data: Rc<RefCell<super::PictureData>>,
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
                    ParamSpecObject::builder::<Option<Pixbuf>>("thumbnail").build(),
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
                    self.data.borrow_mut().path = input_value;
                    // Reset the thumbnail when the path changes
                    self.data.borrow_mut().thumbnail = None;
                }
                "thumbnail" => {
                    let input_value = value.get().expect("Needs a Pixbuf.");
                    self.data.borrow_mut().thumbnail = input_value;
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "path" => self.data.borrow().path.to_value(),
                "thumbnail" => {
                    let mut data = self.data.borrow_mut();
                    match &data.thumbnail {
                        Some(val) => val.to_value(),
                        None => {
                            data.update_thumbnail();
                            data.thumbnail.to_value()
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }
    }
}
