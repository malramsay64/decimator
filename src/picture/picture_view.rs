use adw::prelude::*;
use adw::subclass::prelude::*;

use glib::Object;
use gtk::gdk::Texture;
use gtk::PolicyType;
use gtk::ScrollType;
use gtk::ScrolledWindow;
use gtk::SignalListItemFactory;
use gtk::SingleSelection;

use gtk::gdk_pixbuf::Pixbuf;
use gtk::gio::ListModel;
use gtk::gio::ListStore;
use gtk::glib;
use gtk::glib::clone;

use super::PictureData;
use super::PictureObject;
use super::PictureThumbnail;

glib::wrapper! {
    pub struct PictureView(ObjectSubclass<imp::PictureView>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl PictureView {
    pub fn new() -> Self {
        Object::builder().build()
    }

    #[tracing::instrument(
        name = "Setting thumbnail images.", 
        skip(self, pictures),
        fields(num_pictures=pictures.len()))
    ]
    pub fn set_thumbnails(&self, pictures: Vec<PictureData>) {
        let mut model = ListStore::new(PictureObject::static_type());
        model.extend(pictures.into_iter().map(PictureObject::from));

        // By building a new ListStore and replacing the current model we are
        // able to remove any issues with things occurring in multiple steps.
        // Here eveything occurs at the same time.
        self.imp().thumbnails.replace(Some(model));

        dbg!(&self.imp().thumbnails.borrow());

        self.init_factory();
        self.init_selection_model();
        // self.init_scroll();
    }

    #[tracing::instrument(name = "Setting preview image.", skip(self, picture_object))]
    fn set_preview(&self, picture_object: &PictureObject) {
        // TODO: This should potentially be much better an async function
        // let buffer = Pixbuf::from_file(picture_object.get_filepath().as_str())
        //     .expect("Image not found")
        //     .apply_embedded_orientation()
        //     .expect("Unable to apply image orientation.");
        self.imp()
            .preview_image
            .replace(Some(picture_object.clone()));
        // self.imp()
        //     .preview
        //     .replace(Some(Texture::for_pixbuf(&buffer)));
    }

    #[tracing::instrument(name = "Initialising selection model.", skip(self))]
    fn init_selection_model(&self) {
        let model: ListStore = self.imp().thumbnails.borrow().clone().unwrap();
        let selection_model = SingleSelection::builder()
            .autoselect(true)
            .model(&model)
            .build();

        selection_model.connect_selected_item_notify(clone!(@weak self as window => move |item| {
            window.set_preview(
                &item
                .selected_item()
                .expect("No items selected")
                .downcast::<PictureObject>()
                .expect("Item has to be a `PictureObject`")
            );
        }));

        self.imp().thumbnail_list.set_model(Some(&selection_model));
        self.set_preview(
            &selection_model
                .selected_item()
                .expect("No items selected")
                .downcast::<PictureObject>()
                .expect("Object neeeds to be a `PictureObject`"),
        );
    }

    //* This needs to be done after the
    #[tracing::instrument(name = "Initialising thumbnail factory.", skip(self))]
    fn init_factory(&self) {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let thumbnail = PictureThumbnail::new();
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
                .downcast::<PictureThumbnail>()
                .expect("The child has to be a `ThumbnailPicture`.");

            image_thumbnail.bind(&picture_object);
        });

        factory.connect_unbind(move |_, list_item| {
            let image_thumbnail = list_item
                .child()
                .expect("The child has to exist.")
                .downcast::<PictureThumbnail>()
                .expect("The child has to be a `ThumbnailPicture`.");

            image_thumbnail.unbind();
        });

        self.imp().thumbnail_list.set_factory(Some(&factory));
    }
}

impl Default for PictureView {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use gtk::gdk::Texture;

    use std::cell::RefCell;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use gio::ListStore;
    use glib::Binding;
    use gtk::{gio, glib};
    use gtk::{CompositeTemplate, ListView, Picture, ScrolledWindow};

    use super::PictureObject;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/resources/picture_view.ui")]
    pub struct PictureView {
        #[template_child]
        pub thumbnail_scroll: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub preview: TemplateChild<Picture>,
        pub thumbnails: RefCell<Option<ListStore>>,
        #[template_child]
        pub thumbnail_list: TemplateChild<ListView>,
        pub preview_image: RefCell<Option<PictureObject>>,
        pub bindings: RefCell<Vec<Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PictureView {
        const NAME: &'static str = "PictureView";
        type Type = super::PictureView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PictureView {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.init_factory();
            // obj.init_scroll();
        }
    }

    impl WidgetImpl for PictureView {}

    impl BoxImpl for PictureView {}
}
