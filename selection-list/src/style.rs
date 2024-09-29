use iced::{Background, Color, Theme};

/// The Status of a widget event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// can be pressed.
    Active,
    /// can be pressed and it is being hovered.
    Hovered,
    /// is being pressed.
    Pressed,
    /// cannot be pressed.
    Disabled,
    /// is focused.
    Focused,
    /// is Selected.
    Selected,
}

/// The style function of widget.
pub type StyleFn<'a, Theme, Style> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

/// The appearance of a menu.
#[derive(Debug, Clone, Copy)]
pub struct Style {
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

impl std::default::Default for Style {
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

// /// A set of rules that dictate the style of a container.
// pub trait StyleSheet {
//     ///Style for the trait to use.
//     type Style: std::default::Default + Copy;
//     /// Produces the style of a container.
//     fn style(&self, style: Self::Style) -> Style;
// }

// impl StyleSheet for Theme {
//     type Style = SelectionListStyles;
//     fn style(&self, _style: Self::Style) -> Style {
//         let palette = self.extended_palette();
//         let foreground = self.palette();

//         Style {
//             text_color: foreground.text,
//             background: palette.background.base.color.into(),
//             border_color: foreground.text,
//             hovered_text_color: palette.primary.weak.text,
//             hovered_background: palette.primary.weak.color.into(),
//             selected_text_color: palette.primary.strong.text,
//             selected_background: palette.primary.strong.color.into(),
//             ..Style::default()
//         }
//     }
// }

// #[derive(Clone, Copy, Debug, Default)]
// #[allow(missing_docs, clippy::missing_docs_in_private_items)]
// /// Default Prebuilt ``SelectionList`` Styles
// pub enum SelectionListStyles {
//     #[default]
//     Default,
// }

/// The Catalog of a [`SelectionList`](crate::SelectionList).
pub trait Catalog {
    ///Style for the trait to use.
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self, Style>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(primary)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The primary theme of a [`Badge`](crate::widget::selection_list::SelectionList).
#[must_use]
pub fn primary(_theme: &Theme, status: Status) -> Style {
    let base = Style::default();

    match status {
        Status::Hovered => Style {
            text_color: Color::WHITE,
            background: Background::Color([0.0, 0.5, 1.0].into()),
            ..base
        },
        Status::Selected => Style {
            text_color: Color::WHITE,
            background: Background::Color([0.2, 0.5, 0.8].into()),
            ..base
        },
        _ => base,
    }
}
