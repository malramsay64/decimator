use entity::Selection;
use iced::widget::{button, column, container, horizontal_space, image, pop, row, text, Button};
use iced::Element;

use super::PictureData;
use crate::thumbnail::ThumbnailMessage;
use crate::Message;

/// Defining the data for a thumbnail image
#[derive(Clone, Debug)]
pub struct PictureThumbnail {
    pub handle: Option<image::Handle>,
    pub data: PictureData,
}

impl PartialEq for PictureThumbnail {
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }
}

impl Eq for PictureThumbnail {}

impl PartialOrd for PictureThumbnail {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.data.capture_time?.cmp(&other.data.capture_time?))
    }
}

impl Ord for PictureThumbnail {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.data.capture_time, other.data.capture_time) {
            (Some(s), Some(o)) => s.cmp(&o),
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    }
}

impl PictureThumbnail {
    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let button_width = iced::Length::from(40.0);
        let button_ignore: Element<'a, Message> = button(text("I").center())
            .on_press(ThumbnailMessage::SetSelection((self.data.id, Selection::Ignore)).into())
            .width(button_width)
            .into();

        let button_ordinary: Element<'a, Message> = button(text("O").center())
            .on_press(ThumbnailMessage::SetSelection((self.data.id, Selection::Ordinary)).into())
            .width(button_width)
            .into();

        let button_pick: Element<'a, Message> = Button::new(text("P").center())
            .on_press(ThumbnailMessage::SetSelection((self.data.id, Selection::Pick)).into())
            .width(button_width)
            .into();

        let image_handle: Element<'a, Message> = if let Some(handle) = &self.handle {
            image(handle)
                .width(240)
                .height(240)
                .content_fit(iced::ContentFit::Contain)
                .into()
        } else {
            pop(container(horizontal_space()).width(240).height(240))
                .anticipate(240)
                .on_show(move |_| ThumbnailMessage::ThumbnailPoppedIn(self.data.id).into())
                .into()
        };
        button(
            column![
                image_handle,
                // TODO: Re-enable once supported by iced_aw
                row![button_ignore, button_ordinary, button_pick]
            ]
            .align_x(iced::Alignment::Center)
            .padding(10),
        )
        .on_press(ThumbnailMessage::SetActive(self.data.id).into())
        .into()
    }
}
