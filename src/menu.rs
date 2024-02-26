use iced::widget::{button, horizontal_space, row, text, toggler};
use iced::{Border, Element, Theme};
use iced_aw::menu::{Item, Menu};
use iced_aw::native::SegmentedButton;
use iced_aw::{menu_bar, menu_items, quad};

use crate::{AppData, AppMsg, AppView};

/// Helper to generate a separator for the menu
fn separator<'a>() -> Element<'a, AppMsg, Theme, iced::Renderer> {
    quad::Quad {
        quad_border: Border {
            color: [0.5; 3].into(),
            width: 0.,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
    .into()
}

pub fn menu_view(data: &AppData) -> Element<AppMsg> {
    let menu: Element<AppMsg> = menu_bar!((button("Menu"), {
        Menu::new(menu_items!((toggler(
            String::from("Pick"),
            data.thumbnail_view.pick(),
            AppMsg::DisplayPick
        ))(toggler(
            String::from("Ordinary"),
            data.thumbnail_view.ordinary(),
            AppMsg::DisplayOrdinary
        ))(toggler(
            String::from("Ignore"),
            data.thumbnail_view.ignore(),
            AppMsg::DisplayIgnore
        ))(toggler(
            String::from("Hidden"),
            data.thumbnail_view.hidden(),
            AppMsg::DisplayHidden
        ))(separator())(
            button(text("Generate New Thumbnails")).on_press(AppMsg::UpdateThumbnails(true))
        )(
            button(text("Redo All Thumbnails")).on_press(AppMsg::UpdateThumbnails(false))
        )))
    }))
    .width(500)
    .into();
    let tabs = row!(
        SegmentedButton::new(
            text("Preview"),
            AppView::Preview,
            Some(data.app_view),
            AppMsg::SetView
        ),
        SegmentedButton::new(
            text("Grid"),
            AppView::Grid,
            Some(data.app_view),
            AppMsg::SetView
        ),
    );
    row!(tabs, horizontal_space(), menu)
        .padding(10)
        .align_items(iced::Alignment::Center)
        .into()
}
