mod choice;
mod viewer;

use choice::Choice;
use iced::advanced::text;
use iced::Element;
use viewer::Viewer;

/// Creates a new [`Viewer`] with the given image `Handle`.
pub fn viewer<Handle>(handle: Handle) -> Viewer<Handle> {
    Viewer::new(handle)
}

/// Creates a new [`Choice`].
///
/// [`Choice`]: widget::Choice
pub fn choice<Message, Theme, Renderer, V>(
    label: Element<'_, Message, Theme, Renderer>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> Message,
) -> Choice<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
    Theme: choice::style::StyleSheet,
    V: Copy + Eq,
{
    Choice::new(label, value, selected, on_click)
}
