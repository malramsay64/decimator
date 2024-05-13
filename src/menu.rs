use iced::widget::{button, horizontal_space, row, text, toggler};
use iced::{Border, Color, Element, Length, Renderer, Theme};
use iced_aw::menu::{self, Item, Menu, MenuBar};
use iced_aw::widgets::SegmentedButton;
use iced_aw::{menu_bar, menu_items, quad};

use crate::{AppData, AppMsg, AppView};

/// Helper to generate a separator for the menu
fn separator<'a>() -> Element<'a, AppMsg, Theme, iced::Renderer> {
    quad::Quad {
        quad_color: Color::from([0.5; 3]).into(),
        quad_border: Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        height: Length::Fixed(5.0),
        ..Default::default()
    }
    .into()
}

pub fn menu_view(data: &AppData) -> Element<'_, AppMsg, Theme, Renderer> {
    let menu_template = |items| Menu::new(items).spacing(5.0);

    #[rustfmt::skip]
    let menu = menu_bar!(
        (button("Menu"), { menu_template(menu_items!(
            (toggler(
                String::from("Pick"),
                data.thumbnail_view.pick(),
                AppMsg::DisplayPick
            ))
            (toggler(
                String::from("Ordinary"),
                data.thumbnail_view.ordinary(),
                AppMsg::DisplayOrdinary
            ))
            (toggler(
                String::from("Ignore"),
                data.thumbnail_view.ignore(),
                AppMsg::DisplayIgnore
            ))
            (toggler(
                String::from("Hidden"),
                data.thumbnail_view.hidden(),
                AppMsg::DisplayHidden
            ))
            (separator())
            (button(text("Generate New Thumbnails")).on_press(AppMsg::UpdateThumbnails(true)))
            (button(text("Redo All Thumbnails")).on_press(AppMsg::UpdateThumbnails(false)))
        )).width(240.)
        })
    )
    .draw_path(menu::DrawPath::Backdrop);

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
        .height(Length::Shrink)
        .width(Length::Fill)
        .padding(10.)
        .align_items(iced::Alignment::Center)
        .into()
}
