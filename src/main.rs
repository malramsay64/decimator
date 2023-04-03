#![feature(let_chains)]
use std::convert::identity;

use adw::prelude::*;
use camino::Utf8PathBuf;
use data::{query_directory_pictures, query_unique_directories};
use gtk::glib;
use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::component::{
    AsyncComponent, AsyncComponentController, AsyncComponentParts, AsyncController,
};
use relm4::factory::AsyncFactoryVecDeque;
use relm4::prelude::*;
use relm4::AsyncComponentSender;
use sqlx::SqlitePool;

mod data;
mod directory;
mod import;
// mod menu;
mod picture;
mod telemetry;
// mod window;

use directory::Directory;
use picture::{PictureView, PictureViewMsg};
use telemetry::{get_subscriber, init_subscriber};

const APP_ID: &str = "com.malramsay.Decimator";

#[derive(Debug)]
pub enum AppMsg {
    UpdateDirectories,
    SelectDirectory(Option<i32>),
    FilterTogglePick,
    FilterToggleOrdinary,
    FilterToggleIgnore,
    SelectionPick,
    SelectionOrdinary,
    SelectionIgnore,
}

#[derive(Debug)]
struct App {
    database: SqlitePool,
    directories: AsyncFactoryVecDeque<Directory>,
    picture_view: AsyncController<PictureView>,
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
                        pack_start = &gtk::Box {
                            gtk::Button {
                                set_label: "Add Directory",
                            },
                            gtk::Button {
                                set_label: "Import",
                            }
                        },
                    },
                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        set_width_request: 325,
                        #[local_ref]
                        directory_list -> gtk::ListBox {
                            set_selection_mode: gtk::SelectionMode::Single,

                            connect_row_selected[sender] => move |_, row| {
                                let index = row.map(|r| r.index());
                                // println!("{index:?}");
                                sender.input(AppMsg::SelectDirectory(index));
                            }
                        }
                    }
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

    // menu! {
    //     main_menu:
    // }

    #[tracing::instrument(name = "Initialising App", skip(root, sender))]
    async fn init(
        database_path: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let database = SqlitePool::connect(&database_path)
            .await
            .expect("Unable to initialise sqlite database");

        let mut directories =
            AsyncFactoryVecDeque::new(gtk::ListBox::default(), sender.input_sender());

        {
            let mut directory_guard = directories.guard();
            let directory_vec = query_unique_directories(&database).await.unwrap();
            for dir in directory_vec {
                directory_guard.push_back(Utf8PathBuf::from(dir));
            }
        }
        let picture_view = PictureView::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let model = App {
            database,
            directories,
            picture_view,
        };
        let directory_list = model.directories.widget();

        let widgets = view_output!();

        let app = relm4::main_application();

        app.set_accelerators_for_action::<PickAction>(&["p"]);
        app.set_accelerators_for_action::<OrdinaryAction>(&["o"]);
        app.set_accelerators_for_action::<IgnoreAction>(&["i"]);

        {
            let group = RelmActionGroup::<WindowActionGroup>::new();

            let sender_pick = sender.clone();
            let action_pick: RelmAction<PickAction> = {
                RelmAction::new_stateless(move |_| {
                    sender_pick.input(AppMsg::SelectionPick);
                })
            };
            let sender_ordinary = sender.clone();
            let action_ordinary: RelmAction<OrdinaryAction> = {
                RelmAction::new_stateless(move |_| {
                    sender_ordinary.input(AppMsg::SelectionOrdinary);
                })
            };
            let sender_ignore = sender.clone();
            let action_ignore: RelmAction<IgnoreAction> = {
                RelmAction::new_stateless(move |_| {
                    sender_ignore.input(AppMsg::SelectionIgnore);
                })
            };

            let sender_filter_pick = sender.clone();
            let action_filter_pick: RelmAction<PickFilterAction> = {
                RelmAction::new_stateless(move |_| {
                    sender_filter_pick.input(AppMsg::FilterTogglePick);
                })
            };
            let sender_filter_ordinary = sender.clone();
            let action_filter_ordinary: RelmAction<OrdinaryFilterAction> = {
                RelmAction::new_stateless(move |_| {
                    sender_filter_ordinary.input(AppMsg::FilterToggleOrdinary);
                })
            };
            let sender_filter_ignore = sender.clone();
            let action_filter_ignore: RelmAction<IgnoreFilterAction> = {
                RelmAction::new_stateless(move |_| {
                    sender_filter_ignore.input(AppMsg::FilterToggleIgnore);
                })
            };

            group.add_action(&action_filter_pick);
            group.add_action(&action_filter_ordinary);
            group.add_action(&action_filter_ignore);
            group.add_action(&action_pick);
            group.add_action(&action_ordinary);
            group.add_action(&action_ignore);

            let actions = group.into_action_group();

            widgets
                .main_window
                .insert_action_group("win", Some(&actions));
        }

        widgets
            .flap_status
            .bind_property("active", &widgets.flap, "reveal-flap")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        AsyncComponentParts { model, widgets }
    }

    #[tracing::instrument(name = "Updating App", level = "debug", skip(self, _sender, _root))]
    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AppMsg::UpdateDirectories => {
                let mut directory_guard = self.directories.guard();
                let directories = query_unique_directories(&self.database).await.unwrap();
                directory_guard.clear();
                for dir in directories {
                    directory_guard.push_back(Utf8PathBuf::from(dir));
                }
            }
            AppMsg::SelectDirectory(index) => {
                let pictures =
                    if let Some(directory) = index.and_then(|i| self.directories.get(i as usize)) {
                        query_directory_pictures(&self.database, directory.path.clone().into())
                            .await
                            .unwrap()
                    } else {
                        vec![]
                    };
                self.picture_view
                    .emit(PictureViewMsg::SelectPictures(pictures))
            }
            AppMsg::FilterTogglePick => self.picture_view.emit(PictureViewMsg::FilterTogglePick),
            AppMsg::FilterToggleOrdinary => {
                self.picture_view.emit(PictureViewMsg::FilterToggleOrdinary)
            }
            AppMsg::FilterToggleIgnore => {
                self.picture_view.emit(PictureViewMsg::FilterToggleIgnore)
            }
            AppMsg::SelectionPick => self.picture_view.emit(PictureViewMsg::SelectionPick),
            AppMsg::SelectionOrdinary => self.picture_view.emit(PictureViewMsg::SelectionOrdinary),
            AppMsg::SelectionIgnore => self.picture_view.emit(PictureViewMsg::SelectionIgnore),
        }
    }
}

relm4::new_action_group!(WindowActionGroup, "win");

relm4::new_stateless_action!(NextAction, WindowActionGroup, "next");
relm4::new_stateless_action!(PreviousAction, WindowActionGroup, "previous");

relm4::new_stateless_action!(PickAction, WindowActionGroup, "pick");
relm4::new_stateless_action!(OrdinaryAction, WindowActionGroup, "ordinary");
relm4::new_stateless_action!(IgnoreAction, WindowActionGroup, "ignore");

relm4::new_stateless_action!(PickFilterAction, WindowActionGroup, "pick_filter");
relm4::new_stateless_action!(OrdinaryFilterAction, WindowActionGroup, "ordinary_filter");
relm4::new_stateless_action!(IgnoreFilterAction, WindowActionGroup, "ignore_filter");

fn main() {
    // Configure tracing information
    let subscriber = get_subscriber(APP_ID.into(), "debug".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Set up the database we are running from
    let mut path = glib::user_data_dir();
    path.push(crate::APP_ID);
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    let database_path = format!("sqlite://{}/database.db?mode=rwc", path.display());

    // Starting the Relm Application Service
    let app = RelmApp::new(APP_ID);
    app.run_async::<App>(database_path)
}
