use std::cell::RefCell;

use camino::Utf8PathBuf;
use data::{
    query_directory_pictures, query_unique_directories, update_selection_state, update_thumbnails,
};
use iced::keyboard::KeyCode;
use iced::widget::{
    button, column, container, horizontal_space, lazy, row, scrollable, text, toggler,
};
use iced::{Application, Command, Element, Length, Theme};
use iced_aw::native::Grid;
use iced_aw::{menu_bar, menu_tree, quad, CloseCondition, MenuTree};
use iced_widget::scrollable::{Id, Properties};
use import::find_new_images;
use itertools::Itertools;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use selection_list::SelectionListBuilder;
use uuid::Uuid;
use widget::choice;

use crate::import::import;
use crate::widget::viewer;

mod data;
mod directory;
mod import;
mod picture;
pub mod telemetry;
mod thumbnail;
mod widget;
use directory::DirectoryData;
use entity::Selection;
use picture::PictureThumbnail;
use thumbnail::ThumbnailView;

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
    SetSelection(Selection),
    // Signal to emit when we want to export, this creates the export dialog
    SetView(AppView),
    SelectionExportRequest,
    // Contains the path where the files are being exported to
    SelectionExport(Utf8PathBuf),
    SelectionPrintRequest,
    UpdatePictureView(Option<Uuid>),
    ThumbnailNext,
    ThumbnailPrev,
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
    thumbnail_view: ThumbnailView,
    thumbnail_scroller: scrollable::Id,

    preview: Option<Uuid>,
    preview_cache: RefCell<lru::LruCache<Uuid, iced::widget::image::Handle>>,
}

fn separator<'a>() -> MenuTree<'a, AppMsg, iced::Renderer> {
    menu_tree!(quad::Quad {
        color: [0.5; 3].into(),
        border_radius: [4.0; 4],
        inner_bounds: quad::InnerBounds::Ratio(0.98, 0.1),
        ..Default::default()
    })
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
        let menu: Element<AppMsg> = menu_bar!(MenuTree::with_children(
            button("Menu"),
            vec![
                MenuTree::new(toggler(
                    String::from("Pick"),
                    self.thumbnail_view.pick(),
                    AppMsg::DisplayPick
                )),
                MenuTree::new(toggler(
                    String::from("Ordinary"),
                    self.thumbnail_view.ordinary(),
                    AppMsg::DisplayOrdinary
                )),
                MenuTree::new(toggler(
                    String::from("Ignore"),
                    self.thumbnail_view.ignore(),
                    AppMsg::DisplayIgnore
                )),
                MenuTree::new(toggler(
                    String::from("Hidden"),
                    self.thumbnail_view.hidden(),
                    AppMsg::DisplayHidden
                )),
                separator(),
                menu_tree!(button(text("Generate New Thumbnails"))
                    .on_press(AppMsg::UpdateThumbnails(true))),
                menu_tree!(
                    button(text("Redo All Thumbnails")).on_press(AppMsg::UpdateThumbnails(false))
                ),
            ]
        )
        .width(400))
        .close_condition(CloseCondition {
            leave: true,
            click_inside: false,
            click_outside: true,
        })
        .into();
        let tabs = row!(
            choice(
                text("Preview").into(),
                AppView::Preview,
                Some(self.app_view),
                AppMsg::SetView
            ),
            choice(
                text("Grid").into(),
                AppView::Grid,
                Some(self.app_view),
                AppMsg::SetView
            ),
        );
        row!(tabs, horizontal_space(Length::Fill), menu)
            .padding(10)
            .align_items(iced::Alignment::Center)
            .into()
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
            SelectionListBuilder::new(values, |dir| {
                AppMsg::SelectDirectory(DirectoryData::add_prefix(&dir.directory))
            },)
            .width(200.)
            .build()
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
        SelectionListBuilder::new(view, |i| AppMsg::UpdatePictureView(Some(i)))
            .direction(selection_list::Direction::Horizontal)
            .height(320.)
            .build()
            .into()
    }

    fn grid_view(&self) -> Element<AppMsg> {
        let thumbnails = lazy(self.thumbnail_view.version(), |_| {
            self.thumbnail_view
                .get_view()
                .into_iter()
                .map(PictureThumbnail::view)
                .fold(
                    Grid::new()
                        .width(Length::Fill)
                        .strategy(iced_aw::Strategy::ColumnWidthFlex(260.)),
                    |i, g| i.push(g),
                )
        });
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
                    AppMsg::ScrollTo(_id) => Command::none(),
                    AppMsg::SetSelection(s) => {
                        if let Some(id) = inner.preview {
                            inner.thumbnail_view.set_selection(&id, s);
                            Command::perform(
                                async move { update_selection_state(&database, id, s).await.unwrap() },
                                move |_| AppMsg::ScrollTo(id),
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
                        if let Some(id) = inner.preview {
                            Command::perform(async move {}, move |_| AppMsg::ScrollTo(id))
                        } else {
                            Command::none()
                        }
                    }
                    AppMsg::ThumbnailPrev => {
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

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
