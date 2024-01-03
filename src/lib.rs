use std::cell::RefCell;

use camino::Utf8PathBuf;
use data::{
    query_directory_pictures, query_unique_directories, update_selection_state, update_thumbnails,
};
use iced::keyboard::KeyCode;
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text};
use iced::{Application, Command, Element, Length, Theme};
use iced_aw::native::Grid;
use iced_widget::scrollable::{scroll_to, AbsoluteOffset, Id, Properties};
use import::find_new_images;
use itertools::Itertools;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use selection_list::SelectionList;
use uuid::Uuid;

use crate::import::import;
use crate::widget::viewer;

mod data;
mod directory;
mod import;
mod menu;
mod picture;
pub mod telemetry;
mod thumbnail;
mod widget;

use directory::DirectoryData;
use entity::Selection;
use picture::PictureThumbnail;
use thumbnail::ThumbnailData;

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
    ScrollTo(Uuid),
    SetSelection((Uuid, Selection)),
    SetSelectionCurrent(Selection),
    // Signal to emit when we want to export, this creates the export dialog
    SetView(AppView),
    SelectionExportRequest,
    // Contains the path where the files are being exported to
    SelectionExport(Utf8PathBuf),
    SelectionPrintRequest,
    UpdatePictureView(Option<Uuid>),
    ThumbnailNext,
    ThumbnailPrev,
    DirectoryNext,
    DirectoryPrev,
    Ignore,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    #[default]
    Preview,
    Grid,
}

#[derive(Debug)]
pub struct AppData {
    database: DatabaseConnection,
    directories: Vec<DirectoryData>,
    directory: Option<Utf8PathBuf>,

    app_view: AppView,
    thumbnail_view: ThumbnailData,
    thumbnail_scroller: Id,

    preview: Option<Uuid>,
    preview_cache: RefCell<lru::LruCache<Uuid, iced::widget::image::Handle>>,
}

impl AppData {
    fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            directories: vec![],
            directory: None,
            app_view: Default::default(),
            thumbnail_view: Default::default(),
            thumbnail_scroller: Id::unique(),
            preview: None,
            preview_cache: RefCell::new(lru::LruCache::new(20.try_into().unwrap())),
        }
    }

    fn menu_view(&self) -> Element<AppMsg> {
        menu::menu_view(self)
    }

    fn directory_view(&self) -> Element<AppMsg> {
        let dirs = self.directories.iter().sorted_unstable().rev();
        let values: Vec<_> = dirs.clone().zip(dirs.map(DirectoryData::view)).collect();
        column![
            row![
                button(text("Add")).on_press(AppMsg::DirectoryAddRequest),
                horizontal_space(20.),
                button(text("Import")).on_press(AppMsg::DirectoryImportRequest),
            ]
            .padding(10),
            SelectionList::new(values, |dir| {
                AppMsg::SelectDirectory(DirectoryData::add_prefix(&dir.directory))
            })
            .item_width(250.)
            .item_height(30.)
            .width(260.)
            .view()
        ]
        .width(Length::Shrink)
        .height(Length::Fill)
        .into()
    }

    fn thumbnail_view(&self) -> Element<AppMsg> {
        let view: Vec<_> = self
            .thumbnail_view
            .positions()
            .into_iter()
            .zip(
                self.thumbnail_view
                    .get_view()
                    .into_iter()
                    .map(PictureThumbnail::view),
            )
            .collect();

        SelectionList::new_with_selection(
            view,
            |i| AppMsg::UpdatePictureView(Some(i)),
            self.preview
                .map_or(Some(0), |id| self.thumbnail_view.get_position(&id)),
        )
        .direction(selection_list::Direction::Horizontal)
        .item_height(320.)
        .item_width(240.)
        .height(320.)
        .id(self.thumbnail_scroller.clone())
        .view()
    }

    /// Provides an overview of all the images on a grid
    fn grid_view(&self) -> Element<AppMsg> {
        let thumbnails = self
            .thumbnail_view
            .get_view()
            .into_iter()
            .map(PictureThumbnail::view)
            .fold(
                Grid::new()
                    .width(Length::Fill)
                    .strategy(iced_aw::Strategy::ColumnWidthFlex(260.)),
                |i, g| i.push(g),
            );
        scrollable(thumbnails)
            .direction(scrollable::Direction::Vertical(
                Properties::new().width(2.).scroller_width(10.),
            ))
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

// TODO: Remove the need to have an uninitialised state of the application
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
                    AppMsg::SetView(view) => {
                        inner.app_view = view;
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
                        AppMsg::UpdateDirectories,
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
                        // Default to selecting the first image within a directory
                        inner.preview = inner.thumbnail_view.positions().first().copied();
                        Command::none()
                    }
                    // Modify Thumbnail filters
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
                    // TODO: Implement ScrollTo action
                    AppMsg::ScrollTo(id) => {
                        let offset = inner.thumbnail_view.get_position(&id).unwrap() as f32 * 240.;
                        scroll_to(
                            inner.thumbnail_scroller.clone(),
                            AbsoluteOffset { x: offset, y: 0. },
                        )
                    }
                    AppMsg::SetSelectionCurrent(s) => {
                        if let Some(id) = inner.preview {
                            Command::perform(async {}, move |_| AppMsg::SetSelection((id, s)))
                        } else {
                            Command::none()
                        }
                    }
                    AppMsg::SetSelection((id, s)) => {
                        inner.thumbnail_view.set_selection(&id, s);
                        Command::perform(
                            async move { update_selection_state(&database, id, s).await.unwrap() },
                            move |_| AppMsg::ScrollTo(id),
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
                    AppMsg::DirectoryNext => Command::none(),
                    AppMsg::DirectoryPrev => Command::none(),
                    AppMsg::ThumbnailNext => {
                        tracing::info!("Selecting Next");
                        inner.preview = inner.thumbnail_view.next(inner.preview.as_ref());
                        if let Some(id) = inner.preview {
                            Command::perform(async move {}, move |_| AppMsg::ScrollTo(id))
                        } else {
                            Command::none()
                        }
                    }
                    AppMsg::ThumbnailPrev => {
                        tracing::info!("Selecting Prev");
                        inner.preview = inner.thumbnail_view.prev(inner.preview.as_ref());
                        if let Some(id) = inner.preview {
                            Command::perform(async move {}, move |_| AppMsg::ScrollTo(id))
                        } else {
                            Command::none()
                        }
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
                match inner.app_view {
                    AppView::Preview => {
                        column![
                            inner.menu_view(),
                            inner.preview_view(),
                            inner.thumbnail_view()
                        ]
                        .width(Length::Fill)
                        .height(Length::Fill)
                    }
                    AppView::Grid => {
                        column![inner.menu_view(), inner.grid_view(),]
                            .width(Length::Fill)
                            .height(Length::Fill)
                    }
                }
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
                    // TODO: Keyboard Navigation of directories
                    // KeyCode::H | KeyCode::Left => Some(AppMsg::DirectoryPrev),
                    // KeyCode::H | KeyCode::Left => Some(AppMsg::DirectoryNext),
                    KeyCode::P => Some(AppMsg::SetSelectionCurrent(Selection::Pick)),
                    KeyCode::O => Some(AppMsg::SetSelectionCurrent(Selection::Ordinary)),
                    KeyCode::I => Some(AppMsg::SetSelectionCurrent(Selection::Ignore)),
                    _ => None,
                },
                _ => None,
            }),
        }
    }

    fn title(&self) -> String {
        String::from("Decimator")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
