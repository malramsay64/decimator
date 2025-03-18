use std::borrow::Borrow;

use anyhow::Error;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use data::{query_directory_pictures, query_unique_directories, update_thumbnails, Progress};
use entity::directory as entity_directory;
use entity::directory::Model;
use entity::picture as entity_picture;
use entity::Selection;
use futures::future::Select;
use futures::{StreamExt, TryStreamExt};
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key};
use iced::wgpu::hal::ProgrammableStage;
use iced::widget::{button, column, container, progress_bar, row, scrollable, text};
use iced::Color;
use iced::Event::Keyboard;
use iced::Theme;
use iced::{event, task, Element, Length, Subscription, Task};
use import::find_new_images;
use itertools::Itertools;
use picture::ThumbnailData;
use sea_orm::entity::*;
use sea_orm::prelude::*;
use sea_orm::query::*;
use sea_orm::ActiveValue;
use sea_orm::DatabaseConnection;
use tracing::info;
use uuid::Uuid;

use crate::import::import;

mod data;
pub mod directory;
mod import;
// The menu is not currently working with the iced master branch
mod menu;
pub mod picture;
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
#[derive(Debug, Clone, PartialEq, Eq, Ord)]
pub struct DirectoryDataDB {
    id: Uuid,
    directory: Utf8PathBuf,
    parent_id: Option<Uuid>,
    children: Vec<Uuid>,
}

fn directory_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let mut style = match status {
        button::Status::Active => {
            button::Style::default().with_background(palette.background.base.color)
        }
        button::Status::Hovered => {
            button::Style::default().with_background(palette.background.strong.color)
        }
        button::Status::Pressed => {
            button::Style::default().with_background(palette.primary.base.color)
        }
        _ => button::primary(theme, status),
    };
    style.text_color = Color::WHITE;
    style
}
impl DirectoryDataDB {
    fn new(model: entity::directory::Model, children: Vec<entity::directory::Model>) -> Self {
        Self {
            id: model.id,
            directory: model.directory.into(),
            parent_id: model.parent_id,
            children: children.into_iter().map(|i| i.id).collect(),
        }
    }
    fn into_active(self) -> entity::directory::ActiveModel {
        entity_directory::ActiveModel {
            id: ActiveValue::Unchanged(self.id),
            directory: ActiveValue::Set(self.directory.to_string()),
            parent_id: ActiveValue::Set(self.parent_id),
        }
    }
    pub fn strip_prefix(&self) -> &Utf8Path {
        &self.directory
        // .strip_prefix(dirs::home_dir().unwrap())
        // .unwrap()
    }
    pub fn view(&self, selected: bool) -> Element<'_, Message> {
        let message = if selected {
            None
        } else {
            Some(DirectoryMessage::SelectDirectory(self.clone()).into())
        };
        button(text(self.strip_prefix().as_str()).width(Length::Fill))
            .on_press_maybe(message)
            .style(directory_style)
            .into()
    }
}

impl PartialOrd for DirectoryDataDB {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.directory.partial_cmp(&other.directory)
    }
}

impl From<entity::directory::Model> for DirectoryDataDB {
    fn from(value: entity::directory::Model) -> Self {
        Self {
            id: value.id,
            directory: value.directory.into(),
            parent_id: value.parent_id,
            children: vec![],
        }
    }
}

async fn get_parent_directory(
    db: &DatabaseConnection,
    current_dir: &Utf8PathBuf,
) -> Result<Option<Uuid>, Error> {
    let d = entity::directory::Entity::find()
        .filter(entity::directory::Column::Directory.eq(current_dir.clone().into_string()))
        .one(db)
        .await?;
    tracing::debug!("Directories {:?}", d);
    // The directory already exists within the database
    if let Some(db_dir) = d {
        tracing::debug!("Directory already exists {:?}", db_dir);
        Ok(Some(db_dir.id))
    // We have a parent directory that could be created
    } else if let Some(parent) = current_dir.parent() {
        let parent_id = Box::pin(get_parent_directory(db, &parent.to_path_buf())).await?;
        let id = Uuid::new_v4();
        let db_dir = DirectoryDataDB {
            parent_id,
            id,
            directory: current_dir.clone(),
            children: vec![],
        };
        tracing::debug!("Adding directory {db_dir:?} from parent: {parent:?}");
        entity::directory::Entity::insert(db_dir.into_active())
            .exec(db)
            .await
            .inspect_err(|e| tracing::error!("{e:?}"))?;
        Ok(Some(id))
    } else {
        tracing::debug!("No parent directories");
        Ok(None)
    }
}

async fn update_database(database: &DatabaseConnection) -> Result<(), Error> {
    let mut stream = entity_picture::Entity::find().stream(database).await?;
    while let Some(p) = stream.next().await {
        let p = p.expect("Value not loaded correctly.");
        let directory: Utf8PathBuf = p.directory.clone().into();
        tracing::debug!("Updating Picture: {}", p.filename);
        let d = get_parent_directory(database, &directory).await?;
        tracing::debug!("Found Parent Directory: {:?}", d);
        let picture_update = entity::picture::ActiveModel {
            id: ActiveValue::Unchanged(p.id),
            directory_id: ActiveValue::Set(d),
            ..Default::default()
        };

        entity_picture::Entity::update(picture_update)
            .exec(database)
            .await
            .inspect_err(|e| tracing::error!("{e:?}"))?;
    }
    Ok(())
}

/// Messages for running the application
#[derive(Debug, Clone)]
pub enum Message {
    ThumbnailUpdate(Progress),
    ThumbnailFinished(Result<(), data::ThumbnailError>),
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
    Update,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    Preview,
    #[default]
    Grid,
}

#[derive(Debug, Clone, Default)]
pub enum DownloadState {
    #[default]
    Idle,
    Downloading {
        progress: f32,
        _task: task::Handle,
    },
    Finished,
    Errored,
}

#[derive(Debug)]
pub struct App {
    database: DatabaseConnection,
    // directories: Vec<DirectoryData>,
    // directory: Option<Utf8PathBuf>,
    app_view: AppView,
    thumbnail_view: ThumbnailView,
    directory_view: DirectoryView,
    thumbnail_import: DownloadState,
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
            thumbnail_import: Default::default(),
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
            Message::ThumbnailUpdate(new_progress) => {
                if let DownloadState::Downloading { progress, .. } = &mut self.thumbnail_import {
                    *progress = new_progress.percent;
                }
                Task::none()
            }
            Message::ThumbnailFinished(result) => {
                self.thumbnail_import = match result {
                    Ok(_) => DownloadState::Finished,
                    Err(_) => DownloadState::Errored,
                };
                // self.thumbnail_import = None;
                Task::none()
            }
            Message::UpdateThumbnails(all) => {
                let (task, handle) = Task::sip(
                    update_thumbnails(&database, all),
                    Message::ThumbnailUpdate,
                    Message::ThumbnailFinished,
                )
                .abortable();
                self.thumbnail_import = DownloadState::Downloading {
                    progress: 0.,
                    _task: handle.abort_on_drop(),
                };
                task
            }
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
            Message::Update => {
                let database = self.database.clone();
                Task::perform(async move { update_database(&database).await }, |_| {
                    Message::Ignore
                })
            }
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
        let selected = self.thumbnail_view.get_selected();
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
                    Key::Character("p") => {
                        Some(ThumbnailMessage::SetSelectionCurrent(Selection::Pick).into())
                    }
                    Key::Character("o") => {
                        Some(ThumbnailMessage::SetSelectionCurrent(Selection::Ordinary).into())
                    }
                    Key::Character("i") => {
                        Some(ThumbnailMessage::SetSelectionCurrent(Selection::Ignore).into())
                    }
                    _ => None,
                }
            }
            _ => None,
        });
        keyboard_sub
    }
}
