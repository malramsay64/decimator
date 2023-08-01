
use iced_widget::core::{Background, Color};
use iced_widget::style::Theme;

/// The appearance of a radio button.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the radio button.
    pub background: Background,
    /// The border width of the radio button.
    pub border_width: f32,
    /// The border [`Color`] of the radio button.
    pub border_color: Color,
    /// The text [`Color`] of the radio button.
    pub text_color: Option<Color>,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            background: Background::Color([0.87, 0.87, 0.87].into()),
            border_width: 1.0,
            border_color: [0.8, 0.8, 0.8].into(),
            text_color: Some(Color::BLACK),
        }
    }
}

/// A set of rules that dictate the style of a Choice button.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default + Copy;

    /// Produces the active [`Appearance`] of a radio button.
    fn active(&self, style: &Self::Style, is_selected: bool) -> Appearance;

    /// Produces the hovered [`Appearance`] of a radio button.
    fn hovered(&self, style: &Self::Style, is_selected: bool) -> Appearance;
}

/// The style of a choice button.
#[derive(Default, Clone, Copy)]
pub enum Choice {
    /// The default style.
    #[default]
    Default,
}

impl StyleSheet for Theme {
    type Style = Choice;

    fn active(&self, style: &Self::Style, is_selected: bool) -> Appearance {
        let mut appearance = Appearance::default();
        match style {
            Choice::Default => {
                appearance.text_color = Some(if is_selected {Color::BLACK} else {Color::WHITE});
            }
        }
        appearance
    }

    fn hovered(&self, style: &Self::Style, is_selected: bool) -> Appearance {
        match style {
            Choice::Default => {
                let active = self.active(style, is_selected);
                let palette = self.extended_palette();

                Appearance {
                    background: palette.primary.weak.color.into(),
                    ..active
                }
            }
        }
    }
}
