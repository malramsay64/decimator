use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::BindingFlags;

use glib::Object;
use gtk::gdk::Texture;

use gtk::glib;

use crate::picture_object::PictureObject;

glib::wrapper! {
pub struct PicturePreview(ObjectSubclass<imp::PicturePreview>)
    @extends gtk::Frame, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PicturePreview {
    pub fn new() -> Self {
        Object::builder().build()
    }

    #[tracing::instrument(name = "Unbinding preview from widget.")]
    pub fn bind(&self, picture_object: &PictureObject) {
        let preview_picture = self.imp().preview_picture.get();
        let rating = self.imp().rating.get();
        let mut bindings = self.imp().bindings.borrow_mut();

        let rating_binding = picture_object
            .bind_property("rating", &rating, "label")
            .flags(BindingFlags::SYNC_CREATE)
            .build();

        bindings.push(rating_binding);

        let picture_binding = picture_object
            .bind_property("path", &preview_picture, "filename")
            .flags(BindingFlags::SYNC_CREATE)
            .build();

        bindings.push(picture_binding);
    }

    #[tracing::instrument(name = "Unbinding preview from widget.")]
    pub fn unbind(&self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::Binding;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{CompositeTemplate, Frame, Label, Picture};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/resources/thumbnail_image.ui")]
    pub struct PicturePreview {
        #[template_child]
        pub preview_picture: TemplateChild<Picture>,
        #[template_child]
        pub rating: TemplateChild<Label>,
        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PicturePreview {
        const NAME: &'static str = "PreviewPicture";
        type Type = super::PicturePreview;
        type ParentType = gtk::Frame;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for PicturePreview {}

    // Trait shared by all widgets
    impl WidgetImpl for PicturePreview {}

    // Trait shared by all boxes
    impl FrameImpl for PicturePreview {}
}
