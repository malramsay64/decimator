#![feature(let_chains)]
use std::convert::identity;

use adw::prelude::*;
use camino::Utf8PathBuf;
use data::{query_directory_pictures, query_unique_directories, update_thumbnails};
use gtk::glib;
use import::find_new_images;
use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup, *};
use relm4::component::{
    AsyncComponent, AsyncComponentController, AsyncComponentParts, AsyncController,
};
use relm4::prelude::*;
use relm4::typed_list_view::TypedListView;
use relm4::AsyncComponentSender;
use relm4_components::open_dialog::*;
use sea_orm::{Database, DatabaseConnection};

use crate::import::import;

mod data;
mod directory;
mod import;
// mod menu;
mod picture;
mod telemetry;
// mod window;

use directory::DirectoryData;
use picture::{PictureView, PictureViewMsg};
use telemetry::{get_subscriber_terminal, init_subscriber};

const APP_ID: &str = "com.malramsay.Decimator";

#[derive(Debug)]
pub enum AppMsg {
    DirectoryAddRequest,
    DirectoryAdd(Utf8PathBuf),
    DirectoryImportRequest,
    DirectoryImport(Utf8PathBuf),
    UpdateDirectories,
    UpdateThumbnailsAll,
    UpdateThumbnailsNew,
    SelectDirectories(Vec<u32>),
    FilterPick(bool),
    FilterOrdinary(bool),
    FilterIgnore(bool),
    FilterHidden(bool),
    SelectionPick,
    SelectionOrdinary,
    SelectionIgnore,
    SelectionExportRequest,
    SelectionExport(Utf8PathBuf),
    ThumbnailNext,
    ThumbnailPrev,
    Ignore,
}

#[derive(Debug)]
struct App {
    database: DatabaseConnection,
    directories: TypedListView<DirectoryData, gtk::MultiSelection>,
    picture_view: AsyncController<PictureView>,
    dialog_import: Controller<OpenDialog>,
    dialog_add: Controller<OpenDialog>,
    dialog_export: Controller<OpenDialog>,
    progress: gtk::ProgressBar,
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = String;
    type Input = AppMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        #[name = "main_window"]
        adw::Window {
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
                    #[local_ref]
                    progress -> gtk::ProgressBar { }
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
                            set_menu_model: Some(&main_menu),
                        },

                        #[wrap(Some)]
                        set_title_widget = &adw::WindowTitle {
                            set_title: "Decimator"
                        }
                    },
                    model.picture_view.widget(),
                 }
            }
        }
    }

    menu! {
        main_menu: {
            "Export" => ActionExport,
            section! {
                "Generate New Thumbnails" => ActionUpdateThumbnailNew,
                "Update All Thumbnails" => ActionUpdateThumbnailAll,
            },
            section! {
                "Hide Picked" => ActionFilterPick,
                "Hide Ordinary" => ActionFilterOrdinary,
                "Hide Ignored" => ActionFilterIgnore,
            },
            section!{
                "Hidden Images" => ActionFilterHidden,
            }
        }
    }

    #[tracing::instrument(name = "Initialising App", skip(root, sender))]
    async fn init(
        database_path: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let database = Database::connect(&database_path)
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

        let picture_view = PictureView::builder()
            .launch(database.clone())
            .forward(sender.input_sender(), identity);

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

        let progress = gtk::ProgressBar::new();

        let model = App {
            database,
            directories,
            picture_view,
            dialog_import,
            dialog_add,
            dialog_export,
            progress: progress.clone(),
        };
        let directory_list = &model.directories.view;

        let widgets = view_output!();

        let app = relm4::main_application();

        app.set_accelerators_for_action::<ActionPrev>(&["h"]);
        app.set_accelerators_for_action::<ActionNext>(&["l"]);

        app.set_accelerators_for_action::<ActionPick>(&["p"]);
        app.set_accelerators_for_action::<ActionOrdinary>(&["o"]);
        app.set_accelerators_for_action::<ActionIgnore>(&["i"]);

        {
            // TODO: Write a send message action macro and a toggle state action macro

            let mut group = RelmActionGroup::<WindowActionGroup>::new();

            let sender_update_thumbnail_all = sender.clone();
            let action_update_thumbnail_all: RelmAction<ActionUpdateThumbnailAll> = {
                RelmAction::new_stateless(move |_| {
                    sender_update_thumbnail_all.input(AppMsg::UpdateThumbnailsAll);
                })
            };
            let sender_update_thumbnail_new = sender.clone();
            let action_update_thumbnail_new: RelmAction<ActionUpdateThumbnailNew> = {
                RelmAction::new_stateless(move |_| {
                    sender_update_thumbnail_new.input(AppMsg::UpdateThumbnailsNew);
                })
            };

            let sender_pick = sender.clone();
            let action_pick: RelmAction<ActionPick> = {
                RelmAction::new_stateless(move |_| {
                    sender_pick.input(AppMsg::SelectionPick);
                })
            };
            let sender_ordinary = sender.clone();
            let action_ordinary: RelmAction<ActionOrdinary> = {
                RelmAction::new_stateless(move |_| {
                    sender_ordinary.input(AppMsg::SelectionOrdinary);
                })
            };
            let sender_ignore = sender.clone();
            let action_ignore: RelmAction<ActionIgnore> = {
                RelmAction::new_stateless(move |_| {
                    sender_ignore.input(AppMsg::SelectionIgnore);
                })
            };
            let sender_export = sender.clone();
            let action_export: RelmAction<ActionExport> = {
                RelmAction::new_stateless(move |_| {
                    sender_export.input(AppMsg::SelectionExportRequest);
                })
            };

            let sender_next = sender.clone();
            let action_next: RelmAction<ActionNext> = {
                RelmAction::new_stateless(move |_| {
                    sender_next.input(AppMsg::ThumbnailNext);
                })
            };
            let sender_prev = sender.clone();
            let action_prev: RelmAction<ActionPrev> = {
                RelmAction::new_stateless(move |_| {
                    sender_prev.input(AppMsg::ThumbnailPrev);
                })
            };

            let sender_filter_pick = sender.clone();
            let action_filter_pick: RelmAction<ActionFilterPick> = {
                RelmAction::new_stateful(&false, move |_, state: &mut bool| {
                    *state = !*state;
                    sender_filter_pick.input(AppMsg::FilterPick(*state));
                })
            };
            let sender_filter_ordinary = sender.clone();
            let action_filter_ordinary: RelmAction<ActionFilterOrdinary> = {
                RelmAction::new_stateful(&false, move |_, state: &mut bool| {
                    *state = !*state;
                    sender_filter_ordinary.input(AppMsg::FilterOrdinary(*state));
                })
            };
            let sender_filter_ignore = sender.clone();
            let action_filter_ignore: RelmAction<ActionFilterIgnore> = {
                RelmAction::new_stateful(&false, move |_, state: &mut bool| {
                    *state = !*state;
                    sender_filter_ignore.input(AppMsg::FilterIgnore(*state));
                })
            };
            let sender_filter_hidden = sender.clone();
            let action_filter_hidden: RelmAction<ActionFilterHidden> = {
                RelmAction::new_stateful(&true, move |_, state: &mut bool| {
                    *state = !*state;
                    sender_filter_hidden.input(AppMsg::FilterHidden(*state));
                })
            };

            group.add_action(action_update_thumbnail_all);
            group.add_action(action_update_thumbnail_new);
            group.add_action(action_filter_pick);
            group.add_action(action_filter_ordinary);
            group.add_action(action_filter_ignore);
            group.add_action(action_pick);
            group.add_action(action_ordinary);
            group.add_action(action_ignore);
            group.add_action(action_export);
            group.add_action(action_next);
            group.add_action(action_prev);
            group.add_action(action_filter_hidden);

            let actions = group.into_action_group();

            widgets
                .main_window
                .insert_action_group(WindowActionGroup::NAME, Some(&actions));
        }

        // Get all the directories from the database
        sender.input(AppMsg::UpdateDirectories);
        widgets
            .flap_status
            .bind_property("active", &widgets.flap, "reveal-flap")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        AsyncComponentParts { model, widgets }
    }

    #[tracing::instrument(name = "Updating App", level = "debug", skip(self, sender, _root))]
    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AppMsg::DirectoryImportRequest => self.dialog_import.emit(OpenDialogMsg::Open),
            AppMsg::DirectoryImport(dir) => {
                // let db = self.database.clone();
                import(&self.database, &dir, &self.progress).await.unwrap();
                // relm4::spawn(async move { import(&db, &dir).await })
                //     .await
                //     .unwrap();
                sender.input(AppMsg::UpdateDirectories);
            }
            AppMsg::DirectoryAddRequest => self.dialog_add.emit(OpenDialogMsg::Open),
            AppMsg::DirectoryAdd(dir) => {
                // let db = self.database.clone();
                find_new_images(&self.database, &dir).await;
                // relm4::spawn(async move { find_new_images(&db, &dir).await })
                //     .await
                //     .unwrap();
                sender.input(AppMsg::UpdateDirectories);
            }
            AppMsg::UpdateDirectories => {
                let directories = query_unique_directories(&self.database).await.unwrap();
                self.directories.clear();
                self.directories.extend_from_iter(directories.into_iter());
            }
            AppMsg::UpdateThumbnailsAll => {
                // TODO: Add a dialog confirmation box
                let db = self.database.clone();
                relm4::spawn(async move {
                    update_thumbnails(&db, true)
                        .await
                        .expect("Unable to update thumbnails");
                })
                .await
                .unwrap();
            }
            AppMsg::UpdateThumbnailsNew => {
                let db = self.database.clone();
                relm4::spawn(async move {
                    update_thumbnails(&db, false)
                        .await
                        .expect("Unable to update thumbnails");
                })
                .await
                .unwrap();
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
                self.picture_view
                    .emit(PictureViewMsg::SelectPictures(pictures))
            }
            AppMsg::FilterPick(value) => self.picture_view.emit(PictureViewMsg::FilterPick(value)),
            AppMsg::FilterOrdinary(value) => self
                .picture_view
                .emit(PictureViewMsg::FilterOrdinary(value)),
            AppMsg::FilterIgnore(value) => {
                self.picture_view.emit(PictureViewMsg::FilterIgnore(value))
            }
            AppMsg::FilterHidden(value) => {
                self.picture_view.emit(PictureViewMsg::FilterHidden(value))
            }
            AppMsg::SelectionPick => self.picture_view.emit(PictureViewMsg::SelectionPick),
            AppMsg::SelectionOrdinary => self.picture_view.emit(PictureViewMsg::SelectionOrdinary),
            AppMsg::SelectionIgnore => self.picture_view.emit(PictureViewMsg::SelectionIgnore),
            AppMsg::SelectionExportRequest => self.dialog_export.emit(OpenDialogMsg::Open),
            AppMsg::SelectionExport(dir) => {
                self.picture_view.emit(PictureViewMsg::SelectionExport(dir))
            }
            AppMsg::ThumbnailNext => self.picture_view.emit(PictureViewMsg::ImageNext),
            AppMsg::ThumbnailPrev => self.picture_view.emit(PictureViewMsg::ImagePrev),
            AppMsg::Ignore => {}
        }
    }
}

relm4::new_action_group!(WindowActionGroup, "win");

relm4::new_stateless_action!(
    ActionUpdateThumbnailAll,
    WindowActionGroup,
    "update_thumbnails_all"
);

relm4::new_stateless_action!(
    ActionUpdateThumbnailNew,
    WindowActionGroup,
    "update_thumbnails_new"
);

relm4::new_stateless_action!(ActionNext, WindowActionGroup, "next");
relm4::new_stateless_action!(ActionPrev, WindowActionGroup, "previous");

relm4::new_stateless_action!(ActionPick, WindowActionGroup, "pick");
relm4::new_stateless_action!(ActionOrdinary, WindowActionGroup, "ordinary");
relm4::new_stateless_action!(ActionIgnore, WindowActionGroup, "ignore");
relm4::new_stateless_action!(ActionExport, WindowActionGroup, "export");

relm4::new_stateful_action!(ActionFilterPick, WindowActionGroup, "pick_filter", (), bool);
relm4::new_stateful_action!(
    ActionFilterOrdinary,
    WindowActionGroup,
    "ordinary_filter",
    (),
    bool
);
relm4::new_stateful_action!(
    ActionFilterIgnore,
    WindowActionGroup,
    "ignore_filter",
    (),
    bool
);
relm4::new_stateful_action!(
    ActionFilterHidden,
    WindowActionGroup,
    "hidden_filter",
    (),
    bool
);

fn main() {
    // Configure tracing information
    let subscriber = get_subscriber_terminal(APP_ID.into(), "info".into(), std::io::stdout);
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
    app.run_async::<App>(database_path)
}
