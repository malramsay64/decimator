use entity::Selection;
use iced::widget::{button, column, container, horizontal_space, image, pop, row, text, Button};
use iced::{Background, Element, Theme};

use super::PictureData;
use crate::thumbnail::ThumbnailMessage;
use crate::Message;

/// Defining the data for a thumbnail image
#[derive(Clone, Debug)]
pub struct PictureThumbnail {
    pub handle: Option<image::Handle>,
    pub data: PictureData,
}

fn thumbnail_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    match status {
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
    }
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
    pub fn view<'a>(&'a self, selected: bool) -> Element<'a, Message> {
        let button_width = iced::Length::from(40.0);
        let buttons = vec![
            ("I", Selection::Ignore),
            ("O", Selection::Ordinary),
            ("P", Selection::Pick),
        ];
        let buttons = buttons.into_iter().map(|(t, selection)| {
            let message = if self.data.selection == selection {
                None
            } else {
                Some(ThumbnailMessage::SetSelection((self.data.id, selection)).into())
            };
            button(text(t).center())
                .on_press_maybe(message)
                .width(button_width)
                .into()
        });
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
        let message: Option<Message> = if selected {
            None
        } else {
            Some(ThumbnailMessage::SetActive(self.data.id).into())
        };
        button(
            column![image_handle, row(buttons),]
                .align_x(iced::Alignment::Center)
                .padding(10),
        )
        .style(thumbnail_style)
        .on_press_maybe(message)
        .into()
    }
}
