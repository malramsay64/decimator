use camino::Utf8PathBuf;
use iced::widget::{button, text};
use iced::Element;

use crate::AppMsg;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirectoryData {
    pub directory: Utf8PathBuf,
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
    pub fn view(&self) -> Element<AppMsg> {
        button(text(self.directory.clone().into_string()))
            .on_press(AppMsg::SelectDirectory(self.directory.clone()))
            .width(240)
            .into()
    }
}
