use adw::prelude::*;
use camino::Utf8PathBuf;
use data::{query_directory_pictures, query_unique_directories};
use gtk::gdk::Texture;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;
use relm4::component::{AsyncComponent, AsyncComponentParts};
use relm4::factory::AsyncFactoryVecDeque;
use relm4::loading_widgets::LoadingWidgets;
use relm4::prelude::*;
use relm4::{view, AsyncComponentSender};
use sqlx::SqlitePool;

mod data;
mod directory;
mod import;
// mod menu;
mod picture;
mod telemetry;
// mod window;

use directory::Directory;
use picture::PictureThumbnail;
use telemetry::{get_subscriber, init_subscriber};

const APP_ID: &str = "com.malramsay.Decimator";

#[derive(Debug)]
pub enum AppMsg {
    UpdateDirectories,
    SelectDirectory(DynamicIndex),
    SelectPreview(Option<i32>),
}

#[derive(Debug)]
struct App {
    database: SqlitePool,
    directories: AsyncFactoryVecDeque<Directory>,
    thumbnails: AsyncFactoryVecDeque<PictureThumbnail>,
    preview_image: Option<Texture>,
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = String;
    type Input = AppMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
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
                        #[wrap(Some)]
                        set_title_widget = &adw::WindowTitle {
                            set_title: "Directories",
                        }
                    },
                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        set_width_request: 325,
                        #[local_ref]
                        directory_list -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 5
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
                            set_title: "Content"
                        }
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        gtk::Box {
                            set_vexpand: true,
                            set_hexpand: true,
                            gtk::Picture {
                                #[watch]
                                set_paintable: model.preview_image.as_ref(),

                            }
                        },
                        gtk::ScrolledWindow {
                            set_propagate_natural_width: true,
                            set_has_frame: true,

                            #[local_ref]
                            thumbnail_grid -> gtk::ListBox {
                                set_width_request: 260,
                                set_show_separators: true,
                                set_selection_mode: gtk::SelectionMode::Single,

                                connect_row_selected[sender] => move |_, row| {
                                    let index = row.map(|r| r.index());
                                    println!("{index:?}");
                                    sender.input(AppMsg::SelectPreview(index));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // menu! {
    //     main_menu:
    // }

    fn init_loading_widgets(root: &mut Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local_ref]
            root {
                set_title: Some("Decimator Relm Demo"),
                set_default_size: (300, 100),

                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_halign: gtk::Align::Center,
                }
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    #[tracing::instrument(name = "Initialising App", skip(root, sender))]
    async fn init(
        database_path: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let database = SqlitePool::connect(&database_path)
            .await
            .expect("Unable to initialise sqlite database");

        let mut directories = AsyncFactoryVecDeque::new(gtk::Box::default(), sender.input_sender());
        let thumbnails = AsyncFactoryVecDeque::new(gtk::ListBox::default(), sender.input_sender());

        {
            let mut directory_guard = directories.guard();
            let directory_vec = query_unique_directories(&database).await.unwrap();
            for dir in directory_vec {
                directory_guard.push_back(Utf8PathBuf::from(dir));
            }
        }

        let model = App {
            database,
            directories,
            thumbnails,
            preview_image: None,
        };
        let directory_list = model.directories.widget();
        let thumbnail_grid = model.thumbnails.widget();

        let widgets = view_output!();

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
                println!("{:?}", &directories);
                directory_guard.clear();
                for dir in directories {
                    directory_guard.push_back(Utf8PathBuf::from(dir));
                }
            }
            AppMsg::SelectDirectory(index) => {
                let directory = self.directories.get(index.current_index()).unwrap();
                let pictures =
                    query_directory_pictures(&self.database, directory.path.clone().into())
                        .await
                        .unwrap();
                let mut thumbnail_guard = self.thumbnails.guard();
                thumbnail_guard.clear();
                for pic in pictures {
                    thumbnail_guard.push_back(pic);
                }
            }
            AppMsg::SelectPreview(index) => {
                self.preview_image =
                    if let Some(pic) = index.and_then(|i| self.thumbnails.get(i as usize)) {
                        let filepath = pic.picture.filepath.clone();
                        Some(
                            relm4::spawn(async move {
                                let image = Pixbuf::from_file(filepath)
                                    .expect("Image not found.")
                                    .apply_embedded_orientation()
                                    .expect("Unable to apply orientation.");
                                Texture::for_pixbuf(&image)
                            })
                            .await
                            .unwrap(),
                        )
                    } else {
                        None
                    }
            }
        }
    }
}

fn main() {
    let subscriber = get_subscriber(APP_ID.into(), "debug".into(), std::io::stdout);
    init_subscriber(subscriber);
    let app = RelmApp::new(APP_ID);
    let mut path = glib::user_data_dir();
    path.push(crate::APP_ID);
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    let database_path = format!("sqlite://{}/database.db?mode=rwc", path.display());
    app.run_async::<App>(database_path)
}
