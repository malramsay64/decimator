//! Create choices using Choice buttons.
use iced::subscription::events;
use iced_core::event::{self, Event};
use iced_core::layout::Node;
use iced_core::mouse::Cursor;
use iced_core::widget::{tree, Tree};
use iced_core::{
    layout, mouse, renderer, text, touch, Alignment, Clipboard, Color, Element, Layout, Length,
    Point, Rectangle, Shell, Widget,
};
use style::StyleSheet;

pub mod style;

/// The ratio of the border radius.
const BORDER_RADIUS_RATIO: f32 = 10.;

pub struct Choice<'a, Message, Renderer = iced::Renderer>
where
    Renderer: iced_core::Renderer,
    Renderer::Theme: StyleSheet,
{
    content: Element<'a, Message, Renderer>,
    /// The padding of the [`Badge`].
    padding: u16,
    /// The width of the [`Badge`].
    width: Length,
    /// The height of the [`Badge`].
    height: Length,
    /// The horizontal alignment of the [`Badge`](Badge).
    horizontal_alignment: Alignment,
    /// The vertical alignment of the [`Badge`](Badge).
    vertical_alignment: Alignment,
    /// The style of the [`Badge`](Badge).
    style: <Renderer::Theme as StyleSheet>::Style,
    /// The content [`Element`](iced_native::Element) of the [`Badge`](Badge).
    is_selected: bool,
    on_click: Message,
}

impl<'a, Message, Renderer> Choice<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a new [`Choice`] button.
    ///
    /// It expects:
    ///   * the value related to the [`Choice`] button
    ///   * the label of the [`Choice`] button
    ///   * the current selected value
    ///   * a function that will be called when the [`Choice`] is selected. It
    ///   receives the value of the radio and must produce a `Message`.
    /// Creates a new [`Badge`](Badge) with the given content.
    ///
    /// It expects:
    ///     * the content [`Element`](iced_native::Element) to display in the [`Badge`](Badge).
    pub fn new<T, V, F>(content: T, value: V, selected: Option<V>, f: F) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
        V: Eq + Copy,
        F: FnOnce(V) -> Message,
    {
        Self {
            padding: 7,
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: Alignment::Center,
            vertical_alignment: Alignment::Center,
            style: <Renderer::Theme as StyleSheet>::Style::default(),
            content: content.into(),
            is_selected: Some(value) == selected,
            on_click: f(value),
        }
    }

    /// Sets the width of the [`Choice`] button.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the style of the [`Choice`] button.
    pub fn style(mut self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }
}
/// The local state of a [`Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_pressed: bool,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        State::default()
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Choice<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet + crate::text::StyleSheet,
{
    fn width(&self) -> Length {
        self.width
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let padding = self.padding.into();
        let limits = limits
            .loose()
            .width(self.width)
            .height(self.height)
            .pad(padding);

        let mut content = self.content.as_widget().layout(renderer, &limits.loose());
        let size = limits.resolve(content.size());

        content.move_to(Point::new(padding.left, padding.top));
        content.align(self.horizontal_alignment, self.vertical_alignment, size);

        Node::with_children(size.pad(padding), vec![content])
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(layout.bounds()) {
                    shell.publish(self.on_click.clone());

                    event::Status::Captured
                } else {
                    event::Status::Ignored
                }
            }
            _ => event::Status::Ignored,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let mut children = layout.children();
        let is_mouse_over = bounds.contains(cursor.position().unwrap_or_default());
        let style_sheet = if is_mouse_over {
            theme.hovered(&self.style, self.is_selected)
        } else {
            theme.active(&self.style, self.is_selected)
        };

        //println!("height: {}", bounds.height);
        // 34 15
        //  x
        let border_radius = style_sheet
            .border_radius
            .unwrap_or(bounds.height / BORDER_RADIUS_RATIO);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: border_radius.into(),
                border_width: style_sheet.border_width,
                border_color: style_sheet.border_color.unwrap_or(Color::BLACK),
            },
            style_sheet.background,
        );

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style_sheet.text_color,
            },
            children
                .next()
                .expect("Graphics: Layout should have a children layout for Badge"),
            cursor,
            viewport,
        );
    }
}

impl<'a, Message, Renderer> From<Choice<'a, Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + text::Renderer,
    Renderer::Theme: StyleSheet + crate::text::StyleSheet,
{
    fn from(choice: Choice<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(choice)
    }
}
