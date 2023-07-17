use camino::Utf8PathBuf;
use data::{query_directory_pictures, query_unique_directories, update_thumbnails};
use iced::widget::{self, Image};
use iced::{Application, Command, Element, Renderer, Settings, Theme};
use import::find_new_images;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use crate::import::import;
use crate::picture::ZoomStates;

mod data;
mod directory;
mod import;
mod schema;
// mod relm_ext;
// mod menu;
mod picture;
mod telemetry;
// mod window;

use directory::DirectoryData;
use picture::{PictureThumbnail, Selection};
use telemetry::{get_subscriber_terminal, init_subscriber};

const APP_ID: &str = "com.malramsay.Decimator";

#[derive(Debug)]
pub enum AppMsg {
    Initialise(String),
    DirectoryAddRequest,
    DirectoryAdd(Utf8PathBuf),
    DirectoryImportRequest,
    DirectoryImport(Utf8PathBuf),
    UpdateDirectories,
    UpdateThumbnails(bool),
    SelectDirectories(Utf8PathBuf),
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
    SelectionZoom(ZoomStates),
    UpdatePictureView(Option<Image>),
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
struct AppData {
    database: DatabaseConnection,
    directories: Vec<DirectoryData>,
    thumbnails: Vec<PictureThumbnail>,
    preview_image: Option<Image>,
}

#[derive(Debug, Default)]
enum App {
    #[default]
    Uninitialised,
    Initialised(AppData),
}

impl Application for App {
    type Flags = String;
    type Message = AppMsg;
    type Theme = Theme;
    type Executor = iced::executor::Default;

    #[tracing::instrument(name = "Initialising App")]
    fn new(database_path: Self::Flags) -> (Self, Command<AppMsg>) {
        (
            Self::default(),
            Self {
                database,
                directories: vec![],
                thumbnails: vec![],
                preview_image: None,
            },
            Command::perform(async move {}, AppMsg::Initialise(database_path)),
        )
    }

    #[tracing::instrument(name = "Updating App", level = "info", skip(self))]
    fn update(&mut self, msg: Self::Message) -> Command<AppMsg> {
        tracing::info!("{:?}", &msg);
        match msg {
            AppMsg::Initialise(database_path) => {
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
                *self = Self::Initialised(AppData {
                    database,
                    directories: vec![],
                    thumbnails: vec![],
                    preview_image: None,
                });
                Command::none()
            }
            AppMsg::DirectoryImportRequest => Command::none(),
            AppMsg::DirectoryImport(dir) => Command::perform(
                async move { import(&self.database, &dir).await.unwrap() },
                |_| AppMsg::UpdateDirectories,
            ),
            AppMsg::DirectoryAddRequest => Command::none(),
            AppMsg::DirectoryAdd(dir) => Command::perform(
                async move {
                    find_new_images(&self.database, &dir).await;
                },
                |_| AppMsg::UpdateDirectories,
            ),
            AppMsg::UpdateDirectories => Command::perform(
                async move {
                    self.directories = query_unique_directories(&self.database).await.unwrap();
                },
                |_| AppMsg::Ignore,
            ),
            AppMsg::UpdateThumbnails(all) => Command::perform(
                async move {
                    // TODO: Add a dialog confirmation box
                    update_thumbnails(&self.database, all)
                        .await
                        .expect("Unable to update thumbnails");
                },
                |_| AppMsg::Ignore,
            ),
            AppMsg::SelectDirectories(dir) => Command::perform(
                async move {
                    let pictures = query_directory_pictures(&self.database, &vec![dir.into()])
                        .await
                        .unwrap();
                },
                |pics| AppMsg::Ignore,
            ),
            AppMsg::DisplayPick(value) => Command::none(),
            AppMsg::DisplayOrdinary(value) => Command::none(),
            AppMsg::DisplayIgnore(value) => Command::none(),
            AppMsg::DisplayHidden(value) => Command::none(),
            AppMsg::SetSelection(s) => Command::none(),
            AppMsg::SelectionExportRequest => Command::none(),
            AppMsg::SelectionExport(dir) => Command::none(),
            AppMsg::SelectionPrintRequest => Command::none(),
            AppMsg::Ignore => Command::none(),
            AppMsg::ThumbnailNext => widget::focus_next(),
            AppMsg::ThumbnailPrev => widget::focus_previous(),
            AppMsg::UpdatePictureView(view) => {
                self.preview_image = view;
                Command::none()
            }
            AppMsg::SelectionZoom(scale) => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, AppMsg, Renderer<Theme>> {
        column![].into()
    }

    fn title(&self) -> String {
        String::from("Decimator")
    }
}

fn main() {
    // Configure tracing information
    let subscriber = get_subscriber_terminal(APP_ID.into(), "debug".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Set up the database we are running from
    let mut path = dirs::data_local_dir().expect("Unable to find local data dir");
    path.push(crate::APP_ID);
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    let database_path = format!("sqlite://{}/database.db?mode=rwc", path.display());

    App::run(Settings::default());
}
