use entity::Selection;
use iced::widget::{button, column, image, row, text};
use iced::Element;

use super::PictureData;
use crate::directory::ButtonCustomTheme;
use crate::widget::choice;
use crate::AppMsg;

pub type PictureThumbnail = PictureData;

impl Eq for PictureThumbnail {}

impl PartialOrd for PictureThumbnail {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.capture_time?.cmp(&other.capture_time?))
    }
}

impl Ord for PictureThumbnail {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.capture_time, other.capture_time) {
            (Some(s), Some(o)) => s.cmp(&o),
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    }
}

impl PictureThumbnail {
    pub fn view(self) -> Element<'static, AppMsg> {
        if let Some(thumbnail) = self.thumbnail {
            column![
                iced::widget::image(image::Handle::from_pixels(
                    thumbnail.width(),
                    thumbnail.height(),
                    thumbnail.to_vec()
                ))
                .width(240)
                .height(240),
                row![
                    choice(
                        text("I").into(),
                        Selection::Ignore,
                        Some(self.selection),
                        |s| { AppMsg::SetSelection(s) }
                    )
                    .width(40),
                    choice(
                        text("O").into(),
                        Selection::Ordinary,
                        Some(self.selection),
                        |s| { AppMsg::SetSelection(s) }
                    )
                    .width(40),
                    choice(
                        text("P").into(),
                        Selection::Pick,
                        Some(self.selection),
                        |s| { AppMsg::SetSelection(s) }
                    )
                    .width(40),
                ]
            ]
            .align_items(iced_core::Alignment::Center)
            .padding(10)
            .into()
        } else {
            column![text("No thumbnail")]
                .align_items(iced_core::Alignment::Center)
                .width(240)
                .height(240)
                .into()
        }
    }
}
