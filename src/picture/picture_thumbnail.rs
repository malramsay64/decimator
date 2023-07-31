use iced::widget::{button, column, image, row, text};
use iced::Element;

use super::{PictureData, Selection};
use crate::widget::choice;
use crate::AppMsg;

pub type PictureThumbnail = PictureData;

impl Eq for PictureThumbnail {}

impl PartialOrd for PictureThumbnail {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.capture_time?.partial_cmp(&other.capture_time?)
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
            button(column![
                iced::widget::image(image::Handle::from_pixels(
                    thumbnail.width(),
                    thumbnail.height(),
                    thumbnail.to_vec()
                ))
                .width(240)
                .height(240),
                row![
                    choice("I", Selection::Ignore, Some(self.selection), |s| {
                        AppMsg::SetSelection(s)
                    }),
                    choice("O", Selection::Ordinary, Some(self.selection), |s| {
                        AppMsg::SetSelection(s)
                    }),
                    choice("P", Selection::Pick, Some(self.selection), |s| {
                        AppMsg::SetSelection(s)
                    }),
                ]
                .spacing(10)
                .padding(20)
            ])
            .on_press(AppMsg::UpdatePictureView(Some(self.id)))
            .into()
        } else {
            column![text("No image")].into()
        }
    }
}
