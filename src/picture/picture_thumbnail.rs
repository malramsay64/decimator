use entity::Selection;
use iced::widget::{column, image, row, text};
use iced::Element;
use iced_aw::native::SegmentedButton;

use super::PictureData;
use crate::AppMsg;

/// Defining the data for a thumbnail image
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
        let button_ignore = SegmentedButton::new(
            text("I"),
            Selection::Ignore,
            Some(self.selection),
            AppMsg::SetSelectionCurrent,
        )
        .width(40.0.into());

        let button_ordinary = SegmentedButton::new(
            text("O"),
            Selection::Ordinary,
            Some(self.selection),
            AppMsg::SetSelectionCurrent,
        )
        .width(40.0.into());

        let button_pick = SegmentedButton::new(
            text("P"),
            Selection::Pick,
            Some(self.selection),
            AppMsg::SetSelectionCurrent,
        )
        .width(40.0.into());
        if let Some(thumbnail) = self.thumbnail {
            column![
                iced::widget::image(image::Handle::from_pixels(
                    thumbnail.width(),
                    thumbnail.height(),
                    thumbnail.to_vec()
                ))
                .width(240)
                .height(240),
                row![button_ignore, button_ordinary, button_pick]
            ]
            .align_items(iced::Alignment::Center)
            .padding(10)
            .into()
        } else {
            column![text("No thumbnail")]
                .align_items(iced::Alignment::Center)
                .width(240)
                .height(240)
                .into()
        }
    }
}
