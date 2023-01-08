use std::collections::HashSet;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::Application;
use camino::{Utf8Path, Utf8PathBuf};
use gio::ListStore;
use glib::{clone, Object};
use gtk::pango::EllipsizeMode;
use gtk::{
    gio, glib, Align, FileChooserAction, FileChooserDialog, Label, PolicyType, ResponseType,
    ScrollType, ScrolledWindow, SignalListItemFactory, SingleSelection, StringObject, Widget,
};
use sqlx::{QueryBuilder, Sqlite};
use tokio::sync::oneshot;
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

use crate::data::{query_directory_pictures, query_existing_pictures, query_unique_directories};
use crate::picture::{PictureData, PictureObject, PicturePath, PictureThumbnail};

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

fn is_image(entry: &DirEntry) -> bool {
    match entry.path().extension().and_then(|s| s.to_str()) {
        Some("jpg" | "JPG") => true,
        Some("tiff" | "png" | "gif" | "RAW" | "webp" | "heif" | "heic" | "arw" | "ARW") => false,
        _ => false,
    }
}

impl Window {
    pub fn new(app: &Application) -> Self {
        Object::builder().property("application", app).build()
    }

    fn thumbnails(&self) -> ListStore {
        self.imp()
            .thumbnails
            .borrow()
            .clone()
            .expect("`thumbnails` should be set up in setup_path")
    }

    fn directories(&self) -> ListStore {
        self.imp()
            .directories
            .borrow()
            .clone()
            .expect("`directories` should be set up in setup_path")
    }

    #[tracing::instrument(name = "Updating window display path", skip(self))]
    pub fn set_path(&self, path: String) {
        let runtime = self.imp().runtime.clone();
        let db = self.imp().database.clone();
        let (tx, mut rx) = oneshot::channel();
        runtime.as_ref().block_on(async move {
            let results = query_directory_pictures(db.as_ref(), path).await.unwrap();
            tx.send(results).unwrap();
        });
        let directories: Vec<PictureData> = rx.try_recv().unwrap();
        let mut model = ListStore::new(PictureObject::static_type());
        model.extend(directories.into_iter().map(PictureObject::from));

        self.imp().thumbnails.replace(Some(model));
        self.init_selection_model();
    }

    #[tracing::instrument(name = "Initialising selection model.", skip(self))]
    fn init_selection_model(&self) {
        let selection_model = SingleSelection::builder()
            .autoselect(true)
            .model(&self.thumbnails())
            .build();

        selection_model.connect_selected_item_notify(clone!(@weak self as window => move |item| {
            let picture = item
                .selected_item()
                .expect("No items selected")
                .downcast::<PictureObject>().expect("Require a `PictureObject`");

            window.imp().preview.bind(&picture);
        }));

        self.imp().thumbnail_list.set_model(Some(&selection_model));

        // Select the first item in the list when we initialise so there will
        // always be something selected.
        selection_model.select_item(0, true);

        self.imp().preview.bind(
            &selection_model
                .selected_item()
                .unwrap()
                .downcast::<PictureObject>()
                .unwrap(),
        );
    }

    #[tracing::instrument(name = "Retrieving existing pictures within the database", skip(self))]
    fn get_existing_paths(&self, directory: &Utf8Path) -> HashSet<PicturePath> {
        let runtime = self.imp().runtime.clone();
        let db = self.imp().database.clone();
        let (tx, mut rx) = oneshot::channel();

        runtime.as_ref().block_on(async move {
            let existing_pictures = query_existing_pictures(db.as_ref(), directory.to_string())
                .await
                .unwrap();

            tx.send(existing_pictures).unwrap();
        });

        let pictures: Vec<PicturePath> = rx.try_recv().unwrap();

        HashSet::from_iter(pictures.into_iter())
    }

    #[tracing::instrument(name = "Adding pictures from directory", skip(self))]
    fn add_pictures_from(&self, directory: &Utf8Path) {
        let existing_pictures = self.get_existing_paths(directory);

        tracing::info!(
            "Found {} existing files within directory",
            existing_pictures.len()
        );

        let images: Vec<_> = WalkDir::new(directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(is_image)
            .map(|p: DirEntry| Utf8PathBuf::try_from(p.into_path()).expect("Invalid UTF-8 path."))
            .map(PicturePath::from)
            .filter(|p| !existing_pictures.contains(p))
            .collect();

        if images.is_empty() {
            tracing::info!("No new images found in directory {directory}");
            return;
        } else {
            tracing::info!("Adding {} new images to the database.", images.len());
        }

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("INSERT INTO picture(id, directory, filename)");

        query_builder.push_values(images, |mut b, picture| {
            b.push_bind(Uuid::new_v4())
                .push_bind(picture.directory)
                .push_bind(picture.filename);
        });

        // Run the database query to update all the files
        let runtime = self.imp().runtime.clone();
        let db = self.imp().database.clone();
        let query = query_builder.build();

        runtime.as_ref().block_on(async move {
            query.execute(db.as_ref()).await.unwrap();
        });

        // We need to update the list of directories
        self.init_tree();
        self.init_tree_model();
    }

    #[tracing::instrument(name = "Selecting new directory dialog.", skip(self))]
    fn new_directory(&self) {
        let dialog = FileChooserDialog::new(
            Some("Choose Directory"),
            Some(self),
            FileChooserAction::SelectFolder,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Select", ResponseType::Accept),
            ],
        );
        dialog.connect_response(clone!(@weak self as window => move |dialog, response| {
            let directory: Utf8PathBuf;
            if response != ResponseType::Accept {
                dialog.destroy();
                return;
            } else {
                directory = dialog.file().expect("No folder selected").path().expect("Unable to convert to path").try_into().expect("Unable to convert to UTF-8 string.");
                dialog.destroy();
            }
            //
            window.add_pictures_from(&directory);
        }));
        dialog.present();
    }

    #[tracing::instrument(name = "Setting up Actions.", skip(self))]
    fn setup_actions(&self) {
        // Create action to create new collection and add to action group "win"
        let action_new_directory = gio::SimpleAction::new("new-directory", None);
        action_new_directory.connect_activate(clone!(@weak self as window => move |_, _| {
            window.new_directory();
        }));
        self.add_action(&action_new_directory);
    }

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
                .expect("The child has to be a `PictureThumbnail`.");

            image_thumbnail.bind(&picture_object);
        });

        factory.connect_unbind(move |_, list_item| {
            let image_thumbnail = list_item
                .child()
                .expect("The child has to exist.")
                .downcast::<PictureThumbnail>()
                .expect("The child has to be a `PictureThumbnail`.");

            image_thumbnail.unbind();
        });

        self.imp().thumbnail_list.set_factory(Some(&factory));
    }

    #[tracing::instrument(name = "Initialising directory tree.", skip(self))]
    fn init_tree(&self) {
        let runtime = self.imp().runtime.clone();
        let db = self.imp().database.clone();
        let (tx, mut rx) = oneshot::channel();
        runtime.as_ref().block_on(async move {
            let results = query_unique_directories(db.as_ref()).await.unwrap();
            tx.send(results).unwrap();
        });
        let directories: Vec<String> = rx.try_recv().unwrap();

        let mut list_model = ListStore::new(StringObject::static_type());
        list_model.extend(
            directories
                .into_iter()
                .map(|s: String| StringObject::new(&s)),
        );

        self.imp().directories.replace(Some(list_model));
    }

    #[tracing::instrument(name = "Initialising Scrolling", skip(self))]
    fn init_scroll(&self) {
        let scroll = ScrolledWindow::builder()
            .focus_on_click(true)
            .overlay_scrolling(false)
            .has_frame(true)
            .vscrollbar_policy(PolicyType::Never)
            .hscrollbar_policy(PolicyType::Always)
            .propagate_natural_height(true)
            .build();
        scroll.emit_scroll_child(ScrollType::StepForward, true);

        self.imp()
            .thumbnail_scroll
            .emit_scroll_child(ScrollType::End, true);
    }

    #[tracing::instrument(name = "Initialising directory tree Model.", skip(self))]
    fn init_tree_model(&self) {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let label = Label::builder()
                .ellipsize(EllipsizeMode::Start)
                .lines(1)
                .halign(Align::Start)
                .width_request(280)
                .build();
            list_item.set_child(Some(&label));

            list_item
                .property_expression("item")
                .chain_property::<StringObject>("string")
                .bind(&label, "label", Widget::NONE);
        });

        self.imp().filetree.set_factory(Some(&factory));
    }

    #[tracing::instrument(name = "Initialising filetree selection", skip(self))]
    fn init_tree_selection_model(&self) {
        let selection_model = SingleSelection::builder()
            .model(&self.directories())
            .build();

        selection_model.connect_selected_item_notify(clone!(@weak self as window => move |item| {
            let file_path = item
                .selected_item()
                .expect("No items selected")
                .downcast::<StringObject>()
                .expect("The item has to be a `String`.")
                .property::<String>("string");

            window.set_path(file_path)
        }));

        self.imp().filetree.set_model(Some(&selection_model));
    }
}

mod imp {

    use std::cell::RefCell;
    use std::fs::File;
    use std::sync::Arc;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use gio::ListStore;
    use glib::subclass::InitializingObject;
    use gtk::{gio, glib, CompositeTemplate, ListView, ScrolledWindow};
    use sqlx::SqlitePool;
    use tokio::runtime::{Builder as RuntimeBuilder, Runtime};
    use tokio::sync::oneshot;

    use crate::picture::{PictureData, PictureObject, PicturePreview};

    #[derive(CompositeTemplate)]
    #[template(resource = "/resources/decimator.ui")]
    pub struct Window {
        #[template_child]
        pub thumbnail_scroll: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub preview: TemplateChild<PicturePreview>,
        #[template_child]
        pub thumbnail_list: TemplateChild<ListView>,
        #[template_child]
        pub filetree: TemplateChild<ListView>,
        pub thumbnails: RefCell<Option<ListStore>>,
        pub directories: RefCell<Option<ListStore>>,
        pub runtime: Arc<Runtime>,
        pub database: Arc<SqlitePool>,
    }

    impl Default for Window {
        fn default() -> Self {
            let runtime: Arc<Runtime> = Arc::new(
                RuntimeBuilder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Unable to initialise tokio runtime."),
            );
            let mut path = glib::user_data_dir();
            path.push(crate::APP_ID);
            std::fs::create_dir_all(&path).expect("Could not create directory.");
            // We use rwc to create the file if it doesn't alrPool
            let database_path = format!("sqlite://{}/database.db?mode=rwc", path.display());
            let (tx, mut rx) = oneshot::channel();

            runtime.as_ref().block_on(async move {
                let pool = SqlitePool::connect(&database_path)
                    .await
                    .expect("Unable to initialise sqlite database");
                tx.send(pool).unwrap();
            });
            let database = Arc::new(rx.try_recv().unwrap());

            Self {
                thumbnail_scroll: Default::default(),
                preview: Default::default(),
                thumbnail_list: Default::default(),
                filetree: Default::default(),
                thumbnails: Default::default(),
                directories: Default::default(),
                runtime,
                database,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "DecimatorWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

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

            obj.init_scroll();
            obj.init_factory();
            obj.init_tree();
            obj.init_tree_selection_model();
            obj.init_tree_model();
            obj.setup_actions();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for Window {}

    // Trait shared by all windows
    impl WindowImpl for Window {
        fn close_request(&self) -> glib::signal::Inhibit {
            let backup_data: Vec<PictureData> = self
                .obj()
                .thumbnails()
                .snapshot()
                .iter()
                .filter_map(Cast::downcast_ref::<PictureObject>)
                .map(PictureData::from)
                .collect();

            let mut path = glib::user_data_dir();
            path.push(crate::APP_ID);
            std::fs::create_dir_all(&path).expect("Could not create directory.");
            path.push("data.json");

            let file = File::create(path).expect("Could not create json file.");
            serde_json::to_writer(file, &backup_data).expect("Could not write data to json file");

            self.parent_close_request()
        }
    }

    // Trait shared by all application windows
    impl ApplicationWindowImpl for Window {}

    impl AdwApplicationWindowImpl for Window {}
}
