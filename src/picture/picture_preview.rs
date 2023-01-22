use adw::prelude::*;
use adw::subclass::prelude::*;
use gdk::Texture;
use glib::{BindingFlags, Object};
use gtk::builders::ToggleButtonBuilder;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::{gdk, glib, ToggleButton};

use super::{PictureObject, Selection};

glib::wrapper! {
    pub struct PicturePreview(ObjectSubclass<imp::PicturePreview>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl PicturePreview {
    pub fn new() -> Self {
        Object::builder().build()
    }

    #[tracing::instrument(name = "Binding preview to widget.")]
    pub fn bind(&self, picture_object: &PictureObject) {
        let picture = self.imp().preview.get();
        let rating = self.imp().rating.get();
        let selection = self.imp().selection.get();
        let mut bindings = self.imp().bindings.borrow_mut();

        let rating_binding = picture_object
            // TODO Bind rating, using path because it is always set
            .bind_property("path", &rating, "label")
            .flags(BindingFlags::SYNC_CREATE)
            .build();
        bindings.push(rating_binding);

        let buffer_binding = picture_object
            .bind_property("path", &picture, "paintable")
            .flags(BindingFlags::SYNC_CREATE)
            .transform_to(|_, p: String| {
                Some(Texture::for_pixbuf(
                    &Pixbuf::from_file(p)
                        .expect("image not found")
                        .apply_embedded_orientation()
                        .expect("Unagle to apply orientation"),
                ))
            })
            .build();

        bindings.push(buffer_binding);
    }

    #[tracing::instrument(name = "Unbinding preview from widget.", level = "trace")]
    pub fn unbind(&self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }
    }

    #[tracing::instrument(name = "Initialising toggles", level = "trace")]
    pub fn init_toggles(&self) {
        let selection = self.imp().selection.get();
        let ignore = ToggleButtonBuilder::new()
            .label("I")
            .action_name("win.image-select")
            .action_target(&Selection::Ignore.to_variant())
            .build();
        let ordinary = ToggleButtonBuilder::new()
            .label("O")
            .group(&ignore)
            .action_name("win.image-select")
            .action_target(&Selection::Ordinary.to_variant())
            .build();
        let pick = ToggleButtonBuilder::new()
            .label("P")
            .group(&ignore)
            .action_name("win.image-select")
            .action_target(&Selection::Pick.to_variant())
            .build();
        selection.append(&ignore);
        selection.append(&ordinary);
        selection.append(&pick);
    }
}

impl Default for PicturePreview {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::Binding;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Box, CompositeTemplate, Label, Picture};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/resources/picture_preview.ui")]
    pub struct PicturePreview {
        #[template_child]
        pub preview: TemplateChild<Picture>,
        #[template_child]
        pub rating: TemplateChild<Label>,
        #[template_child]
        pub selection: TemplateChild<Box>,
        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PicturePreview {
        const NAME: &'static str = "PicturePreview";
        type Type = super::PicturePreview;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for PicturePreview {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();

            // Setup
            let obj = self.obj();

            obj.init_toggles();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for PicturePreview {}

    // Trait shared by all boxes
    impl BoxImpl for PicturePreview {}
}
