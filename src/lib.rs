use std::cell::RefCell;

use camino::Utf8PathBuf;
use data::{
    query_directory_pictures, query_unique_directories, update_selection_state, update_thumbnails,
};
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key};
use iced::widget::scrollable::{scroll_to, AbsoluteOffset, Id, Properties};
use iced::widget::{button, column, container, row, scrollable, text, Scrollable};
use iced::Event::Keyboard;
use iced::{event, Application, Command, Element, Length, Subscription, Theme};
use iced_aw::Wrap;
use import::find_new_images;
use itertools::Itertools;
use sea_orm::DatabaseConnection;
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
    /// The request to open the directory selection menu
    DirectoryAdd,
    /// The request to open the directory selection menu
    DirectoryImport,
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
    SelectionExport,
    // Contains the path where the files are being exported to
    SelectionPrint,
    UpdatePictureView(Option<Uuid>),
    ThumbnailNext,
    ThumbnailPrev,
    DirectoryNext,
    DirectoryPrev,
    Ignore,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Preview,
    #[default]
    Grid,
}

#[derive(Debug)]
pub struct App {
    database: DatabaseConnection,
    directories: Vec<DirectoryData>,
    directory: Option<Utf8PathBuf>,

    app_view: AppView,
    thumbnail_view: ThumbnailData,
    thumbnail_scroller: Id,

    preview: Option<Uuid>,
    preview_cache: RefCell<lru::LruCache<Uuid, iced::widget::image::Handle>>,
}

impl App {
    fn menu_view(&self) -> Element<AppMsg> {
        // horizontal_space().into()
        menu::menu_view(self).into()
    }

    fn directory_view(&self) -> Element<AppMsg> {
        let dirs = self.directories.iter().sorted_unstable().rev();

        let values: Vec<_> = dirs.clone().zip(dirs.map(DirectoryData::view)).collect();
        column![
            row![
                button(text("Add")).on_press(AppMsg::DirectoryAdd),
                // horizontal_space(),
                button(text("Import")).on_press(AppMsg::DirectoryImport),
                button(text("Export")).on_press(AppMsg::SelectionExport),
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

impl Application for App {
    type Flags = DatabaseConnection;
    type Message = AppMsg;
    type Theme = Theme;
    type Executor = iced::executor::Default;

    #[tracing::instrument(name = "Initialising App")]
    fn new(database: DatabaseConnection) -> (Self, Command<Self::Message>) {
        let app = Self {
            database,
            directories: vec![],
            directory: None,
            app_view: Default::default(),
            thumbnail_view: Default::default(),
            thumbnail_scroller: Id::unique(),
            preview: None,
            preview_cache: RefCell::new(lru::LruCache::new(20.try_into().unwrap())),
        };
        (
            app,
            Command::perform(async {}, |_| AppMsg::QueryDirectories),
        )
    }

    #[tracing::instrument(name = "Updating App", level = "info", skip(self))]
    fn update(&mut self, msg: Self::Message) -> Command<AppMsg> {
        let database = self.database.clone();
        match msg {
            AppMsg::SetView(view) => {
                self.app_view = view;
                Command::none()
            }
            AppMsg::DirectoryImport => Command::perform(
                async move {
                    let dir: Utf8PathBuf = rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .expect("No Directory found")
                        .path()
                        .to_str()
                        .unwrap()
                        .into();

                    import(&database, &dir).await.unwrap()
                },
                |_| AppMsg::QueryDirectories,
            ),
            AppMsg::DirectoryAdd => Command::perform(
                async move {
                    let dir = rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .expect("No Directory found")
                        .path()
                        .to_str()
                        .unwrap()
                        .into();
                    find_new_images(&database, &dir).await;
                },
                |_| AppMsg::QueryDirectories,
            ),
            AppMsg::QueryDirectories => Command::perform(
                async move { query_unique_directories(&database).await.unwrap() },
                AppMsg::UpdateDirectories,
            ),
            AppMsg::UpdateDirectories(dirs) => {
                self.directories = dirs;
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
                self.directory = Some(dir.clone());
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
                self.thumbnail_view.set_thumbnails(thumbnails);
                // Default to selecting the first image within a directory
                self.preview = self.thumbnail_view.positions().first().copied();
                Command::none()
            }
            // Modify Thumbnail filters
            AppMsg::DisplayPick(value) => {
                self.thumbnail_view.set_pick(value);
                Command::none()
            }
            AppMsg::DisplayOrdinary(value) => {
                self.thumbnail_view.set_ordinary(value);
                Command::none()
            }
            AppMsg::DisplayIgnore(value) => {
                self.thumbnail_view.set_ignore(value);
                Command::none()
            }
            AppMsg::DisplayHidden(value) => {
                self.thumbnail_view.set_hidden(value);
                Command::none()
            }
            // TODO: Implement ScrollTo action
            AppMsg::ScrollTo(id) => {
                let offset = self.thumbnail_view.get_position(&id).unwrap() as f32 * 240.;
                scroll_to(
                    self.thumbnail_scroller.clone(),
                    AbsoluteOffset { x: offset, y: 0. },
                )
            }
            AppMsg::SetSelectionCurrent(s) => {
                if let Some(id) = self.preview {
                    self.update(AppMsg::SetSelection((id, s)))
                } else {
                    Command::none()
                }
            }
            AppMsg::SetSelection((id, s)) => {
                self.thumbnail_view.set_selection(&id, s);
                Command::perform(
                    async move { update_selection_state(&database, id, s).await.unwrap() },
                    move |_| AppMsg::Ignore,
                )
            }
            AppMsg::SelectionExport => {
                let items = self.thumbnail_view.get_view();
                Command::perform(
                    async move {
                        let dir: Utf8PathBuf = rfd::AsyncFileDialog::new()
                            .pick_folder()
                            .await
                            .expect("No Directory found")
                            .path()
                            .to_str()
                            .unwrap()
                            .into();

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
            AppMsg::SelectionPrint => Command::none(),
            AppMsg::Ignore => Command::none(),
            AppMsg::DirectoryNext => Command::none(),
            AppMsg::DirectoryPrev => Command::none(),
            AppMsg::ThumbnailNext => {
                tracing::debug!("Selecting Next Thumbnail");
                self.preview = self.thumbnail_view.next(self.preview.as_ref());
                if let Some(id) = self.preview {
                    self.update(AppMsg::ScrollTo(id))
                } else {
                    Command::none()
                }
            }
            AppMsg::ThumbnailPrev => {
                tracing::debug!("Selecting Prev Thumbnail");
                self.preview = self.thumbnail_view.prev(self.preview.as_ref());
                if let Some(id) = self.preview {
                    Command::perform(async move {}, move |_| AppMsg::ScrollTo(id))
                } else {
                    Command::none()
                }
            }
            AppMsg::UpdatePictureView(preview) => {
                self.preview = preview;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<AppMsg> {
        let content: Element<AppMsg> = row![
            self.directory_view(),
            match self.app_view {
                AppView::Preview => {
                    column![self.menu_view(), self.preview_view(), self.thumbnail_view()]
                        .width(Length::Fill)
                        .height(Length::Fill)
                }
                AppView::Grid => {
                    column![self.menu_view(), self.grid_view()]
                        .width(Length::Fill)
                        .height(Length::Fill)
                }
            }
        ]
        .into();
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<AppMsg> {
        let keyboard_sub = event::listen_with(|event, _| match event {
            Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                match key.as_ref() {
                    Key::Character("h") | Key::Named(Named::ArrowLeft) => {
                        Some(AppMsg::ThumbnailPrev)
                    }
                    Key::Character("l") | Key::Named(Named::ArrowRight) => {
                        Some(AppMsg::ThumbnailNext)
                    }
                    // TODO: Keyboard Navigation of directories
                    // KeyCode::H | KeyCode::Left => Some(AppMsg::DirectoryPrev),
                    // KeyCode::H | KeyCode::Left => Some(AppMsg::DirectoryNext),
                    Key::Character("p") => Some(AppMsg::SetSelectionCurrent(Selection::Pick)),
                    Key::Character("o") => Some(AppMsg::SetSelectionCurrent(Selection::Ordinary)),
                    Key::Character("i") => Some(AppMsg::SetSelectionCurrent(Selection::Ignore)),
                    _ => None,
                }
            }
            _ => None,
        });
        keyboard_sub
    }

    fn title(&self) -> String {
        String::from("Decimator")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
