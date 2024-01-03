use std::hash::Hash;
use std::marker::PhantomData;

use iced::advanced::layout::Node;
use iced::advanced::widget::{self, Tree};
use iced::advanced::{self, mouse, renderer, Clipboard, Shell, Widget};
use iced::widget::scrollable::Properties;
use iced::widget::{container, scrollable, Container, Scrollable};
use iced::{event, Element, Event, Length, Rectangle};

mod list;
mod style;

pub use list::Direction;
use style::StyleSheet;

pub use crate::list::ListState;

pub struct SelectionListBuilder<'a, Label, Message, Renderer = iced::Renderer>
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
    scroll_id: widget::Id,
}

impl<'a, Label, Message, Renderer> SelectionListBuilder<'a, Label, Message, Renderer>
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
            scroll_id: widget::Id::unique(),
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
    pub fn id(mut self, id: widget::Id) -> Self {
        self.scroll_id = id;
        self
    }

    pub fn build(self) -> SelectionList<'a, Message, Renderer> {
        let scrollable_direction = match self.direction {
            Direction::Vertical => scrollable::Direction::Vertical(Properties::default()),
            Direction::Horizontal => scrollable::Direction::Horizontal(Properties::default()),
        };
        let scroll_id = scrollable::Id::unique();
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
            .id(scroll_id.clone()),
        )
        .width(self.width)
        .height(self.height);
        SelectionList {
            scroll_id,
            container,
            renderer: PhantomData,
            style: Default::default(),
            width: self.width,
            height: self.height,
        }
    }
}

pub struct SelectionList<'a, Message, Renderer = iced::Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet + scrollable::StyleSheet + container::StyleSheet,
{
    scroll_id: scrollable::Id,
    container: Container<'a, Message, Renderer>,

    renderer: PhantomData<Renderer>,
    style: <Renderer::Theme as StyleSheet>::Style,
    width: Length,
    height: Length,
}

impl<'a, Message, Renderer> SelectionList<'a, Message, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Message: Clone + 'a,
    Renderer::Theme: StyleSheet + scrollable::StyleSheet + container::StyleSheet,
{
    pub fn id(&self) -> widget::Id {
        self.scroll_id.clone().into()
    }

    /// Sets the width of the [`SelectionList`](SelectionList).
    #[must_use]
    pub fn width<L: Into<Length>>(mut self, width: L) -> Self {
        let width = width.into();
        self.width = width;
        self.container = self.container.width(width);
        self
    }

    /// Sets the height of the [`SelectionList`](SelectionList).
    #[must_use]
    pub fn height<L: Into<Length>>(mut self, height: L) -> Self {
        let height = height.into();
        self.height = height;
        self.container = self.container.height(height);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for SelectionList<'a, Message, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Message: Clone,
    Renderer::Theme: StyleSheet + container::StyleSheet + scrollable::StyleSheet,
{
    fn width(&self) -> iced::Length {
        Length::Shrink
    }

    fn height(&self) -> iced::Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &advanced::layout::Limits,
    ) -> advanced::layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let content = self.container.layout(renderer, &limits);
        let size = limits.resolve(content.size());
        Node::with_children(size, vec![content])
    }

    fn draw(
        &self,
        state: &widget::Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as advanced::Renderer>::Theme,
        style: &advanced::renderer::Style,
        layout: advanced::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border_color: theme.style(self.style).border_color,
                border_width: theme.style(self.style).border_width,
                border_radius: (0.0).into(),
            },
            theme.style(self.style).background,
        );

        self.container.draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout
                .children()
                .next()
                .expect("Scrollable Child Missing in Selection List"),
            cursor,
            &layout.bounds(),
        );
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: advanced::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.container.on_event(
            &mut state.children[0],
            event,
            layout
                .children()
                .next()
                .expect("Scrollable Child Missing in Selection List"),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::None
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.container as &dyn Widget<_, _>)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.container as &dyn Widget<_, _>]);
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: advanced::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.container
            .mouse_interaction(&state.children[0], layout, cursor, viewport, renderer)
    }
}

impl<'a, Message, Renderer> From<SelectionList<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Message: Clone + 'a,
    Renderer::Theme: StyleSheet + container::StyleSheet + scrollable::StyleSheet,
{
    fn from(list: SelectionList<'a, Message, Renderer>) -> Self {
        Self::new(list)
    }
}
