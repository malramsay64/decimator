use camino::Utf8PathBuf;
use data::{query_directory_pictures, query_unique_directories, update_thumbnails};
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::Event::Keyboard;
use iced::{event, Element, Length, Subscription, Task};
use import::find_new_images;
use itertools::Itertools;
use sea_orm::DatabaseConnection;
use tracing::info;
use uuid::Uuid;

use crate::import::import;

mod data;
pub mod directory;
mod import;
// The menu is not currently working with the iced master branch
mod menu;
mod picture;
pub mod telemetry;
mod thumbnail;
mod widget;

use directory::{DirectoryData, DirectoryMessage, DirectoryView};
use picture::{PictureData, PictureThumbnail};
use thumbnail::{ThumbnailMessage, ThumbnailView};

#[derive(Debug, Clone)]
pub enum AppMessage {}

impl From<AppMessage> for Message {
    fn from(val: AppMessage) -> Self {
        Message::App(val)
    }
}

#[derive(Debug, Clone)]
pub enum DatabaseMessage {
    UpdateImage(PictureData),
    LoadThumbnail(Uuid),
}

impl From<DatabaseMessage> for Message {
    fn from(val: DatabaseMessage) -> Self {
        Message::Database(val)
    }
}

/// Messages for running the application
#[derive(Debug, Clone)]
pub enum Message {
    Thumbnail(ThumbnailMessage),
    Database(DatabaseMessage),
    Directory(DirectoryMessage),
    App(AppMessage),
    UpdateThumbnails(bool),
    // Signal to emit when we want to export, this creates the export dialog
    SetView(AppView),
    SelectionExport,
    // Contains the path where the files are being exported to
    SelectionPrint,
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
    // directories: Vec<DirectoryData>,
    // directory: Option<Utf8PathBuf>,
    app_view: AppView,
    thumbnail_view: ThumbnailView,
    directory_view: DirectoryView,
}

impl App {
    fn menu_view(&self) -> Element<Message> {
        menu::menu_view(self)
    }

    #[tracing::instrument(name = "Initialising App")]
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            database: database.clone(),
            directory_view: DirectoryView::new(database.clone()),
            app_view: Default::default(),
            thumbnail_view: ThumbnailView::new(database, 20.try_into().unwrap()),
        }
    }

    #[tracing::instrument(name = "Updating App", level = "info", skip(self))]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        let database = self.database.clone();
        match message {
            Message::Database(m) => Task::none(),
            Message::Thumbnail(m) => self.thumbnail_view.update(m),
            Message::App(m) => Task::none(),
            Message::Directory(m) => self.directory_view.update(m),
            Message::SetView(view) => {
                self.app_view = view;
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
            // Modify Thumbnail filters
            Message::SelectionExport => {
                let items: Vec<_> = self.thumbnail_view.get_view().cloned().collect();
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
                            let origin = file.data.filepath.clone();
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
        }
    }

    pub fn view(&self) -> Element<Message> {
        let content: Element<Message> = row![
            self.directory_view.view(),
            match self.app_view {
                AppView::Preview => {
                    column![
                        menu::menu_view(self),
                        self.thumbnail_view.get_preview_view()
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                }
                AppView::Grid => {
                    column![menu::menu_view(self), self.thumbnail_view.get_grid_view()]
                        .width(Length::Fill)
                        .height(Length::Fill)
                }
            }
        ]
        .into();
        container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard_sub = event::listen_with(|event, _, _| match event {
            Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                match key.as_ref() {
                    Key::Character("h") | Key::Named(Named::ArrowLeft) => {
                        Some(ThumbnailMessage::Prev.into())
                    }
                    Key::Character("l") | Key::Named(Named::ArrowRight) => {
                        Some(ThumbnailMessage::Next.into())
                    }
                    // TODO: Keyboard Navigation of directories
                    // KeyCode::H | KeyCode::Left => Some(Message::DirectoryPrev),
                    // KeyCode::H | KeyCode::Left => Some(Message::DirectoryNext),
                    // Key::Character("p") => Some(Message::SetSelectionCurrent(Selection::Pick)),
                    // Key::Character("o") => Some(Message::SetSelectionCurrent(Selection::Ordinary)),
                    // Key::Character("i") => Some(Message::SetSelectionCurrent(Selection::Ignore)),
                    _ => None,
                }
            }
            _ => None,
        });
        keyboard_sub
    }
}
