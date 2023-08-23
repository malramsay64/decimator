use std::hash::Hash;
use std::marker::PhantomData;

use iced::advanced::layout::Node;
use iced::advanced::widget::Tree;
use iced::advanced::{mouse, renderer, Clipboard, Shell, Widget};
use iced::widget::{container, scrollable, Container, Scrollable};
use iced::{event, Background, Color, Element, Event, Length, Rectangle, Theme};

mod list;

#[derive(Clone, Copy, Debug, Default)]
#[allow(missing_docs, clippy::missing_docs_in_private_items)]
/// Default Prebuilt ``SelectionList`` Styles
pub enum SelectionListStyles {
    #[default]
    Default,
}

/// The appearance of a menu.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The List Label Text Color
    pub text_color: Color,
    /// The background
    pub background: Background,
    /// The container Border width
    pub border_width: f32,
    /// The container Border color
    pub border_color: Color,
    /// The List Label Text Select Color
    pub hovered_text_color: Color,
    /// The List Label Text Select Background Color
    pub hovered_background: Background,
    /// The List Label Text Select Color
    pub selected_text_color: Color,
    /// The List Label Text Select Background Color
    pub selected_background: Background,
}
impl std::default::Default for Appearance {
    fn default() -> Self {
        Self {
            text_color: Color::BLACK,
            background: Background::Color([0.87, 0.87, 0.87].into()),
            border_width: 1.0,
            border_color: [0.7, 0.7, 0.7].into(),
            hovered_text_color: Color::WHITE,
            hovered_background: Background::Color([0.0, 0.5, 1.0].into()),
            selected_text_color: Color::WHITE,
            selected_background: Background::Color([0.2, 0.5, 0.8].into()),
        }
    }
}

/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    ///Style for the trait to use.
    type Style: std::default::Default + Copy;
    /// Produces the style of a container.
    fn style(&self, style: Self::Style) -> Appearance;
}

impl StyleSheet for Theme {
    type Style = SelectionListStyles;
    fn style(&self, _style: Self::Style) -> Appearance {
        let palette = self.extended_palette();
        let foreground = self.palette();

        Appearance {
            text_color: foreground.text,
            background: palette.background.base.color.into(),
            border_color: foreground.text,
            hovered_text_color: palette.primary.weak.text,
            hovered_background: palette.primary.weak.color.into(),
            selected_text_color: palette.primary.strong.text,
            selected_background: palette.primary.strong.color.into(),
            ..Appearance::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
enum SelectionListMsg {
    #[default]
    None,
    Set(Option<u64>),
}

#[derive(Default, Clone, Copy, Debug)]
enum Order {
    #[default]
    Ascending,
    Descending,
}

pub struct SelectionList<'a, Label, Message, Renderer = iced::Renderer>
where
    Label: Eq + Hash + Clone,
    Message: Clone,
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet + container::StyleSheet,
{
    container: Container<'a, Message, Renderer>,
    selected: Option<Label>,

    renderer: PhantomData<Renderer>,
    style: <Renderer::Theme as StyleSheet>::Style,
    width: Length,
    height: Length,
}

impl<'a, Label, Message, Renderer> SelectionList<'a, Label, Message, Renderer>
where
    Label: Clone + Hash + Eq + 'static,
    Renderer: renderer::Renderer + 'a,
    Message: Clone + 'static,
    Renderer::Theme: StyleSheet + container::StyleSheet + scrollable::StyleSheet,
{
    pub fn new(
        items: Vec<Element<'a, Message, Renderer>>,
        labels: Vec<Label>,
        on_selected: impl Fn(Label) -> Message + 'static,
    ) -> Self
where {
        let container = Container::new(
            Scrollable::new(list::List::new(items, labels, on_selected, 200., 40.))
                .width(Length::Fill),
        );
        Self {
            container,
            selected: None,
            renderer: PhantomData,
            style: <Renderer::Theme as StyleSheet>::Style::default(),
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }
    /// Sets the width of the [`SelectionList`](SelectionList).
    #[must_use]
    pub fn width<L: Into<Length>>(mut self, width: L) -> Self {
        self.width = width.into();
        self
    }
    /// Sets the width of the [`SelectionList`](SelectionList).
    #[must_use]
    pub fn height<L: Into<Length>>(mut self, height: L) -> Self {
        self.height = height.into();
        self
    }
}

impl<'a, Label, Message, Renderer> Widget<Message, Renderer>
    for SelectionList<'a, Label, Message, Renderer>
where
    Label: Eq + Hash + Clone,
    Renderer: renderer::Renderer,
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

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::stateless()
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

    fn operate(
        &self,
        _state: &mut Tree,
        _layout: iced::advanced::Layout<'_>,
        _renderer: &Renderer,
        _operation: &mut dyn iced::advanced::widget::Operation<Message>,
    ) {
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
