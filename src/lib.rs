use std::cell::RefCell;

use camino::Utf8PathBuf;
use data::{
    query_directory_pictures, query_unique_directories, update_selection_state, update_thumbnails,
};
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key};
use iced::widget::scrollable::{scroll_to, AbsoluteOffset, Id};
use iced::widget::{button, column, container, row, scrollable, text, Scrollable};
use iced::Event::Keyboard;
use iced::{event, Element, Length, Subscription, Task};
use iced_aw::Wrap;
use import::find_new_images;
use itertools::Itertools;
use sea_orm::DatabaseConnection;
use selection_list::SelectionList;
use tracing::info;
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
pub enum Message {
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
    fn menu_view(&self) -> Element<Message> {
        // horizontal_space().into()
        menu::menu_view(self)
    }

    fn directory_view(&self) -> Element<Message> {
        let dirs = self.directories.iter().sorted_unstable().rev();
        let mut position = None;
        if let Some(dir) = self.directory.clone() {
            position = dirs.clone().position(|d| dir == d.directory);
        }
        info!("Position: {:?}", position);

        let values: Vec<_> = dirs.clone().zip(dirs.map(DirectoryData::view)).collect();
        column![
            row![
                button(text("Add")).on_press(Message::DirectoryAdd),
                // horizontal_space(),
                button(text("Import")).on_press(Message::DirectoryImport),
                button(text("Export")).on_press(Message::SelectionExport),
            ]
            // The row doesn't introspect size automatically, so we have to force it with the calls to width and height
            .height(Length::Shrink)
            // .width(Length::Fill)
            .padding(10.),
            Scrollable::new(
                SelectionList::new_with_selection(
                    values,
                    |dir| { Message::SelectDirectory(DirectoryData::add_prefix(&dir.directory)) },
                    position
                )
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
    fn thumbnail_view(&self) -> Element<Message> {
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
                |i| Message::UpdatePictureView(Some(i)),
                position,
            )
            .direction(selection_list::Direction::Horizontal)
            .item_height(320.)
            .item_width(240.)
            .height(320.),
        )
        .id(self.thumbnail_scroller.clone())
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default(),
        ))
        .into()
    }

    /// Provides an overview of all the images on a grid
    fn grid_view(&self) -> Element<Message> {
        let mut thumbnails = self
            .thumbnail_view
            .get_view()
            .into_iter()
            .map(PictureThumbnail::view)
            .fold(Wrap::new(), |i, g| i.push(g));
        thumbnails.width = Length::Fill;
        scrollable(thumbnails)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new().width(2.).scroller_width(10.),
            ))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn preview_view(&self) -> Element<Message> {
        if let Some(id) = &self.preview {
            if let Some(filepath) = self.thumbnail_view.get_filepath(id) {
                let mut cache = self.preview_cache.borrow_mut();
                let handle = match cache.get(id) {
                    Some(h) => h,
                    None => {
                        let i = picture::load_image(filepath.as_path(), None).unwrap();
                        let handle = iced::widget::image::Handle::from_rgba(
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
                // .width(Length::Fill)
                // .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
            }
        }
        container(text("No image available"))
            // .height(Length::Fill)
            // .width(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

impl App {
    #[tracing::instrument(name = "Initialising App")]
    pub fn new(database: DatabaseConnection) -> Self {
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

    #[tracing::instrument(name = "Updating App", level = "info", skip(self))]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        let database = self.database.clone();
        match message {
            Message::SetView(view) => {
                self.app_view = view;
                Task::none()
            }
            Message::DirectoryImport => Task::perform(
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
                |_| Message::QueryDirectories,
            ),
            Message::DirectoryAdd => Task::perform(
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
                |_| Message::QueryDirectories,
            ),
            Message::QueryDirectories => Task::perform(
                async move { query_unique_directories(&database).await.unwrap() },
                Message::UpdateDirectories,
            ),
            Message::UpdateDirectories(dirs) => {
                self.directories = dirs;
                Task::none()
            }
            Message::UpdateThumbnails(all) => Task::perform(
                async move {
                    // TODO: Add a dialog confirmation box
                    update_thumbnails(&database, all)
                        .await
                        .expect("Unable to update thumbnails");
                },
                |_| Message::Ignore,
            ),
            Message::SelectDirectory(dir) => {
                self.directory = Some(dir.clone());
                Task::perform(
                    async move {
                        query_directory_pictures(&database, dir.into())
                            .await
                            .unwrap()
                    },
                    Message::SetThumbnails,
                )
            }
            Message::SetThumbnails(thumbnails) => {
                self.thumbnail_view.set_thumbnails(thumbnails);
                // Default to selecting the first image within a directory
                self.preview = self.thumbnail_view.positions().first().copied();
                Task::none()
            }
            // Modify Thumbnail filters
            Message::DisplayPick(value) => {
                self.thumbnail_view.set_pick(value);
                Task::none()
            }
            Message::DisplayOrdinary(value) => {
                self.thumbnail_view.set_ordinary(value);
                Task::none()
            }
            Message::DisplayIgnore(value) => {
                self.thumbnail_view.set_ignore(value);
                Task::none()
            }
            Message::DisplayHidden(value) => {
                self.thumbnail_view.set_hidden(value);
                Task::none()
            }
            // TODO: Implement ScrollTo action
            Message::ScrollTo(id) => {
                let offset = self.thumbnail_view.get_position(&id).unwrap() as f32 * 240.;
                scroll_to(
                    self.thumbnail_scroller.clone(),
                    AbsoluteOffset { x: offset, y: 0. },
                )
            }
            Message::SetSelectionCurrent(s) => {
                if let Some(id) = self.preview {
                    self.update(Message::SetSelection((id, s)))
                } else {
                    Task::none()
                }
            }
            Message::SetSelection((id, s)) => {
                self.thumbnail_view.set_selection(&id, s);
                Task::perform(
                    async move { update_selection_state(&database, id, s).await.unwrap() },
                    move |_| Message::Ignore,
                )
            }
            Message::SelectionExport => {
                let items = self.thumbnail_view.get_view();
                Task::perform(
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
                    |_| Message::Ignore,
                )
            }
            Message::SelectionPrint => Task::none(),
            Message::Ignore => Task::none(),
            Message::DirectoryNext => Task::none(),
            Message::DirectoryPrev => Task::none(),
            Message::ThumbnailNext => {
                tracing::debug!("Selecting Next Thumbnail");
                self.preview = self.thumbnail_view.next(self.preview.as_ref());
                if let Some(id) = self.preview {
                    self.update(Message::ScrollTo(id))
                } else {
                    Task::none()
                }
            }
            Message::ThumbnailPrev => {
                tracing::debug!("Selecting Prev Thumbnail");
                self.preview = self.thumbnail_view.prev(self.preview.as_ref());
                if let Some(id) = self.preview {
                    Task::perform(async move {}, move |_| Message::ScrollTo(id))
                } else {
                    Task::none()
                }
            }
            Message::UpdatePictureView(preview) => {
                self.preview = preview;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let content: Element<Message> = row![
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
            // .width(Length::Fill)
            // .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard_sub = event::listen_with(|event, _, _| match event {
            Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                match key.as_ref() {
                    Key::Character("h") | Key::Named(Named::ArrowLeft) => {
                        Some(Message::ThumbnailPrev)
                    }
                    Key::Character("l") | Key::Named(Named::ArrowRight) => {
                        Some(Message::ThumbnailNext)
                    }
                    // TODO: Keyboard Navigation of directories
                    // KeyCode::H | KeyCode::Left => Some(Message::DirectoryPrev),
                    // KeyCode::H | KeyCode::Left => Some(Message::DirectoryNext),
                    Key::Character("p") => Some(Message::SetSelectionCurrent(Selection::Pick)),
                    Key::Character("o") => Some(Message::SetSelectionCurrent(Selection::Ordinary)),
                    Key::Character("i") => Some(Message::SetSelectionCurrent(Selection::Ignore)),
                    _ => None,
                }
            }
            _ => None,
        });
        keyboard_sub
    }
}
