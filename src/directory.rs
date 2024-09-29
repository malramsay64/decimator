use camino::{Utf8Path, Utf8PathBuf};
use iced::widget::{text, Container};
use iced::{Element, Length, Padding, Theme};

use crate::Message;

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

impl DirectoryData {
    pub fn view(&self) -> Element<'_, Message, Theme, iced::Renderer> {
        Container::new(text(self.strip_prefix().as_str()).width(Length::Fill))
            // Top, right, bottom, left
            .padding(Padding::from([0, 10]))
            .align_y(iced::alignment::Vertical::Center)
            .height(Length::Fill)
            .into()
    }
}
