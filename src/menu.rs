use iced::widget::{Button, button, horizontal_rule, horizontal_space, row, text, toggler};
use iced::{Border, Color, Element, Length, Renderer, Theme};
// use iced_aw::menu::{self, Item, Menu};
// use iced_aw::{menu_bar, menu_items, quad};

use crate::{App, AppView, Message};

/// Helper to generate a separator for the menu
fn separator<'a>() -> Element<'a, Message, Theme, iced::Renderer> {
    // quad::Quad {
    //     quad_color: Color::from([0.5; 3]).into(),
    //     quad_border: Border {
    //         radius: 4.0.into(),
    //         ..Default::default()
    //     },
    //     height: Length::Fixed(5.0),
    //     ..Default::default()
    // }
    // .into()
    horizontal_rule(4).into()
}

pub fn menu_view(data: &App) -> Element<'_, Message, Theme, Renderer> {
    // let menu_template = |items| Menu::new(items).spacing(5.0);

    // #[rustfmt::skip]
    // let menu = menu_bar!(
    //     (button("Menu"), { menu_template(menu_items!(
    //         (toggler(data.thumbnail_view.pick())
    //             .label(String::from("Pick"))
    //             .on_toggle(Message::DisplayPick)
    //         )
    //         (toggler(data.thumbnail_view.ordinary()).label(
    //             String::from("Ordinary")).on_toggle(
    //             Message::DisplayOrdinary

    //         ))
    //         (toggler(data.thumbnail_view.ignore()).label(
    //             String::from("Ignore"))
    //             .on_toggle(
    //             Message::DisplayIgnore
    //         ))
    //         (toggler(
    //             data.thumbnail_view.hidden()).label(
    //             String::from("Hidden")).on_toggle(
    //             Message::DisplayHidden
    //         ))
    //         (separator())
    //         (button(text("Generate New Thumbnails")).on_press(Message::UpdateThumbnails(true)))
    //         (button(text("Redo All Thumbnails")).on_press(Message::UpdateThumbnails(false)))
    //     )).width(240.)
    //     })
    // )
    // .draw_path(menu::DrawPath::Backdrop);

    let tabs = row!(
        Button::new(text("Preview")).on_press(Message::SetView(AppView::Preview)),
        Button::new(text("Grid")).on_press(Message::SetView(AppView::Grid)),
    );
    row!(tabs, horizontal_space())
        .height(Length::Shrink)
        .width(Length::Fill)
        .padding(10.)
        .align_y(iced::Alignment::Center)
        .into()
}
