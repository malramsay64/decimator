use camino::Utf8PathBuf;
use data::{query_directory_pictures, query_unique_directories};
use gtk::glib;
use gtk::prelude::*;
use relm4::component::{AsyncComponent, AsyncComponentParts};
use relm4::factory::AsyncFactoryVecDeque;
use relm4::loading_widgets::LoadingWidgets;
use relm4::prelude::*;
use relm4::{gtk, view, AsyncComponentSender};
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
}

struct App {
    database: SqlitePool,
    directories: AsyncFactoryVecDeque<Directory>,
    thumbnails: AsyncFactoryVecDeque<PictureThumbnail>,
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = String;
    type Input = AppMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        gtk::Window {
            set_default_size: (960, 540),
            gtk::Box{
                set_vexpand: true,
                set_hexpand: true,
                gtk::ScrolledWindow {
                    set_width_request: 325,
                    #[local_ref]
                    directory_list -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5
                    }
                },
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_has_frame: true,

                    #[local_ref]
                    thumbnail_grid -> gtk::Grid {
                        set_vexpand: true,
                        set_column_spacing: 5,
                        set_row_spacing: 5,
                    }
                }
            }
        }
    }

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
        let thumbnails = AsyncFactoryVecDeque::new(gtk::Grid::default(), sender.input_sender());

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
        };
        let directory_list = model.directories.widget();
        let thumbnail_grid = model.thumbnails.widget();

        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

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
