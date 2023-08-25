use std::hash::Hash;
use std::marker::PhantomData;

use iced::advanced::layout::Node;
use iced::advanced::widget::Tree;
use iced::advanced::{mouse, renderer, Clipboard, Shell, Widget};
use iced::widget::scrollable::Properties;
use iced::widget::{container, scrollable, Container, Scrollable};
use iced::{event, Element, Event, Length, Rectangle};

mod list;
mod style;

pub use list::Direction;
use style::StyleSheet;

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
        Self {
            width: Length::Shrink,
            height: Length::Shrink,
            item_width: 0.,
            item_height: 0.,
            direction: Direction::Vertical,
            values,
            on_selected: Box::new(on_selected),
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

    pub fn build(self) -> SelectionList<'a, Label, Message, Renderer> {
        let scrollable_direction = match self.direction {
            Direction::Vertical => scrollable::Direction::Vertical(Properties::default()),
            Direction::Horizontal => scrollable::Direction::Horizontal(Properties::default()),
        };
        let container = Container::new(
            Scrollable::new(list::List::new(
                self.values,
                self.on_selected,
                self.item_width,
                self.item_height,
            ))
            .direction(scrollable_direction),
        )
        .width(self.width)
        .height(self.height);
        SelectionList {
            container,
            phantom_data: PhantomData,
            renderer: PhantomData,
            style: Default::default(),
            width: self.width,
            height: self.height,
        }
    }
}

pub struct SelectionList<'a, Label, Message, Renderer = iced::Renderer>
where
    Label: Eq + Hash + Clone,
    Message: Clone,
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet + scrollable::StyleSheet + container::StyleSheet,
{
    container: Container<'a, Message, Renderer>,
    phantom_data: PhantomData<Label>,

    renderer: PhantomData<Renderer>,
    style: <Renderer::Theme as StyleSheet>::Style,
    width: Length,
    height: Length,
}

impl<'a, Label, Message, Renderer> SelectionList<'a, Label, Message, Renderer>
where
    Label: Clone + Hash + Eq,
    Renderer: renderer::Renderer + 'a,
    Message: Clone + 'a,
    Renderer::Theme: StyleSheet + scrollable::StyleSheet + container::StyleSheet,
{
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

impl<'a, Label, Message, Renderer> Widget<Message, Renderer>
    for SelectionList<'a, Label, Message, Renderer>
where
    Label: Eq + Hash + Clone,
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
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let content = self.container.layout(renderer, &limits);
        let size = limits.resolve(content.size());
        Node::with_children(size, vec![content])
    }

    fn draw(
        &self,
        state: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced::advanced::Renderer>::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
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
        layout: iced::advanced::Layout<'_>,
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

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::None
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
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.container
            .mouse_interaction(&state.children[0], layout, cursor, viewport, renderer)
    }
}
impl<'a, Label, Message, Renderer> From<SelectionList<'a, Label, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Label: Eq + Hash + Clone + 'a,
    Renderer: renderer::Renderer + 'a,
    Message: Clone + 'a,
    Renderer::Theme: StyleSheet + container::StyleSheet + scrollable::StyleSheet,
{
    fn from(list: SelectionList<'a, Label, Message, Renderer>) -> Self {
        Self::new(list)
    }
}
