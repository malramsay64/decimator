use iced::widget::{button, column, horizontal_space, progress_bar, row, text, toggler, Button};
use iced::{Element, Length};

use crate::thumbnail::ThumbnailMessage;
use crate::{App, AppView, DownloadState, Message};

pub fn menu_view(data: &App) -> Element<'_, Message> {
    let menu: Element<'_, ThumbnailMessage> = column![
        toggler(data.thumbnail_view.pick())
            .label("Pick")
            .on_toggle(ThumbnailMessage::DisplayPick),
        toggler(data.thumbnail_view.ordinary())
            .label("Ordinary")
            .on_toggle(ThumbnailMessage::DisplayOrdinary),
        toggler(data.thumbnail_view.ignore())
            .label("Ignore")
            .on_toggle(ThumbnailMessage::DisplayIgnore),
        toggler(data.thumbnail_view.hidden())
            .label("Hidden")
            .on_toggle(ThumbnailMessage::DisplayHidden),
    ]
    .into();

    let thumbnails = button("Generate Thumbnails").on_press(Message::UpdateThumbnails(false));

    let tabs = row!(
        Button::new(text("Preview")).on_press(Message::SetView(AppView::Preview)),
        Button::new(text("Grid")).on_press(Message::SetView(AppView::Grid)),
        Button::new("Update").on_press(Message::Update),
    )
    .padding(10);
    let tabs = if let DownloadState::Downloading { progress, .. } = data.thumbnail_import {
        tabs.push(progress_bar(0.0..=100.0, progress))
    } else {
        tabs
    };
    row!(
        tabs,
        horizontal_space(),
        thumbnails,
        menu.map(Message::Thumbnail)
    )
    .height(Length::Shrink)
    .width(Length::Fill)
    .padding(10.)
    .align_y(iced::Alignment::Center)
    .into()
}
