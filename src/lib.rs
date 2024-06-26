use std::cell::RefCell;

use camino::Utf8PathBuf;
use data::{
    query_directory_pictures, query_unique_directories, update_selection_state, update_thumbnails,
};
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key};
use iced::widget::scrollable::{scroll_to, AbsoluteOffset, Id, Properties};
use iced::widget::{button, column, container, row, scrollable, text, Scrollable};
use iced::{event, Application, Command, Element, Event, Length, Subscription, Theme};
use iced_aw::Wrap;
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
// The menu is not currently working with the iced master branch
mod menu;
mod picture;
pub mod telemetry;
mod thumbnail;
mod widget;

use directory::DirectoryData;
use entity::Selection;
use picture::PictureThumbnail;
use thumbnail::ThumbnailData;

/// Messages for running the application
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
    EventOccurred(Event),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Preview,
    #[default]
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
        // horizontal_space().into()
        menu::menu_view(self).into()
    }

    fn directory_view(&self) -> Element<AppMsg> {
        let dirs = self.directories.iter().sorted_unstable().rev();

        let values: Vec<_> = dirs.clone().zip(dirs.map(DirectoryData::view)).collect();
        column![
            row![
                button(text("Add")).on_press(AppMsg::DirectoryAddRequest),
                // horizontal_space(),
                button(text("Import")).on_press(AppMsg::DirectoryImportRequest),
                button(text("Export")).on_press(AppMsg::SelectionExportRequest),
            ]
            // The row doesn't introspect size automatically, so we have to force it with the calls to width and height
            .height(Length::Shrink)
            // .width(Length::Fill)
            .padding(10.),
            Scrollable::new(
                SelectionList::new(values, |dir| {
                    AppMsg::SelectDirectory(DirectoryData::add_prefix(&dir.directory))
                },)
                .item_width(250.)
                .item_height(30.)
                .width(260.)
            )
            .height(Length::Fill)
        ]
        .width(Length::Shrink)
        .height(Length::Fill)
        .into()
    }

    #[tracing::instrument(name = "Update Thumbnail View", level = "info", skip(self))]
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

        let position = self
            .preview
            .map_or(Some(0), |id| self.thumbnail_view.get_position(&id));
        tracing::trace!(
            "Updating thumbnail scrollable view with preview {:?}, at position {:?}",
            self.preview,
            position
        );
        Scrollable::new(
            SelectionList::new_with_selection(
                view,
                |i| AppMsg::UpdatePictureView(Some(i)),
                position,
            )
            .direction(selection_list::Direction::Horizontal)
            .item_height(320.)
            .item_width(240.)
            .height(320.),
        )
        .id(self.thumbnail_scroller.clone())
        .direction(scrollable::Direction::Horizontal(Properties::default()))
        .into()
    }

    /// Provides an overview of all the images on a grid
    fn grid_view(&self) -> Element<AppMsg> {
        let mut thumbnails = self
            .thumbnail_view
            .get_view()
            .into_iter()
            .map(PictureThumbnail::view)
            .fold(Wrap::new(), |i, g| i.push(g));
        thumbnails.width = Length::Fill;
        scrollable(thumbnails)
            .direction(scrollable::Direction::Vertical(
                Properties::new().width(2.).scroller_width(10.),
            ))
            .width(Length::Fill)
            .height(Length::Fill)
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
                            move |_| AppMsg::Ignore,
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
                    AppMsg::EventOccurred(event) => {
                        match event {
                            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                                match key.as_ref() {
                                    keyboard::Key::Character("h")
                                    | Key::Named(Named::ArrowLeft) => {
                                        Command::perform(async move {}, move |_| {
                                            AppMsg::ThumbnailPrev
                                        })
                                    }
                                    keyboard::Key::Character("l")
                                    | Key::Named(Named::ArrowRight) => {
                                        Command::perform(async move {}, move |_| {
                                            AppMsg::ThumbnailNext
                                        })
                                    }
                                    // TODO: Keyboard Navigation of directories
                                    // KeyCode::H | KeyCode::Left => Some(AppMsg::DirectoryPrev),
                                    // KeyCode::H | KeyCode::Left => Some(AppMsg::DirectoryNext),
                                    Key::Character("p") => {
                                        Command::perform(async move {}, move |_| {
                                            AppMsg::SetSelectionCurrent(Selection::Pick)
                                        })
                                    }
                                    Key::Character("o") => {
                                        Command::perform(async move {}, move |_| {
                                            AppMsg::SetSelectionCurrent(Selection::Ordinary)
                                        })
                                    }
                                    Key::Character("i") => {
                                        Command::perform(async move {}, move |_| {
                                            AppMsg::SetSelectionCurrent(Selection::Ignore)
                                        })
                                    }
                                    _ => Command::none(),
                                }
                            }
                            _ => Command::none(),
                        }
                    }
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
                        tracing::debug!("Selecting Next Thumbnail");
                        inner.preview = inner.thumbnail_view.next(inner.preview.as_ref());
                        if let Some(id) = inner.preview {
                            Command::perform(async move {}, move |_| AppMsg::ScrollTo(id))
                        } else {
                            Command::none()
                        }
                    }
                    AppMsg::ThumbnailPrev => {
                        tracing::debug!("Selecting Prev Thumbnail");
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
                        column![inner.menu_view(), inner.grid_view()]
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

    fn subscription(&self) -> Subscription<AppMsg> {
        event::listen().map(AppMsg::EventOccurred)
    }

    fn title(&self) -> String {
        String::from("Decimator")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
