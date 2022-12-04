use std::path::Path;

use gio::ListStore;
use glib::{clone, Object};
use gtk::gdk_pixbuf::Pixbuf;
use gtk::subclass::prelude::*;
use gtk::{glib, Application, Picture, SingleSelection, StringList, StringObject};
use gtk::{prelude::*, SignalListItemFactory};
use log::trace;
use walkdir::{DirEntry, WalkDir};

use crate::picture_object::PictureObject;
use crate::thumbnail_image::ThumbnailPicture;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

fn is_image(entry: &DirEntry) -> bool {
    match entry.path().extension().map(|s| s.to_str()).flatten() {
        Some("jpg" | "JPG" | "heic") => true,
        Some("tiff" | "png" | "gif" | "RAW" | "webp" | "heif" | "arw" | "ARW") => false,
        _ => false,
    }
}

impl Window {
    pub fn new(app: &Application) -> Self {
        Object::builder().property("application", app).build()
    }

    fn thumbnails(&self) -> Option<ListStore> {
        self.imp().thumbnails.borrow().clone()
    }

    pub fn set_path(&self, path: impl AsRef<Path>) {
        let mut model = ListStore::new(PictureObject::static_type());
        model.extend(
            WalkDir::new(path)
                .into_iter()
                // Ignore any directories we don't have permissions for
                .filter_map(|e| e.ok())
                // This removes the directories from the listing, only giving the images
                .filter(is_image)
                .map(|p| p.path().to_str().expect("Invalid UTF8 path.").to_owned())
                .map(PictureObject::new),
        );

        self.imp().thumbnails.replace(Some(model));
        self.init_selection_model();
    }

    fn set_preview(&self, path: String) {
        let buffer = Pixbuf::from_file(&path)
            .expect("Image not found")
            .apply_embedded_orientation()
            .expect("Unable to apply image orientation.");

        self.imp().preview.set_pixbuf(Some(&buffer))
    }

    fn init_selection_model(&self) {
        let selection_model = SingleSelection::builder()
            .model(&self.thumbnails().expect("Thumbnails not set yet"))
            .autoselect(true)
            .build();

        selection_model.connect_selected_item_notify(clone!(@weak self as window => move |item| {
            let file_path = item
                .selected_item()
                .expect("No items selected")
                .downcast::<PictureObject>()
                .expect("The item has to be a `String`.")
                .property::<String>("path");

            dbg!(&file_path);
            window.set_preview(file_path)
        }));

        self.imp().thumbnail_list.set_model(Some(&selection_model));
    }

    fn init_factory(&self) {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let thumbnail = ThumbnailPicture::new();
            list_item.set_child(Some(&thumbnail));
        });

        factory.connect_bind(move |_, list_item| {
            let picture_object = list_item
                .item()
                .expect("The item has to exist.")
                .downcast::<PictureObject>()
                .expect("The item has to be an `PictureObject`.");

            let image_thumbnail = list_item
                .child()
                .expect("The child has to exist.")
                .downcast::<ThumbnailPicture>()
                .expect("The child has to be a `ThumbnailPicture`.");

            image_thumbnail.bind(&picture_object);
        });

        factory.connect_unbind(move |_, list_item| {
            let image_thumbnail = list_item
                .child()
                .expect("The child has to exist.")
                .downcast::<ThumbnailPicture>()
                .expect("The child has to be a `ThumbnailPicture`.");

            image_thumbnail.unbind();
        });

        self.imp().thumbnail_list.set_factory(Some(&factory));
    }
}

mod imp {
    use std::cell::RefCell;

    use gio::ListStore;
    use glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{gio, glib, CompositeTemplate, ListView, Picture, StringList};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/resources/decimator.ui")]
    pub struct Window {
        #[template_child]
        pub preview: TemplateChild<Picture>,
        #[template_child]
        pub thumbnail_list: TemplateChild<ListView>,
        pub thumbnails: RefCell<Option<ListStore>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "DecimatorWindow";
        type Type = super::Window;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for Window {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();

            // Setup
            let obj = self.obj();

            obj.set_path(String::from("/home/malcolm/Pictures/2022/2022-04-14"));
            obj.init_factory();
            obj.init_selection_model();
            // obj.setup_callbacks();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for Window {}

    // Trait shared by all windows
    impl WindowImpl for Window {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for Window {}
}
