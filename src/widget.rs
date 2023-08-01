mod choice;
mod viewer;

use choice::Choice;
use iced::Element;
use viewer::Viewer;

/// Creates a new [`Viewer`] with the given image `Handle`.
pub fn viewer<Handle>(handle: Handle) -> Viewer<Handle> {
    Viewer::new(handle)
}

/// Creates a new [`Choice`].
///
/// [`Choice`]: widget::Choice
pub fn choice<'a, Message, Renderer, V>(
    label: Element<'a, Message, Renderer>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> Message,
) -> Choice<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::text::Renderer,
    Renderer::Theme: choice::style::StyleSheet,
    V: Copy + Eq,
{
    Choice::new(label, value, selected, on_click)
}
