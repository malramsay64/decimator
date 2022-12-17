use adw::prelude::*;
use adw::subclass::prelude::*;
use gdk::Texture;
use glib::BindingFlags;
use glib::Object;

use gtk::{gdk, glib};
use rayon::spawn_fifo;

use super::PictureData;
use super::PictureObject;

glib::wrapper! {
    pub struct PictureThumbnail(ObjectSubclass<imp::PictureThumbnail>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl PictureThumbnail {
    pub fn new() -> Self {
        Object::builder().build()
    }

    #[tracing::instrument(name = "Binding thumbnail to widget.", level = "trace")]
    pub fn bind(&self, picture_object: &PictureObject) {
        let thumbnail_picture = self.imp().thumbnail_picture.get();
        let thumbnail_label = self.imp().thumbnail_label.get();
        let mut bindings = self.imp().bindings.borrow_mut();

        let label_binding = picture_object
            .bind_property("path", &thumbnail_label, "label")
            .flags(BindingFlags::SYNC_CREATE)
            .build();
        bindings.push(label_binding);

        let buffer_binding = picture_object
            .bind_property("thumbnail", &thumbnail_picture, "paintable")
            .flags(BindingFlags::SYNC_CREATE)
            .build();
        bindings.push(buffer_binding);

        match picture_object.property::<Option<Texture>>("thumbnail") {
            Some(_) => {}
            None => {
                let filepath: String = picture_object.property("path");
                let local_picture = picture_object.clone();
                spawn_fifo(move || {
                    let thumbnail = PictureData::get_thumbnail(&filepath);
                    // By using set_property we also trigger the signal telling
                    // GTK the thumbnail has been updated and the Picture
                    // should subsequently be updated.
                    local_picture.set_property("thumbnail", thumbnail);
                });
            }
        }
    }

    #[tracing::instrument(name = "Unbinding thumbnail from widget.", level = "trace")]
    pub fn unbind(&self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}

impl Default for PictureThumbnail {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::Binding;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{CompositeTemplate, Label, Picture};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/resources/picture_thumbnail.ui")]
    pub struct PictureThumbnail {
        #[template_child]
        pub thumbnail_picture: TemplateChild<Picture>,
        #[template_child]
        pub thumbnail_label: TemplateChild<Label>,
        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PictureThumbnail {
        const NAME: &'static str = "PictureThumbnail";
        type Type = super::PictureThumbnail;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for PictureThumbnail {}

    // Trait shared by all widgets
    impl WidgetImpl for PictureThumbnail {}

    // Trait shared by all boxes
    impl BoxImpl for PictureThumbnail {}
}
