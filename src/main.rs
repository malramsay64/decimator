use std::convert::identity;

use adw::prelude::*;
use camino::Utf8PathBuf;
use data::{query_directory_pictures, query_unique_directories, update_thumbnails};
use gtk::{gio, glib};
use import::find_new_images;
use relm4::component::{
    AsyncComponent, AsyncComponentController, AsyncComponentParts, AsyncController,
};
use relm4::prelude::*;
use relm4::typed_list_view::TypedListView;
use relm4::AsyncComponentSender;
use relm4_components::open_dialog::*;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use crate::import::import;
use crate::picture::ViewGrid;

mod data;
mod directory;
mod import;
mod relm_ext;
// mod menu;
mod picture;
mod telemetry;
// mod window;

use directory::DirectoryData;
use picture::{Selection, ViewGridMsg, ViewPreview, ViewPreviewMsg};
use telemetry::{get_subscriber_terminal, init_subscriber};

const APP_ID: &str = "com.malramsay.Decimator";

use relm4::safe_settings_and_actions::extensions::*;

relm4::safe_settings_and_actions! {
    #[derive(Debug)]
    @value(param: &'a str, map: <str>)
    UpdateThumbnails(group:"win", name: "update-thumbnails") {
        All = ("All"),
        New = ("New"),
    }

    Export(group: "win", name: "export");
    Print(group: "win", name: "print");

    // Interactions with the pictures

    Next(group: "win", name: "next");
    Previous(group: "win", name: "previous");

    @value(param: u32)
    @state(param: u32, owned: u32)
    Zoom(group: "win", name: "previous");

    #[derive(Debug)]
    @value(param: &'a str, map: <str>)
    SetSelection(group: "win", name: "set-selection") {
        Pick = ("pick"),
        Ordinary = ("ordinary"),
        Ignore = ("ignore"),
    }

    @state(param: bool, owned: bool)
    DisplayPick(group: "win", name: "filter_pick");

    @state(param: bool, owned: bool)
    DisplayOrdinary(group: "win", name: "filter_ordinary");

    @state(param: bool, owned: bool)
    DisplayIgnore(group: "win", name: "filter_ignore");

    @state(param: bool, owned: bool)
    DisplayHidden(group: "win", name: "filter_hidden");
}

#[derive(Debug)]
pub enum AppMsg {
    DirectoryAddRequest,
    DirectoryAdd(Utf8PathBuf),
    DirectoryImportRequest,
    DirectoryImport(Utf8PathBuf),
    UpdateDirectories,
    UpdateThumbnails(bool),
    SelectDirectories(Vec<u32>),
    DisplayPick(bool),
    DisplayOrdinary(bool),
    DisplayIgnore(bool),
    DisplayHidden(bool),
    SetSelection(Selection),
    // Signal to emit when we want to export, this creates the export dialog
    SelectionExportRequest,
    // Contains the path where the files are being exported to
    SelectionExport(Utf8PathBuf),
    SelectionPrintRequest,
    SelectionZoom(Option<u32>),
    UpdatePictureView(PictureView),
    ThumbnailNext,
    ThumbnailPrev,
    Ignore,
}

#[derive(Debug, Default)]
pub enum PictureView {
    #[default]
    Preview,
    Grid,
}

impl TryFrom<&str> for PictureView {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Preview" => Ok(Self::Preview),
            "Grid" => Ok(Self::Grid),
            _ => Err(anyhow::anyhow!(
                "Unable to convert pictureview from {value}"
            )),
        }
    }
}

#[derive(Debug)]
struct App {
    database: DatabaseConnection,
    directories: TypedListView<DirectoryData, gtk::MultiSelection>,
    picture_view: PictureView,
    view_preview: AsyncController<ViewPreview>,
    view_grid: AsyncController<ViewGrid>,
    dialog_import: Controller<OpenDialog>,
    dialog_add: Controller<OpenDialog>,
    dialog_export: Controller<OpenDialog>,
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = String;
    type Input = AppMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        #[root]
        #[name = "main_window"]
        adw::ApplicationWindow {
            set_default_size: (960, 540),
            #[name = "flap"]
            adw::Flap {
                set_vexpand: true,

                #[wrap(Some)]
                set_flap = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    #[name = "sidebar_header"]
                    adw::HeaderBar {
                        set_show_end_title_buttons: false,
                        set_show_start_title_buttons: false,
                        #[wrap(Some)]
                        set_title_widget = &adw::WindowTitle {
                            set_title: ""
                        },
                        pack_start = &gtk::Button {
                                set_label: "Add Directory",
                                connect_clicked => AppMsg::DirectoryAddRequest,
                        },
                        pack_end = &gtk::Button {
                                set_label: "Import",
                                connect_clicked => AppMsg::DirectoryImportRequest,
                        },
                    },
                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        set_width_request: 325,
                        #[local_ref]
                        directory_list -> gtk::ListView {}
                    },
                    // #[local_ref]
                    // progress_bars -> gtk::Box{ }
                },
                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_vexpand: true,
                    #[name = "content_header"]
                    adw::HeaderBar {
                        #[name = "flap_status"]
                        pack_start = &gtk::ToggleButton {
                            set_icon_name: "sidebar-show-symbolic",
                            set_active: true,
                        },
                        pack_end = &gtk::MenuButton {
                            set_icon_name: "open-menu-symbolic",
                            #[wrap(Some)]
                            set_menu_model = &gio::Menu {
                                action["Export"]: Export,
                                section["Thumbnails"] = &gio::Menu {
                                    action["Create New"]: UpdateThumbnails::New,
                                    action["Update All"]: UpdateThumbnails::All,
                                },
                                section["Filters"] = &gio::Menu {
                                    action["Picked"]: DisplayPick,
                                    action["Ordinary"]: DisplayOrdinary,
                                    action["Ignore"]: DisplayIgnore,
                                },
                                section["Hidden Images"] = &gio::Menu {
                                    action["Hidden"]: DisplayHidden,
                                },
                            },
                        },

                        #[wrap(Some)]
                        set_title_widget = &adw::WindowTitle {
                            set_title: "Decimator"
                        }
                    },

                    #[name = "picture_view"]
                    adw::ViewStack {
                        connect_visible_child_notify[sender] => move |stack| {
                            sender.input(AppMsg::UpdatePictureView(stack.visible_child_name().unwrap().as_str().try_into().unwrap()));
                        }
                    },

                    adw::ViewSwitcherBar{
                        set_stack: Some(&picture_view),
                        set_reveal: true,
                    }
                 }
            },

            add_action = &gio::SimpleAction::new_safe::<Export>() {
                connect_activate_safe[sender] => move |Export, _| sender.input(AppMsg::SelectionExportRequest),
            },
            add_action = &gio::SimpleAction::new_safe::<UpdateThumbnails>() {
                connect_activate_safe_enum[sender] => move |_, value| sender.input(AppMsg::UpdateThumbnails(match value {
                    UpdateThumbnails::New => false,
                    UpdateThumbnails::All => true,
                })),
            },
            add_action = &gio::SimpleAction::new_safe::<SetSelection>() {
                connect_activate_safe_enum[sender] =>
                    move | _, value| sender.input(AppMsg::SetSelection(match value {
                    SetSelection::Pick => Selection::Pick,
                    SetSelection::Ordinary => Selection::Ordinary,
                    SetSelection::Ignore => Selection::Ignore,
                })),
            },
            add_action = &gio::SimpleAction::new_stateful_safe::<DisplayPick>(true) {
                connect_activate_safe_with_mut_state[sender] =>
                    move |DisplayPick, _, state| {
                        *state = !*state;
                        sender.input(AppMsg::DisplayPick(*state));
                },
            },
            add_action = &gio::SimpleAction::new_stateful_safe::<DisplayOrdinary>(true) {
                connect_activate_safe_with_mut_state[sender] =>
                    move |DisplayOrdinary, _, state| {
                        *state = !*state;
                        sender.input(AppMsg::DisplayOrdinary(*state));

                },
            },
            add_action = &gio::SimpleAction::new_stateful_safe::<DisplayIgnore>(true) {
                connect_activate_safe_with_mut_state[sender] =>
                    move |DisplayIgnore, _, state| {
                        *state = !*state;
                        sender.input(AppMsg::DisplayIgnore(*state));
                },
            },
            add_action = &gio::SimpleAction::new_stateful_safe::<DisplayHidden>(false) {
                connect_activate_safe_with_mut_state[sender] =>
                    move |DisplayHidden, _, state| {
                        *state = !*state;
                        sender.input(AppMsg::DisplayHidden(*state));
                },
            },

        },

    }

    #[tracing::instrument(name = "Initialising App", skip(root, sender))]
    async fn init(
        database_path: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut connection_options = ConnectOptions::new(database_path);
        // The minimum number of connections is rather important. There are cases within the application where
        // we have multiple connections open simultaneously to handle the streaming of data from the database
        // while performing operations on the data. This doesn't work if we don't increase the minimum number
        // of connections resulting in a lock on the connections.
        connection_options.max_connections(20).min_connections(4);
        tracing::debug!("Connection Options: {:?}", connection_options);
        let database = Database::connect(connection_options)
            .await
            .expect("Unable to initialise sqlite database");

        let directories: TypedListView<DirectoryData, gtk::MultiSelection> =
            TypedListView::with_sorting();

        let directory_sender = sender.clone();
        directories.selection_model.connect_selection_changed(
            move |selection, _position, _n_items| {
                let mut indicies = vec![];
                let bitvec = selection.selection();
                if let Some((iter, value)) = gtk::BitsetIter::init_first(&bitvec) {
                    indicies.push(value);
                    for value in iter {
                        indicies.push(value);
                    }
                    dbg!(&indicies);
                    directory_sender.input(AppMsg::SelectDirectories(indicies))
                }
            },
        );

        let dialog_settings = OpenDialogSettings {
            folder_mode: true,
            create_folders: false,
            accept_label: String::from("Import"),
            ..Default::default()
        };

        let dialog_import = OpenDialog::builder()
            .transient_for_native(&root)
            .launch(dialog_settings)
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => {
                    AppMsg::DirectoryImport(path.try_into().unwrap())
                }
                OpenDialogResponse::Cancel => AppMsg::Ignore,
            });

        let dialog_settings = OpenDialogSettings {
            folder_mode: true,
            create_folders: false,
            accept_label: String::from("Add"),
            ..Default::default()
        };

        let dialog_add = OpenDialog::builder()
            .transient_for_native(&root)
            .launch(dialog_settings)
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => AppMsg::DirectoryAdd(path.try_into().unwrap()),
                OpenDialogResponse::Cancel => AppMsg::Ignore,
            });

        let dialog_settings = OpenDialogSettings {
            folder_mode: true,
            create_folders: true,
            accept_label: String::from("Export"),
            ..Default::default()
        };

        let dialog_export = OpenDialog::builder()
            .transient_for_native(&root)
            .launch(dialog_settings)
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => {
                    AppMsg::SelectionExport(path.try_into().unwrap())
                }
                OpenDialogResponse::Cancel => AppMsg::Ignore,
            });

        let view_preview = ViewPreview::builder()
            .launch(database.clone())
            .forward(sender.input_sender(), identity);

        let view_grid = ViewGrid::builder()
            .launch(database.clone())
            .forward(sender.input_sender(), identity);

        // let progress_bars = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let model = App {
            database,
            directories,
            picture_view: PictureView::default(),
            view_preview,
            view_grid,
            dialog_import,
            dialog_add,
            dialog_export,
            // progress_bars,
        };
        let directory_list = &model.directories.view;

        let widgets = view_output!();

        widgets
            .picture_view
            .add_titled(model.view_preview.widget(), Some("Preview"), "Preview")
            .set_icon_name(Some("window-scrolling"));
        widgets
            .picture_view
            .add_titled(model.view_grid.widget(), Some("Grid"), "Grid")
            .set_icon_name(Some("grid-large"));

        let app = relm4::main_application();

        app.set_accels_for_action_safe(Print, &["<Ctrl>P"]);

        app.set_accels_for_action_safe(Next, &["h"]);
        app.set_accels_for_action_safe(Previous, &["l"]);

        app.set_accels_for_action_safe(SetSelection::Pick, &["p"]);
        app.set_accels_for_action_safe(SetSelection::Ordinary, &["o"]);
        app.set_accels_for_action_safe(SetSelection::Ignore, &["i"]);

        // Get all the directories from the database
        sender.input(AppMsg::UpdateDirectories);
        widgets
            .flap_status
            .bind_property("active", &widgets.flap, "reveal-flap")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        AsyncComponentParts { model, widgets }
    }

    #[tracing::instrument(name = "Updating App", level = "info", skip(self, sender, root))]
    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        tracing::info!("{:?}", &msg);
        match msg {
            AppMsg::DirectoryImportRequest => self.dialog_import.emit(OpenDialogMsg::Open),
            AppMsg::DirectoryImport(dir) => {
                import(&self.database, &dir).await.unwrap();
                sender.input(AppMsg::UpdateDirectories);
            }
            AppMsg::DirectoryAddRequest => self.dialog_add.emit(OpenDialogMsg::Open),
            AppMsg::DirectoryAdd(dir) => {
                find_new_images(&self.database, &dir).await;
                sender.input(AppMsg::UpdateDirectories);
            }
            AppMsg::UpdateDirectories => {
                let directories = query_unique_directories(&self.database).await.unwrap();
                self.directories.clear();
                self.directories.extend_from_iter(directories.into_iter());
            }
            AppMsg::UpdateThumbnails(all) => {
                // TODO: Add a dialog confirmation box
                update_thumbnails(&self.database, all)
                    .await
                    .expect("Unable to update thumbnails");
            }
            AppMsg::SelectDirectories(indicies) => {
                let directories: Vec<String> = indicies
                    .into_iter()
                    .map(|i| {
                        self.directories
                            .get_visible(i)
                            .unwrap()
                            .borrow()
                            .directory
                            .to_string()
                    })
                    .collect();
                let pictures = query_directory_pictures(&self.database, &directories)
                    .await
                    .unwrap();
                self.view_preview
                    .emit(ViewPreviewMsg::SelectPictures(pictures.clone()));
                self.view_grid.emit(ViewGridMsg::SelectPictures(pictures));
            }
            AppMsg::DisplayPick(value) => {
                self.view_preview.emit(ViewPreviewMsg::DisplayPick(value));
                self.view_grid.emit(ViewGridMsg::DisplayPick(value));
            }
            AppMsg::DisplayOrdinary(value) => {
                self.view_preview
                    .emit(ViewPreviewMsg::DisplayOrdinary(value));
                self.view_grid.emit(ViewGridMsg::DisplayOrdinary(value));
            }
            AppMsg::DisplayIgnore(value) => {
                self.view_preview.emit(ViewPreviewMsg::DisplayIgnore(value));
                self.view_grid.emit(ViewGridMsg::DisplayIgnore(value));
            }
            AppMsg::DisplayHidden(value) => {
                self.view_preview.emit(ViewPreviewMsg::DisplayHidden(value));
                self.view_grid.emit(ViewGridMsg::DisplayHidden(value));
            }
            AppMsg::SetSelection(s) => match self.picture_view {
                PictureView::Preview => self.view_preview.emit(ViewPreviewMsg::SetSelection(s)),
                PictureView::Grid => self.view_grid.emit(ViewGridMsg::SetSelection(s)),
            },
            AppMsg::SelectionExportRequest => self.dialog_export.emit(OpenDialogMsg::Open),
            AppMsg::SelectionExport(dir) => match self.picture_view {
                PictureView::Preview => {
                    self.view_preview.emit(ViewPreviewMsg::SelectionExport(dir))
                }
                PictureView::Grid => self.view_grid.emit(ViewGridMsg::SelectionExport(dir)),
            },
            AppMsg::SelectionPrintRequest => self.view_preview.emit(
                ViewPreviewMsg::SelectionPrint(root.clone().upcast::<gtk::Window>()),
            ),
            AppMsg::Ignore => {}
            AppMsg::ThumbnailNext => self.view_preview.emit(ViewPreviewMsg::ImageNext),
            AppMsg::ThumbnailPrev => self.view_preview.emit(ViewPreviewMsg::ImagePrev),
            AppMsg::UpdatePictureView(view) => {
                self.picture_view = view;
            }
            AppMsg::SelectionZoom(scale) => {
                self.view_preview.emit(ViewPreviewMsg::SelectionZoom(scale))
            }
        }
    }
}

fn main() {
    // Configure tracing information
    let subscriber = get_subscriber_terminal(APP_ID.into(), "debug".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Set up the database we are running from
    let mut path = glib::user_data_dir();
    path.push(crate::APP_ID);
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    let database_path = format!("sqlite://{}/database.db?mode=rwc", path.display());
    relm4::RELM_THREADS.set(2).unwrap();
    relm4::RELM_BLOCKING_THREADS
        .set(num_cpus::get_physical())
        .unwrap();

    // Starting the Relm Application Service
    let app = RelmApp::new(APP_ID);
    relm4_icons::initialize_icons();
    app.run_async::<App>(database_path)
}
