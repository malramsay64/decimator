use std::hash::Hash;

use iced::advanced::renderer;
use iced::advanced::widget::{self};
use iced::widget::scrollable::Properties;
use iced::widget::{container, scrollable, Container, Scrollable};
use iced::{Element, Length};

mod list;
mod style;

pub use list::Direction;
use style::StyleSheet;

pub use crate::list::ListState;

pub struct SelectionList<'a, Label, Message, Renderer = iced::Renderer>
where
    Label: Eq + Hash + Clone,
    Message: Clone,
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet + scrollable::StyleSheet,
{
    width: Length,
    height: Length,
    item_width: f32,
    item_height: f32,
    direction: Direction,
    values: Vec<(Label, Element<'a, Message, Renderer>)>,
    on_selected: Box<dyn Fn(Label) -> Message + 'a>,
    manual_selection: Option<usize>,
    scroll_id: scrollable::Id,
}

impl<'a, Label, Message, Renderer> SelectionList<'a, Label, Message, Renderer>
where
    Label: Eq + Hash + Clone + 'a,
    Message: Clone + 'a,
    Renderer: renderer::Renderer + 'a,
    Renderer::Theme: StyleSheet + scrollable::StyleSheet + container::StyleSheet,
{
    pub fn new(
        values: Vec<(Label, Element<'a, Message, Renderer>)>,
        on_selected: impl Fn(Label) -> Message + 'a,
    ) -> Self {
        Self::new_with_selection(values, on_selected, None)
    }

    pub fn new_with_selection(
        values: Vec<(Label, Element<'a, Message, Renderer>)>,
        on_selected: impl Fn(Label) -> Message + 'a,
        selection: Option<usize>,
    ) -> Self {
        Self {
            width: Length::Shrink,
            height: Length::Shrink,
            item_width: 0.,
            item_height: 0.,
            direction: Direction::Vertical,
            values,
            on_selected: Box::new(on_selected),
            manual_selection: selection,
            scroll_id: scrollable::Id::unique(),
        }
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    #[must_use]
    pub fn item_height(mut self, item_height: f32) -> Self {
        self.item_height = item_height;
        self
    }

    #[must_use]
    pub fn item_width(mut self, item_width: f32) -> Self {
        self.item_width = item_width;
        self
    }

    #[must_use]
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub fn id(mut self, id: scrollable::Id) -> Self {
        self.scroll_id = id;
        self
    }

    pub fn view(self) -> Element<'a, Message, Renderer> {
        let scrollable_direction = match self.direction {
            Direction::Vertical => scrollable::Direction::Vertical(Properties::default()),
            Direction::Horizontal => scrollable::Direction::Horizontal(Properties::default()),
        };
        let container = Container::new(
            Scrollable::new(
                list::List::new(
                    self.values,
                    self.on_selected,
                    self.manual_selection,
                    self.item_width,
                    self.item_height,
                )
                .direction(self.direction),
            )
            .direction(scrollable_direction)
            .id(self.scroll_id.clone()),
        )
        .width(self.width)
        .height(self.height);
        container.into()
    }
}
