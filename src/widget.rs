mod choice;

use choice::Choice;

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
