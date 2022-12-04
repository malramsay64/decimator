use gio::File;
use glib::BindingFlags;
use glib::Object;
use gtk::gdk::Texture;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::picture_object::PictureObject;

glib::wrapper! {
    pub struct ThumbnailPicture(ObjectSubclass<imp::ThumbnailPicture>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ThumbnailPicture {
    pub fn new() -> Self {
        Object::builder().build()
    }

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
            .transform_to(|_, thumbnail: Option<Pixbuf>| thumbnail.map(|t| Texture::for_pixbuf(&t)))
            .build();
        bindings.push(buffer_binding);
    }

    pub fn unbind(&self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::Binding;
    use gtk::subclass::prelude::*;
    use gtk::{glib, CompositeTemplate, Picture};
    use gtk::{prelude::*, Label};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/resources/thumbnail_image.ui")]
    pub struct ThumbnailPicture {
        #[template_child]
        pub thumbnail_picture: TemplateChild<Picture>,
        #[template_child]
        pub thumbnail_label: TemplateChild<Label>,
        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ThumbnailPicture {
        const NAME: &'static str = "ThumbnailPicture";
        type Type = super::ThumbnailPicture;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for ThumbnailPicture {}

    // Trait shared by all widgets
    impl WidgetImpl for ThumbnailPicture {}

    // Trait shared by all boxes
    impl BoxImpl for ThumbnailPicture {}
}
