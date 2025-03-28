use camino::{Utf8Path, Utf8PathBuf};
use iced::widget::{button, column, container, horizontal_space, row, scrollable, text};
use iced::{Color, Element, Task, Theme};
use itertools::Itertools;
use sea_orm::DatabaseConnection;

use crate::data::{query_directories, query_directory_pictures};
use crate::import::{find_new_images, import};
use crate::thumbnail::ThumbnailMessage;
use crate::{DirectoryDataDB, Message};

#[derive(Debug, Default, Clone)]
pub enum Active {
    #[default]
    None,
    Single(usize),
    Multiple(Vec<usize>),
}

#[derive(Debug, Clone, Default)]
pub struct DirectoryView {
    pub directories: Vec<DirectoryDataDB>,
    pub selected: Active,
    pub database: DatabaseConnection,
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

#[derive(Debug, Clone)]
pub enum DirectoryMessage {
    /// The request to open the directory selection menu
    DirectoryAdd,
    /// The request to open the directory selection menu
    DirectoryImport,
    QueryDirectories,
    UpdateDirectories(Vec<DirectoryDataDB>),
    SelectDirectory(DirectoryDataDB),
    DirectoryNext,
    DirectoryPrev,
}

impl From<DirectoryMessage> for Message {
    fn from(val: DirectoryMessage) -> Self {
        Message::Directory(val)
    }
}

impl DirectoryView {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            directories: Default::default(),
            selected: Default::default(),
            database,
        }
    }
    fn is_selected(&self, index: &usize) -> bool {
        match &self.selected {
            Active::None => false,
            Active::Single(i) => i == index,
            Active::Multiple(selection_list) => selection_list.contains(index),
        }
    }

    pub fn update(&mut self, message: DirectoryMessage) -> Task<Message> {
        let database = self.database.clone();
        match message {
            DirectoryMessage::DirectoryImport => Task::perform(
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
                |_| DirectoryMessage::QueryDirectories,
            )
            .map(Message::Directory),
            DirectoryMessage::DirectoryAdd => Task::perform(
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
                |_| DirectoryMessage::QueryDirectories,
            )
            .map(Message::Directory),
            DirectoryMessage::QueryDirectories => Task::perform(
                async move { query_directories(&database).await.unwrap() },
                DirectoryMessage::UpdateDirectories,
            )
            .map(Message::Directory),
            DirectoryMessage::UpdateDirectories(dirs) => {
                tracing::debug!("Directories: {:?}", self.directories);
                self.directories = dirs.into_iter().sorted().rev().collect();
                Task::none()
            }
            DirectoryMessage::SelectDirectory(dir) => {
                self.selected =
                    Active::Single(self.directories.iter().position(|d| d == &dir).unwrap());
                Task::perform(
                    async move { query_directory_pictures(&database, dir).await.unwrap() },
                    ThumbnailMessage::SetThumbnails,
                )
                .map(Message::Thumbnail)
            }
            DirectoryMessage::DirectoryNext => todo!(),
            DirectoryMessage::DirectoryPrev => todo!(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let values: Element<'_, Message> = column(
            self.directories
                .iter()
                .enumerate()
                .map(|(index, d)| DirectoryDataDB::view(d, self.is_selected(&index))),
        )
        .into();

        container(
            column![
                row![
                    button(text("Add")).on_press(DirectoryMessage::DirectoryAdd.into()),
                    horizontal_space(),
                    button(text("Import")).on_press(DirectoryMessage::DirectoryImport.into()),
                    // button(text("Export")).on_press(DirectoryMessage::SelectionExport),
                ]
                // // The row doesn't introspect size automatically, so we have to force it with the calls to width and height
                .padding(10.),
                container(
                    scrollable(values).direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::new().width(2.).scroller_width(10.),
                    ))
                )
            ]
            .width(250),
        )
        .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirectoryData {
    pub directory: Utf8PathBuf,
}

impl DirectoryData {
    pub fn strip_prefix(&self) -> &Utf8Path {
        self.directory
            .strip_prefix(dirs::home_dir().unwrap())
            .unwrap()
    }

    pub fn add_prefix(path: &Utf8PathBuf) -> Utf8PathBuf {
        Utf8Path::from_path(&dirs::home_dir().unwrap())
            .unwrap()
            .join(path)
            .to_owned()
    }
}

impl From<DirectoryData> for String {
    fn from(d: DirectoryData) -> Self {
        d.directory.to_string()
    }
}

impl From<String> for DirectoryData {
    fn from(value: String) -> Self {
        Self {
            directory: Utf8PathBuf::from(value),
        }
    }
}

// impl DirectoryData {
//     pub fn view(&self, selected: bool) -> Element<'_, Message> {
//         let message = if selected {
//             None
//         } else {
//             Some(DirectoryMessage::SelectDirectory(self.clone()).into())
//         };
//         button(text(self.strip_prefix().as_str()).width(Length::Fill))
//             .on_press_maybe(message)
//             .style(directory_style)
//             .into()
//     }
// }
