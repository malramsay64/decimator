use entity::Selection;
use iced::Element;
use iced::widget::{Button, column, image, row, text};

use super::PictureData;
use crate::Message;

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
    pub fn view(self) -> Element<'static, Message> {
        let button_width = iced::Length::from(40.0);
        let button_ignore: Element<'_, Message> = Button::new(text("I"))
            .on_press(Message::SetSelectionCurrent(Selection::Ignore))
            .width(button_width)
            .into();

        let button_ordinary: Element<'_, Message> = Button::new(text("O"))
            .on_press(Message::SetSelectionCurrent(Selection::Ordinary))
            .width(button_width)
            .into();

        let button_pick: Element<'_, Message> = Button::new(text("P"))
            .on_press(Message::SetSelectionCurrent(Selection::Pick))
            .width(button_width)
            .into();
        if let Some(thumbnail) = self.thumbnail {
            column![
                iced::widget::image::Image::new(image::Handle::from_rgba(
                    thumbnail.width(),
                    thumbnail.height(),
                    thumbnail.to_vec()
                ))
                .width(240)
                .height(240),
                // TODO: Re-enable once supported by iced_aw
                row![button_ignore, button_ordinary, button_pick]
            ]
            .align_x(iced::Alignment::Center)
            .padding(10)
            .into()
        } else {
            column![text("No thumbnail")]
                .align_x(iced::Alignment::Center)
                .width(240)
                .height(240)
                .into()
        }
    }
}
