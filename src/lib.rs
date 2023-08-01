use std::cell::RefCell;
use std::collections::HashMap;

use camino::Utf8PathBuf;
use data::{
    query_directory_pictures, query_unique_directories, update_selection_state, update_thumbnails,
};
use iced::keyboard::KeyCode;
use iced::widget::{
    button, checkbox, column, container, horizontal_space, lazy, row, scrollable, text,
};
use iced::{Application, Command, Element, Length, Theme};
use import::find_new_images;
use itertools::Itertools;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use uuid::Uuid;

use crate::import::import;
use crate::widget::viewer;

mod data;
mod directory;
mod import;
mod picture;
pub mod telemetry;
mod widget;
use directory::DirectoryData;
use picture::{PictureThumbnail, Selection};

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
    SetSelection(Selection),
    // Signal to emit when we want to export, this creates the export dialog
    SelectionExportRequest,
    // Contains the path where the files are being exported to
    SelectionExport(Utf8PathBuf),
    SelectionPrintRequest,
    UpdatePictureView(Option<Uuid>),
    ThumbnailNext,
    ThumbnailPrev,
    Ignore,
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

#[derive(Debug, Default, PartialEq, Eq)]
enum Order {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Default)]
struct ThumbnailView {
    thumbnails: HashMap<Uuid, PictureThumbnail>,
    filter: ThumbnailFilter,
    sort: Order,
    positions: Vec<Uuid>,
    version: u64,
}

impl ThumbnailView {
    /// Internal only function describing all the steps for making an update
    ///
    /// Whenever internal state changes, there are a number of additional steps
    /// required to maintain consistency. This performs all those steps, ensuring
    /// they are in a single location.
    fn update_inner(&mut self) {
        self.version += 1;
        self.positions = self
            .thumbnails
            .values()
            .filter(|t| self.filter.filter(t))
            .sorted()
            .map(|t| t.id)
            .collect();
        if self.sort == Order::Descending {
            self.positions.reverse()
        }
    }

    pub fn next(&mut self, id: Option<&Uuid>) -> Option<Uuid> {
        if let Some(id) = id {
            self.positions
                .iter()
                .position(|i| i == id)
                .map(|i| (i + 1).clamp(0, self.positions.len() - 1))
                .and_then(|i| self.positions.get(i))
                .copied()
        } else {
            None
        }
    }

    pub fn prev(&mut self, id: Option<&Uuid>) -> Option<Uuid> {
        if let Some(id) = id {
            self.positions
                .iter()
                .position(|i| i == id)
                .map(|i| (i - 1).clamp(0, self.positions.len() - 1))
                .and_then(|i| self.positions.get(i))
                .copied()
        } else {
            None
        }
    }

    pub fn pick(&self) -> bool {
        self.filter.pick
    }

    pub fn ordinary(&self) -> bool {
        self.filter.ordinary
    }

    pub fn ignore(&self) -> bool {
        self.filter.ignore
    }

    pub fn hidden(&self) -> bool {
        self.filter.hidden
    }

    pub fn set_thumbnails(&mut self, thumbnails: Vec<PictureThumbnail>) {
        self.thumbnails = thumbnails.into_iter().map(|t| (t.id, t)).collect();
        self.update_inner();
    }

    pub fn set_ignore(&mut self, value: bool) {
        self.filter.ignore = value;
        self.update_inner();
    }
    pub fn set_ordinary(&mut self, value: bool) {
        self.filter.ordinary = value;
        self.update_inner();
    }
    pub fn set_pick(&mut self, value: bool) {
        self.filter.pick = value;
        self.update_inner();
    }
    pub fn set_hidden(&mut self, value: bool) {
        self.filter.hidden = value;
        self.update_inner();
    }

    pub fn set_selection(&mut self, id: &Uuid, selection: Selection) {
        self.thumbnails.get_mut(id).unwrap().selection = selection;
        self.update_inner()
    }

    pub fn get_view(&self) -> Vec<PictureThumbnail> {
        self.positions
            .iter()
            .map(|i| self.thumbnails.get(i).unwrap())
            .cloned()
            .collect()
    }

    pub fn get_filepath(&self, id: &Uuid) -> Option<Utf8PathBuf> {
        self.thumbnails.get(id).map(|t| t.filepath.clone())
    }
}

#[derive(Debug)]
pub struct AppData {
    database: DatabaseConnection,
    directories: Vec<DirectoryData>,
    directory: Option<Utf8PathBuf>,
    thumbnail_view: ThumbnailView,

    preview: Option<Uuid>,
    preview_cache: RefCell<lru::LruCache<Uuid, iced::widget::image::Handle>>,
}

impl AppData {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            directories: vec![],
            directory: None,
            thumbnail_view: Default::default(),
            preview: None,
            preview_cache: RefCell::new(lru::LruCache::new(20.try_into().unwrap())),
        }
    }

    fn menu_view<'a>(&self) -> Element<AppMsg> {
        row!(
            horizontal_space(Length::Fill),
            button(text("Thumbnails")).on_press(AppMsg::UpdateThumbnails(true)),
            checkbox("Pick", self.thumbnail_view.pick(), AppMsg::DisplayPick),
            checkbox(
                "Ordinary",
                self.thumbnail_view.ordinary(),
                AppMsg::DisplayOrdinary
            ),
            checkbox(
                "Ignore",
                self.thumbnail_view.ignore(),
                AppMsg::DisplayIgnore
            ),
            checkbox(
                "Hidden",
                self.thumbnail_view.hidden(),
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
        let thumbnails = lazy(self.thumbnail_view.version, |_| {
            row(self
                .thumbnail_view
                .get_view()
                .into_iter()
                .map(PictureThumbnail::view)
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
        if let Some(id) = &self.preview {
            if let Some(filepath) = self.thumbnail_view.get_filepath(id) {
                let mut cache = self.preview_cache.borrow_mut();
                let handle = match cache.get(id) {
                    Some(h) => h,
                    None => {
                        let i = picture::load_image(filepath.as_path(), None).unwrap();
                        let handle = iced::widget::image::Handle::from_pixels(
                            i.width(),
                            i.height(),
                            i.to_vec(),
                        );
                        cache.put(*id, handle);
                        cache.get(id).unwrap()
                    }
                };
                return container(
                    viewer(handle.clone())
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into();
            }
        }
        container(text("No image available"))
            .height(Length::Fill)
            .width(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Debug, Default)]
pub enum App {
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
                        AppMsg::UpdateDirectories,
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
                        inner.thumbnail_view.set_thumbnails(thumbnails);
                        Command::none()
                    }
                    AppMsg::DisplayPick(value) => {
                        inner.thumbnail_view.set_pick(value);
                        Command::none()
                    }
                    AppMsg::DisplayOrdinary(value) => {
                        inner.thumbnail_view.set_ordinary(value);
                        Command::none()
                    }
                    AppMsg::DisplayIgnore(value) => {
                        inner.thumbnail_view.set_ignore(value);
                        Command::none()
                    }
                    AppMsg::DisplayHidden(value) => {
                        inner.thumbnail_view.set_hidden(value);
                        Command::none()
                    }
                    AppMsg::SetSelection(s) => {
                        if let Some(id) = inner.preview {
                            inner.thumbnail_view.set_selection(&id, s);
                            Command::perform(
                                async move { update_selection_state(&database, id, s).await.unwrap() },
                                |_| AppMsg::Ignore,
                            )
                        } else {
                            Command::none()
                        }
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
                        let items = inner.thumbnail_view.get_view();
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
                    AppMsg::ThumbnailNext => {
                        inner.preview = inner.thumbnail_view.next(inner.preview.as_ref());
                        Command::none()
                    }
                    AppMsg::ThumbnailPrev => {
                        inner.preview = inner.thumbnail_view.prev(inner.preview.as_ref());
                        Command::none()
                    }
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

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        match self {
            App::Uninitialised => iced::subscription::events_with(|_, _| None),
            App::Initialised(_) => iced::subscription::events_with(move |e, _| match e {
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: k, ..
                }) => match k {
                    KeyCode::H | KeyCode::Left => Some(AppMsg::ThumbnailPrev),
                    KeyCode::L | KeyCode::Right => Some(AppMsg::ThumbnailNext),
                    KeyCode::P => Some(AppMsg::SetSelection(Selection::Pick)),
                    KeyCode::O => Some(AppMsg::SetSelection(Selection::Ordinary)),
                    KeyCode::I => Some(AppMsg::SetSelection(Selection::Ignore)),
                    _ => None,
                },
                _ => None,
            }),
        }
    }

    fn title(&self) -> String {
        String::from("Decimator")
    }
}
