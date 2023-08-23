use iced::{Background, Color, Theme};

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

#[derive(Clone, Copy, Debug, Default)]
#[allow(missing_docs, clippy::missing_docs_in_private_items)]
/// Default Prebuilt ``SelectionList`` Styles
pub enum SelectionListStyles {
    #[default]
    Default,
}
