//! Create choices using Choice buttons.
use iced::widget::{Row, Text};
use iced_core::event::{self, Event};
use iced_core::widget::Tree;
use iced_core::{
    alignment, layout, mouse, renderer, text, touch, Alignment, Clipboard, Element, Layout, Length,
    Pixels, Rectangle, Shell, Widget,
};
use style::StyleSheet;

pub mod style;

pub struct Choice<Message, Renderer = iced::Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    is_selected: bool,
    on_click: Message,
    label: String,
    width: Length,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    text_shaping: text::Shaping,
    font: Option<Renderer::Font>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<Message, Renderer> Choice<Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
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
    pub fn new<F, V>(label: impl Into<String>, value: V, selected: Option<V>, f: F) -> Self
    where
        V: Eq + Copy,
        F: FnOnce(V) -> Message,
    {
        Self {
            is_selected: Some(value) == selected,
            on_click: f(value),
            label: label.into(),
            width: Length::Shrink,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_shaping: text::Shaping::Basic,
            font: None,
            style: Default::default(),
        }
    }

    /// Sets the width of the [`Choice`] button.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the text size of the [`Choice`] button.
    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into().0);
        self
    }

    /// Sets the text [`LineHeight`] of the [`Choice`] button.
    pub fn text_line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`Choice`] button.
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    /// Sets the text font of the [`Choice`] button.
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the style of the [`Choice`] button.
    pub fn style(mut self, style: impl Into<<Renderer::Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Choice<Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet + crate::text::StyleSheet,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        Row::<(), Renderer>::new()
            .width(self.width)
            .align_items(Alignment::Center)
            .push(
                Text::new(&self.label)
                    .width(self.width)
                    .size(self.text_size.unwrap_or_else(|| renderer.default_size()))
                    .line_height(self.text_line_height)
                    .shaping(self.text_shaping),
            )
            .layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
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

                    return event::Status::Captured;
                }
            }
            _ => {}
        }

        event::Status::Ignored
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
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let is_mouse_over = cursor.is_over(layout.bounds());

        let mut children = layout.children();

        let custom_style = if is_mouse_over {
            theme.hovered(&self.style, self.is_selected)
        } else {
            theme.active(&self.style, self.is_selected)
        };

        {
            let label_layout = children.next().unwrap();

            crate::text::draw(
                renderer,
                style,
                label_layout,
                &self.label,
                self.text_size,
                self.text_line_height,
                self.font,
                crate::text::Appearance {
                    color: custom_style.text_color,
                },
                alignment::Horizontal::Left,
                alignment::Vertical::Center,
                self.text_shaping,
            );
        }
    }
}

impl<'a, Message, Renderer> From<Choice<Message, Renderer>> for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + text::Renderer,
    Renderer::Theme: StyleSheet + crate::text::StyleSheet,
{
    fn from(choice: Choice<Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(choice)
    }
}
