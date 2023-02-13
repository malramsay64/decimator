use std::cmp::Ord;
use std::collections::HashSet;

use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::Application;
use camino::{Utf8Path, Utf8PathBuf};
use gio::{ListStore, SimpleAction};
use glib::{clone, FromVariant, Object};
use gtk::pango::EllipsizeMode;
use gtk::{
    gio, glib, Align, CustomFilter, CustomSorter, FileChooserAction, FileChooserDialog,
    FilterListModel, Label, ListItem, PolicyType, ResponseType, ScrollType, ScrolledWindow,
    SignalListItemFactory, SingleSelection, SortListModel, StringObject, Widget,
};
use sqlx::SqlitePool;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tracing::Level;

use crate::data::{
    add_new_images, query_directory_pictures, query_existing_pictures, query_unique_directories,
    update_selection_state,
};
use crate::import::{import, map_directory_images};
use crate::picture::{PictureData, PictureObject, PictureThumbnail, Selection};

#[derive(Default, Debug, Clone)]
pub enum FilterState {
    #[default]
    Include,
    Exclude,
}

// Where we have a True value this
#[derive(Debug, Default, Clone)]
pub struct FilterSettings {
    selection_ignore: FilterState,
    selection_ordinary: FilterState,
    selection_pick: FilterState,
}

impl FilterSettings {
    fn filter(&self, picture: &PictureObject) -> bool {
        match picture.selection() {
            Selection::Ignore => match self.selection_ignore {
                FilterState::Include => true,
                FilterState::Exclude => false,
            },
            Selection::Ordinary => match self.selection_ordinary {
                FilterState::Include => true,
                FilterState::Exclude => false,
            },
            Selection::Pick => match self.selection_pick {
                FilterState::Include => true,
                FilterState::Exclude => false,
            },
        }
    }
}

#[derive(Debug, Default, Clone)]
pub enum SortField {
    #[default]
    CaptureDate,
}

#[derive(Debug, Default, Clone)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Default, Clone)]
pub struct SortOrder {
    sort_field: SortField,
    sort_direction: SortDirection,
}

impl SortOrder {
    fn sort(&self, left: &PictureObject, right: &PictureObject) -> std::cmp::Ordering {
        let cmp = match self.sort_field {
            SortField::CaptureDate => match (left.capture_time(), right.capture_time()) {
                (Some(l), Some(r)) => l.cmp(&r),
                (None, Some(_)) => std::cmp::Ordering::Less,
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            },
        };

        match self.sort_direction {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        }
    }
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        Object::builder().property("application", app).build()
    }

    fn thumbnails(&self) -> ListStore {
        self.imp().thumbnails.borrow().clone()
    }

    fn directories(&self) -> ListStore {
        self.imp().directories.borrow().clone()
    }

    #[tracing::instrument(name = "Updating window display path", skip(self))]
    pub fn set_path(&self, path: String) {
        let (tx, mut rx) = oneshot::channel();
        self.runtime().block_on(async move {
            let results = query_directory_pictures(self.database(), path)
                .await
                .unwrap();
            tx.send(results).unwrap();
        });
        let directories: Vec<PictureData> = rx.try_recv().unwrap();
        let mut model = ListStore::new(PictureObject::static_type());
        model.extend(directories.into_iter().map(PictureObject::from));

        self.imp().thumbnails.replace(model);
        self.init_selection_model();
    }

    #[tracing::instrument(name = "Initialising selection model.", skip(self))]
    fn init_selection_model(&self) {
        let filter_model = FilterListModel::new(Some(self.thumbnails()), Some(self.filter()));
        self.filter().connect_changed(
            clone!(@weak self as window, @weak filter_model => move |_, _| {
                filter_model.set_filter(Some(&window.filter()));
            }),
        );

        let sort_model = SortListModel::new(Some(filter_model), Some(self.sorter()));

        // TODO: Bind model to the thumbnails
        let selection_model = SingleSelection::builder().model(&sort_model).build();

        // Provide the updating of a picture in a single location.
        // TODO: Find a way to link these two properties together
        fn update_picture(window: &Window, picture: Option<PictureObject>) {
            window.imp().preview.unbind();
            if let Some(ref p) = picture {
                window.imp().preview.bind(p);
            }
            window.imp().preview_image.replace(picture);
        }

        selection_model.connect_selected_item_notify(clone!(@weak self as window => move |item| {
            let picture = item
                .selected_item()
                .map(|i| i.downcast::<PictureObject>().expect("Require a `PictureObject`"));
            update_picture(&window, picture);
        }));

        self.imp().thumbnail_list.set_model(Some(&selection_model));

        self.sorter().connect_changed(
            clone!(@weak self as window, @weak sort_model => move |_, _| {
                sort_model.set_sorter(Some(&window.sorter()));
            }),
        );
    }

    fn runtime(&self) -> &Runtime {
        self.imp().runtime.get().unwrap()
    }

    fn database(&self) -> &SqlitePool {
        self.imp().database.get().unwrap()
    }

    #[tracing::instrument(name = "Retrieving existing pictures within the database", skip(self))]
    fn get_existing_paths(&self, directory: &Utf8Path) -> HashSet<Utf8PathBuf> {
        let (tx, mut rx) = oneshot::channel();

        self.runtime().block_on(async move {
            let existing_pictures = query_existing_pictures(self.database(), directory.to_string())
                .await
                .unwrap();

            tx.send(existing_pictures).unwrap();
        });

        let pictures = rx.try_recv().unwrap();

        HashSet::from_iter(pictures.into_iter())
    }

    #[tracing::instrument(name = "Adding pictures from directory", skip(self))]
    fn add_pictures_from(&self, directory: &Utf8Path) {
        let existing_pictures = self.get_existing_paths(directory);

        tracing::info!(
            "Found {} existing files within directory",
            existing_pictures.len()
        );

        let images: Vec<_> = map_directory_images(directory)
            .into_iter()
            .filter(|p| !existing_pictures.contains(&p.filepath))
            .collect();

        if images.is_empty() {
            tracing::info!("No new images found in directory {directory}");
            return;
        } else {
            tracing::info!("Adding {} new images to the database.", images.len());
        }

        self.runtime()
            .block_on(async move { add_new_images(self.database(), images).await.unwrap() });

        // We need to update the list of directories
        self.update_directory_list();
    }

    #[tracing::instrument(name = "Creating Import dialog", skip(self))]
    fn import_dialog(&self) {
        let dialog = FileChooserDialog::new(
            Some("Choose Directory for Import"),
            Some(self),
            FileChooserAction::SelectFolder,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Import", ResponseType::Accept),
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
            window.import_new_files(&directory);
            window.update_directory_list()
        } ));

        dialog.present();
    }

    #[tracing::instrument(name = "Importing pictures from directory", skip(self))]
    fn import_new_files(&self, directory: &Utf8Path) {
        self.runtime()
            .block_on(async move { import(self.database(), directory).await.unwrap() })
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
            window.update_directory_list()
        }));
        dialog.present();
    }

    fn init_callbacks(&self) {
        // Setup callback when items of collections change
        self.set_stack();
        self.directories().connect_items_changed(
            clone!(@weak self as window => move |_, _, _, _| {
                window.set_stack();
            }),
        );
    }

    fn set_stack(&self) {
        if self.directories().n_items() > 0 {
            self.imp().stack.set_visible_child_name("main");
        } else {
            self.imp().stack.set_visible_child_name("placeholder");
        }
    }

    fn preview_image(&self) -> PictureObject {
        self.imp().preview_image.borrow().clone().unwrap()
    }

    fn filter_settings(&self) -> FilterSettings {
        self.imp().filter_state.borrow().clone()
    }

    fn filter(&self) -> CustomFilter {
        // Define Filters
        let filter_settings = self.filter_settings();
        CustomFilter::new(move |obj| {
            let picture_object = obj
                .downcast_ref::<PictureObject>()
                .expect("The object needs to be of type `PictureObject`");

            filter_settings.filter(&picture_object)
        })
    }

    fn sort_settings(&self) -> SortOrder {
        self.imp().sort_state.borrow().clone()
    }

    fn sorter(&self) -> CustomSorter {
        let sort_settings = self.sort_settings();
        // Define Filters
        CustomSorter::new(move |left, right| {
            let left = left
                .downcast_ref::<PictureObject>()
                .expect("The object needs to be of type `PictureObject`");

            let right = right
                .downcast_ref::<PictureObject>()
                .expect("The object needs to be of type `PictureObject`");

            sort_settings.sort(left, right).into()
        })
    }

    #[tracing::instrument(name = "Setting up Actions.", skip(self))]
    fn setup_actions(&self) {
        // Create action to create new collection and add to action group "win"
        let action_new_directory = SimpleAction::new("new-directory", None);
        action_new_directory.connect_activate(clone!(@weak self as window => move |_, _| {
            window.new_directory();
            // We have potentially added a new directory, so we need to update
            // the list of all the directories.
            window.update_directory_list();
        }));
        self.add_action(&action_new_directory);

        let action_import = SimpleAction::new("import-directory", None);
        action_import.connect_activate(clone!(@weak self as window => move |_, _| {
            window.import_dialog();
        }));
        self.add_action(&action_import);

        let action_image_select =
            SimpleAction::new("image-select", Some(&Selection::static_variant_type()));

        action_image_select.connect_activate(clone!(@weak self as window => move |_, v| {
            let _span = tracing::span!(Level::INFO, "Updating image selection").entered();
            if let Some(value) = v {
            // We need to set these values to help the borrow checker with move
            // in the closure. We are borrowing different items from window
            // so this is fine, just need the finer control in this instance.
            let preview = window.preview_image();
            let db = window.database();

            tracing::debug!("Setting to value {}", &value.to_string());
            // Set the value within the frontend
            preview.set_property("selection", value.to_string());
            // Update the database with the new status
            window.runtime().block_on(async move {
                update_selection_state(db, preview.id(), Selection::from_variant(value).unwrap()).await.unwrap();
            });

            }
        }));
        self.add_action(&action_image_select);
    }

    #[tracing::instrument(name = "Initialising thumbnail factory.", skip(self))]
    fn init_factory(&self) {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let thumbnail = PictureThumbnail::new();
            list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&thumbnail));
        });

        factory.connect_bind(move |_, list_item| {
            let picture_object = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .item()
                .and_downcast::<PictureObject>()
                .expect("The item has to be an `PictureObject`.");

            let image_thumbnail = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .child()
                .and_downcast::<PictureThumbnail>()
                .expect("The child has to be a `PictureThumbnail`.");

            image_thumbnail.bind(&picture_object);
        });

        factory.connect_unbind(move |_, list_item| {
            let image_thumbnail = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .child()
                .and_downcast::<PictureThumbnail>()
                .expect("The child has to be a `PictureThumbnail`.");

            image_thumbnail.unbind();
        });

        self.imp().thumbnail_list.set_factory(Some(&factory));
    }

    #[tracing::instrument(name = "Updating directory list Model", skip(self))]
    fn update_directory_list(&self) {
        let (tx, mut rx) = oneshot::channel();
        self.runtime().block_on(async move {
            let results = query_unique_directories(self.database()).await.unwrap();
            tx.send(results).unwrap();
        });
        let directories: Vec<String> = rx.try_recv().unwrap();

        let mut list_model = ListStore::new(StringObject::static_type());
        list_model.extend(
            directories
                .into_iter()
                .map(|s: String| StringObject::new(&s)),
        );

        self.imp().directories.replace(list_model);
        // This adds the new directories to the user interface, allowing
        // them to be selected.
        self.init_tree_selection_model();
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

            list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&label));

            list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
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

        if selection_model.n_items() > 0 {
            selection_model.select_item(0, true);
            tracing::debug!("Path has been selected");
            self.set_path(
                selection_model
                    .selected_item()
                    .unwrap()
                    .downcast::<StringObject>()
                    .unwrap()
                    .property::<String>("string"),
            );
        }
    }
}

mod imp {

    use std::cell::RefCell;
    use std::fs::File;

    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use gio::ListStore;
    use glib::subclass::InitializingObject;
    use gtk::{gio, glib, CompositeTemplate, ListView, ScrolledWindow, Stack};
    use once_cell::sync::OnceCell;
    use sqlx::SqlitePool;
    use tokio::runtime::{Builder as RuntimeBuilder, Runtime};
    use tokio::sync::oneshot;

    use super::{FilterSettings, SortOrder};
    use crate::picture::{PictureData, PictureObject, PicturePreview};

    #[derive(CompositeTemplate)]
    #[template(resource = "/resources/decimator.ui")]
    pub struct Window {
        // Provides the capability to run asynchronous tasks. We are using a
        // separate tokio runtime which does seem to make things simpler,
        // however, TODO use the internal glib runtime.
        // The sqlx integration requires a Tokio runtime to work, so this is
        // required for the database access.
        pub runtime: OnceCell<Runtime>,
        // Provide the connection pool for the database. This allows multiple
        // threads access.
        pub database: OnceCell<SqlitePool>,

        pub filter_state: RefCell<FilterSettings>,

        pub sort_state: RefCell<SortOrder>,

        #[template_child]
        pub stack: TemplateChild<Stack>,

        pub directories: RefCell<ListStore>,
        #[template_child]
        pub filetree: TemplateChild<ListView>,

        pub thumbnails: RefCell<ListStore>,
        #[template_child]
        pub thumbnail_list: TemplateChild<ListView>,
        #[template_child]
        pub thumbnail_scroll: TemplateChild<ScrolledWindow>,

        pub preview_image: RefCell<Option<PictureObject>>,
        #[template_child]
        pub preview: TemplateChild<PicturePreview>,
    }

    impl Default for Window {
        fn default() -> Self {
            let runtime = OnceCell::with_value(
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

            runtime.get().unwrap().block_on(async move {
                let pool = SqlitePool::connect(&database_path)
                    .await
                    .expect("Unable to initialise sqlite database");
                tx.send(pool).unwrap();
            });
            let database = OnceCell::with_value(rx.try_recv().unwrap());

            Self {
                thumbnail_scroll: Default::default(),
                filter_state: Default::default(),
                sort_state: Default::default(),
                preview: Default::default(),
                preview_image: Default::default(),
                thumbnail_list: Default::default(),
                filetree: Default::default(),
                thumbnails: Default::default(),
                directories: Default::default(),
                stack: Default::default(),
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
            obj.update_directory_list();
            obj.init_tree_selection_model();
            obj.init_tree_model();
            obj.setup_actions();
            obj.init_callbacks();
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
