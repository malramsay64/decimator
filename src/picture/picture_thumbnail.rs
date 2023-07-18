use iced::widget::image::Handle;
use iced::widget::{column, image, row, text};
use iced::Element;

use super::PictureData;
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
    fn view(&self) -> Element<AppMsg> {
        if let Some(thumbnail) = &self.thumbnail {
            column![
                image::viewer(image::Handle::from_pixels(
                    thumbnail.width(),
                    thumbnail.height(),
                    thumbnail.to_vec()
                ))
                .width(320)
                .height(320),
                row![text(&self.rating), text(&self.selection)]
            ]
            .into()
        } else {
            column![text("No image")].into()
        }
    }
}
