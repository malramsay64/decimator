mod choice;
mod viewer;

use choice::Choice;
use viewer::Viewer;

/// Creates a new [`Viewer`] with the given image `Handle`.
pub fn viewer<Handle>(handle: Handle) -> Viewer<Handle> {
    Viewer::new(handle)
}

/// Creates a new [`Choice`].
///
/// [`Choice`]: widget::Choice
pub fn choice<Message, Renderer, V>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_click: impl FnOnce(V) -> Message,
) -> Choice<Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::text::Renderer,
    Renderer::Theme: choice::style::StyleSheet,
    V: Copy + Eq,
{
    Choice::new(label, value, selected, on_click)
}
