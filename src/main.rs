use camino::Utf8PathBuf;
use data::{
    query_directory_pictures, query_unique_directories, update_selection_state, update_thumbnails,
};
use iced::widget::image::Handle;
use iced::widget::{self, button, column, container, row, text};
use iced::{Application, Command, Element, Length, Settings, Theme};
use image::RgbaImage;
use import::find_new_images;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use uuid::Uuid;

use crate::import::import;
use crate::picture::ZoomStates;

mod data;
mod directory;
mod import;
mod picture;
mod telemetry;

use directory::DirectoryData;
use picture::{PictureThumbnail, Selection};
use telemetry::{get_subscriber_terminal, init_subscriber};

const APP_ID: &str = "com.malramsay.Decimator";

#[derive(Debug, Clone)]
pub enum AppMsg {
    Initialise(DatabaseConnection),
    DirectoryAddRequest,
    DirectoryAdd(Utf8PathBuf),
    DirectoryImportRequest,
    DirectoryImport(Utf8PathBuf),
    QueryDirectories,
    UpdateDirectories(Vec<DirectoryData>),
    UpdateThumbnails(bool),
    SetThumbnails(Vec<PictureThumbnail>),
    SelectDirectory(Utf8PathBuf),
    DisplayPick(bool),
    DisplayOrdinary(bool),
    DisplayIgnore(bool),
    DisplayHidden(bool),
    SetSelection((Uuid, Selection)),
    // Signal to emit when we want to export, this creates the export dialog
    SelectionExportRequest,
    // Contains the path where the files are being exported to
    SelectionExport(Utf8PathBuf),
    SelectionPrintRequest,
    SelectionZoom(ZoomStates),
    UpdatePictureView(Option<RgbaImage>),
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
    // TODO: Use a reference counted preview so this doesn't need to
    // keep being cloned throughout
    preview_image: Option<RgbaImage>,
}

impl AppData {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            directories: vec![],
            thumbnails: vec![],
            preview_image: None,
        }
    }

    fn directory_view(&self) -> Element<AppMsg> {
        column(
            self.directories
                .clone()
                .into_iter()
                .map(|item| {
                    button(text(item.directory.clone().into_string()))
                        .on_press(AppMsg::SelectDirectory(item.directory))
                        .into()
                })
                .collect(),
        )
        .into()
    }

    fn thumbnail_view(&self) -> Element<AppMsg> {
        row(self
            .thumbnails
            .clone()
            .into_iter()
            .map(|image| {
                if let Some(thumbnail) = &image.thumbnail {
                    button(iced::widget::image::viewer(Handle::from_pixels(
                        thumbnail.width(),
                        thumbnail.height(),
                        thumbnail.clone().into_vec(),
                    )))
                    .on_press(AppMsg::UpdatePictureView(image.thumbnail.clone()))
                    .into()
                } else {
                    text("No Thumbnail").into()
                }
            })
            .collect())
        .into()
    }

    fn preview_view(&self) -> Element<AppMsg> {
        if let Some(image) = &self.preview_image {
            iced::widget::image::viewer(iced::widget::image::Handle::from_pixels(
                image.width(),
                image.height(),
                image.to_vec(),
            ))
            .into()
        } else {
            text("No image available").into()
        }
    }
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
            Command::perform(
                async move {
                    let mut connection_options = ConnectOptions::new(database_path);
                    // The minimum number of connections is rather important. There are cases within the application where
                    // we have multiple connections open simultaneously to handle the streaming of data from the database
                    // while performing operations on the data. This doesn't work if we don't increase the minimum number
                    // of connections resulting in a lock on the connections.
                    connection_options.max_connections(20).min_connections(4);
                    tracing::debug!("Connection Options: {:?}", connection_options);
                    Database::connect(connection_options)
                        .await
                        .expect("Unable to initialise sqlite database")
                },
                AppMsg::Initialise,
            ),
        )
    }

    #[tracing::instrument(name = "Updating App", level = "info", skip(self))]
    fn update(&mut self, msg: Self::Message) -> Command<AppMsg> {
        tracing::info!("{:?}", &msg);
        match self {
            Self::Uninitialised => match msg {
                AppMsg::Initialise(database) => {
                    *self = Self::Initialised(AppData::new(database));
                    tracing::debug!("Updated app");
                    Command::perform(async {}, |_| AppMsg::QueryDirectories)
                }
                _ => panic!("App needs to be initialised"),
            },
            Self::Initialised(inner) => {
                let database = inner.database.clone();
                match msg {
                    AppMsg::Initialise(_) => panic!("App is already initialised"),
                    AppMsg::DirectoryImportRequest => Command::none(),
                    AppMsg::DirectoryImport(dir) => Command::perform(
                        async move { import(&database, &dir).await.unwrap() },
                        |_| AppMsg::QueryDirectories,
                    ),
                    AppMsg::DirectoryAddRequest => Command::none(),
                    AppMsg::DirectoryAdd(dir) => Command::perform(
                        async move {
                            find_new_images(&database, &dir).await;
                        },
                        |_| AppMsg::QueryDirectories,
                    ),
                    AppMsg::QueryDirectories => Command::perform(
                        async move { query_unique_directories(&database).await.unwrap() },
                        |dirs| AppMsg::UpdateDirectories(dirs),
                    ),
                    AppMsg::UpdateDirectories(dirs) => {
                        inner.directories = dirs;
                        Command::none()
                    }
                    AppMsg::UpdateThumbnails(all) => Command::perform(
                        async move {
                            // TODO: Add a dialog confirmation box
                            update_thumbnails(&database, all)
                                .await
                                .expect("Unable to update thumbnails");
                        },
                        |_| AppMsg::Ignore,
                    ),
                    AppMsg::SelectDirectory(dir) => Command::perform(
                        async move {
                            query_directory_pictures(&database, dir.into())
                                .await
                                .unwrap()
                        },
                        AppMsg::SetThumbnails,
                    ),
                    AppMsg::SetThumbnails(thumbnails) => {
                        inner.thumbnails = thumbnails;
                        Command::none()
                    }
                    AppMsg::DisplayPick(value) => Command::none(),
                    AppMsg::DisplayOrdinary(value) => Command::none(),
                    AppMsg::DisplayIgnore(value) => Command::none(),
                    AppMsg::DisplayHidden(value) => Command::none(),
                    AppMsg::SetSelection((id, s)) => Command::perform(
                        async move { update_selection_state(&database, id, s).await.unwrap() },
                        |_| AppMsg::Ignore,
                    ),
                    AppMsg::SelectionExportRequest => Command::none(),
                    AppMsg::SelectionExport(dir) => Command::none(),
                    AppMsg::SelectionPrintRequest => Command::none(),
                    AppMsg::Ignore => Command::none(),
                    AppMsg::ThumbnailNext => widget::focus_next(),
                    AppMsg::ThumbnailPrev => widget::focus_previous(),
                    AppMsg::UpdatePictureView(view) => {
                        inner.preview_image = view;
                        Command::none()
                    }
                    AppMsg::SelectionZoom(scale) => Command::none(),
                }
            }
        }
    }

    fn view(&self) -> Element<AppMsg> {
        let content: Element<AppMsg> = match self {
            Self::Uninitialised => column![text("Loading...")].into(),
            Self::Initialised(inner) => row![
                inner.directory_view(),
                column![
                    text("Application"),
                    inner.preview_view(),
                    inner.thumbnail_view()
                ]
            ]
            .into(),
        };
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
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
    dbg!(&database_path);

    App::run(Settings {
        flags: database_path,
        ..Default::default()
    });
}
