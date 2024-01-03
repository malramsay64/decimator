use iced::Element;
use iced_aw::{menu_bar, menu_tree, quad, CloseCondition, MenuTree};
use iced_core::Length;
use iced_widget::{button, horizontal_space, row, text, toggler};

use crate::widget::choice;
use crate::{AppData, AppMsg, AppView};

fn separator<'a>() -> MenuTree<'a, AppMsg, iced::Renderer> {
    menu_tree!(quad::Quad {
        color: [0.5; 3].into(),
        border_radius: [4.0; 4],
        inner_bounds: quad::InnerBounds::Ratio(0.98, 0.1),
        ..Default::default()
    })
}

pub fn menu_view(data: &AppData) -> Element<AppMsg> {
    let menu: Element<AppMsg> = menu_bar!(MenuTree::with_children(
        button("Menu"),
        vec![
            MenuTree::new(toggler(
                String::from("Pick"),
                data.thumbnail_view.pick(),
                AppMsg::DisplayPick
            )),
            MenuTree::new(toggler(
                String::from("Ordinary"),
                data.thumbnail_view.ordinary(),
                AppMsg::DisplayOrdinary
            )),
            MenuTree::new(toggler(
                String::from("Ignore"),
                data.thumbnail_view.ignore(),
                AppMsg::DisplayIgnore
            )),
            MenuTree::new(toggler(
                String::from("Hidden"),
                data.thumbnail_view.hidden(),
                AppMsg::DisplayHidden
            )),
            separator(),
            menu_tree!(
                button(text("Generate New Thumbnails")).on_press(AppMsg::UpdateThumbnails(true))
            ),
            menu_tree!(
                button(text("Redo All Thumbnails")).on_press(AppMsg::UpdateThumbnails(false))
            ),
        ]
    )
    .width(400))
    .close_condition(CloseCondition {
        leave: true,
        click_inside: false,
        click_outside: true,
    })
    .into();
    let tabs = row!(
        choice(
            text("Preview").into(),
            AppView::Preview,
            Some(data.app_view),
            AppMsg::SetView
        ),
        choice(
            text("Grid").into(),
            AppView::Grid,
            Some(data.app_view),
            AppMsg::SetView
        ),
    );
    row!(tabs, horizontal_space(Length::Fill), menu)
        .padding(10)
        .align_items(iced::Alignment::Center)
        .into()
}
