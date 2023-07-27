use std::collections::HashMap;

use camino::Utf8PathBuf;
use data::{
    query_directory_pictures, query_unique_directories, update_selection_state, update_thumbnails,
};
use iced::widget::image::Handle;
use iced::widget::{
    self, button, checkbox, column, container, horizontal_space, lazy, radio, row, scrollable, text,
};
use iced::{Application, Command, Element, Length, Settings, Theme};
use import::find_new_images;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use uuid::Uuid;

use crate::import::import;

mod data;
mod directory;
mod import;
mod picture;
mod telemetry;

use directory::DirectoryData;
use picture::{PictureThumbnail, Selection};
use telemetry::{get_subscriber_terminal, init_subscriber};

const APP_ID: &str = "com.malramsay.Decimator";

/// Messages for runnning the application
#[derive(Debug, Clone)]
pub enum AppMsg {
    /// Set up the application, this can only be done once within the application
    Initialise(DatabaseConnection),
    /// The request to open the directory selection menu
    DirectoryAddRequest,
    DirectoryAdd(Utf8PathBuf),
    /// The request to open the directory selection menu
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
    UpdatePictureView(Option<Uuid>),
    ThumbnailNext,
    ThumbnailPrev,
    UpdateLazy,
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

/// Provide the opportunity to filter thumbnails
///
/// Values are true when the filter is enabled and false
/// when they are disabled.
#[derive(Debug)]
struct ThumbnailFilter {
    ignore: bool,
    ordinary: bool,
    pick: bool,
    hidden: bool,
}

impl Default for ThumbnailFilter {
    fn default() -> Self {
        Self {
            ignore: true,
            ordinary: true,
            pick: true,
            hidden: false,
        }
    }
}

impl ThumbnailFilter {
    fn filter(&self, thumbnail: &PictureThumbnail) -> bool {
        let mut value = false;

        if self.ignore {
            value = value || thumbnail.selection == Selection::Ignore;
        }
        if self.ordinary {
            value = value || thumbnail.selection == Selection::Ordinary;
        }
        if self.pick {
            value = value || thumbnail.selection == Selection::Pick;
        }
        if self.hidden {
            value = value && !thumbnail.hidden
        }
        value
    }
}

#[derive(Debug)]
struct AppData {
    database: DatabaseConnection,
    directories: Vec<DirectoryData>,
    directory: Option<Utf8PathBuf>,
    thumbnails: HashMap<Uuid, PictureThumbnail>,
    /// The version of the thumbnails. This should be incremented
    /// any time the view of the thumbnails change
    version: u64,
    thumbnail_filter: ThumbnailFilter,

    // TODO: Use a reference counted preview so this doesn't need to
    // keep being cloned throughout
    // preview_image: Option<Utf8PathBuf>,
    preview: Option<Uuid>,
}

impl AppData {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            directories: vec![],
            directory: None,
            thumbnails: HashMap::new(),
            thumbnail_filter: Default::default(),
            // preview_image: None,
            preview: None,
            version: 0,
        }
    }

    fn menu_view<'a>(&self) -> Element<AppMsg> {
        row!(
            horizontal_space(Length::Fill),
            button(text("Thumbnails")).on_press(AppMsg::UpdateThumbnails(true)),
            checkbox("Pick", self.thumbnail_filter.pick, AppMsg::DisplayPick),
            checkbox(
                "Ordinary",
                self.thumbnail_filter.ordinary,
                AppMsg::DisplayOrdinary
            ),
            checkbox(
                "Ignore",
                self.thumbnail_filter.ignore,
                AppMsg::DisplayIgnore
            ),
            checkbox(
                "Hidden",
                self.thumbnail_filter.hidden,
                AppMsg::DisplayHidden
            ),
        )
        .spacing(10)
        .align_items(iced::Alignment::Center)
        .into()
    }

    fn directory_view(&self) -> Element<AppMsg> {
        scrollable(column![
            row![
                button(text("Add")).on_press(AppMsg::DirectoryAddRequest),
                horizontal_space(Length::Fill),
                button(text("Import")).on_press(AppMsg::DirectoryImportRequest),
            ],
            column(
                self.directories
                    .clone()
                    .into_iter()
                    .map(|item| {
                        button(text(item.directory.clone().into_string()))
                            .on_press(AppMsg::SelectDirectory(item.directory))
                            .width(240)
                            .into()
                    })
                    .collect(),
            )
        ])
        .width(240)
        .height(Length::Fill)
        .into()
    }

    fn thumbnail_view(&self) -> Element<AppMsg> {
        let thumbnails = lazy(self.version, |_| {
            let mut items: Vec<_> = self
                .thumbnails
                .values()
                .filter(|t| self.thumbnail_filter.filter(t))
                .cloned()
                .collect();
            items.sort();

            row(items
                .into_iter()
                .map(|image| {
                    if let Some(thumbnail) = &image.thumbnail {
                        button(column!(
                            container(iced::widget::image(Handle::from_pixels(
                                thumbnail.width(),
                                thumbnail.height(),
                                thumbnail.clone().into_vec(),
                            )))
                            .height(240)
                            .width(240)
                            .center_x()
                            .center_y(),
                            row![
                                radio("P", Selection::Pick, Some(image.selection), |s| {
                                    AppMsg::SetSelection((image.id, s))
                                }),
                                radio("O", Selection::Ordinary, Some(image.selection), |s| {
                                    AppMsg::SetSelection((image.id, s))
                                }),
                                radio("I", Selection::Ignore, Some(image.selection), |s| {
                                    AppMsg::SetSelection((image.id, s))
                                })
                            ]
                            .spacing(10)
                            .padding(20)
                        ))
                        .on_press(AppMsg::UpdatePictureView(Some(image.id)))
                        .into()
                    } else {
                        text("No Thumbnail").into()
                    }
                })
                .collect())
            .spacing(10)
        });

        container(scrollable(thumbnails).width(Length::Fill).direction(
            iced::widget::scrollable::Direction::Horizontal(
                iced::widget::scrollable::Properties::default(),
            ),
        ))
        .height(320)
        .width(Length::Fill)
        .into()
    }

    fn preview_view(&self) -> Element<AppMsg> {
        if let Some(image) = &self.preview {
            container(
                iced::widget::image::viewer(iced::widget::image::Handle::from_path(
                    self.thumbnails.get(image).unwrap().filepath.as_path(),
                ))
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else {
            container(text("No image available"))
                .height(Length::Fill)
                .width(Length::Fill)
                .center_x()
                .center_y()
                .into()
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
                    AppMsg::UpdateLazy => {
                        inner.version += 1;
                        Command::none()
                    }
                    AppMsg::Initialise(_) => panic!("App is already initialised"),
                    AppMsg::DirectoryImportRequest => Command::perform(
                        async move {
                            rfd::AsyncFileDialog::new()
                                .pick_folder()
                                .await
                                .expect("No Directory found")
                                .path()
                                .to_str()
                                .unwrap()
                                .into()
                        },
                        AppMsg::DirectoryImport,
                    ),
                    AppMsg::DirectoryImport(dir) => Command::perform(
                        async move { import(&database, &dir).await.unwrap() },
                        |_| AppMsg::QueryDirectories,
                    ),
                    AppMsg::DirectoryAddRequest => Command::perform(
                        async move {
                            rfd::AsyncFileDialog::new()
                                .pick_folder()
                                .await
                                .expect("No Directory found")
                                .path()
                                .to_str()
                                .unwrap()
                                .into()
                        },
                        AppMsg::DirectoryAdd,
                    ),
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
                            println!("Thumbnail Update");
                            // TODO: Add a dialog confirmation box
                            update_thumbnails(&database, all)
                                .await
                                .expect("Unable to update thumbnails");
                        },
                        |_| AppMsg::Ignore,
                    ),
                    AppMsg::SelectDirectory(dir) => {
                        inner.directory = Some(dir.clone());
                        Command::perform(
                            async move {
                                query_directory_pictures(&database, dir.into())
                                    .await
                                    .unwrap()
                            },
                            AppMsg::SetThumbnails,
                        )
                    }
                    AppMsg::SetThumbnails(thumbnails) => {
                        inner.thumbnails = thumbnails.into_iter().map(|t| (t.id, t)).collect();
                        Command::perform(async move {}, |_| AppMsg::UpdateLazy)
                    }
                    AppMsg::DisplayPick(value) => {
                        inner.thumbnail_filter.pick = value;
                        Command::perform(async move {}, |_| AppMsg::UpdateLazy)
                    }
                    AppMsg::DisplayOrdinary(value) => {
                        inner.thumbnail_filter.ordinary = value;
                        Command::perform(async move {}, |_| AppMsg::UpdateLazy)
                    }
                    AppMsg::DisplayIgnore(value) => {
                        inner.thumbnail_filter.ignore = value;
                        Command::perform(async move {}, |_| AppMsg::UpdateLazy)
                    }
                    AppMsg::DisplayHidden(value) => {
                        inner.thumbnail_filter.hidden = value;
                        Command::perform(async move {}, |_| AppMsg::UpdateLazy)
                    }
                    AppMsg::SetSelection((id, s)) => {
                        inner.thumbnails.get_mut(&id).unwrap().selection = s;
                        Command::perform(
                            async move { update_selection_state(&database, id, s).await.unwrap() },
                            |_| AppMsg::UpdateLazy,
                        )
                    }
                    AppMsg::SelectionExportRequest => Command::perform(
                        async move {
                            rfd::AsyncFileDialog::new()
                                .pick_folder()
                                .await
                                .expect("No Directory found")
                                .path()
                                .to_str()
                                .unwrap()
                                .into()
                        },
                        AppMsg::SelectionExport,
                    ),
                    AppMsg::SelectionExport(dir) => {
                        let items: Vec<_> = inner
                            .thumbnails
                            .values()
                            .filter(|t| inner.thumbnail_filter.filter(t))
                            .cloned()
                            .collect();
                        Command::perform(
                            async move {
                                for file in items.into_iter() {
                                    let origin = file.filepath;
                                    let destination = dir.join(origin.file_name().unwrap());

                                    tokio::fs::copy(origin, destination)
                                        .await
                                        .expect("Unable to copy image from {path}");
                                }
                            },
                            |_| AppMsg::Ignore,
                        )
                    }
                    AppMsg::SelectionPrintRequest => Command::none(),
                    AppMsg::Ignore => Command::none(),
                    AppMsg::ThumbnailNext => widget::focus_next(),
                    AppMsg::ThumbnailPrev => widget::focus_previous(),
                    AppMsg::UpdatePictureView(preview) => {
                        inner.preview = preview;
                        Command::none()
                    }
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
                    inner.menu_view(),
                    inner.preview_view(),
                    inner.thumbnail_view()
                ]
                .width(Length::Fill)
                .height(Length::Fill)
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

fn main() -> Result<(), iced::Error> {
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
    })
}
