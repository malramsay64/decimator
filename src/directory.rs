use camino::{Utf8Path, Utf8PathBuf};
use iced::advanced::renderer;
use iced::widget::{button, text};
use iced::Element;
use iced_core::{Color, Length};
use iced_style::button::Appearance;
use iced_style::{theme, Theme};

use crate::AppMsg;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirectoryData {
    pub directory: Utf8PathBuf,
}

impl DirectoryData {
    pub fn strip_prefix(&self) -> &Utf8Path {
        self.directory
            .strip_prefix(dirs::home_dir().unwrap())
            .unwrap()
    }
}

impl From<DirectoryData> for String {
    fn from(d: DirectoryData) -> Self {
        d.directory.to_string()
    }
}

impl From<String> for DirectoryData {
    fn from(value: String) -> Self {
        Self {
            directory: Utf8PathBuf::from(value),
        }
    }
}

#[derive(Default)]
pub struct ButtonCustomTheme;

impl button::StyleSheet for ButtonCustomTheme {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> Appearance {
        let palette = style.extended_palette();
        Appearance {
            background: None,
            text_color: palette.background.base.text,
            border_radius: 0.0.into(),
            border_width: 1.,
            border_color: palette.secondary.base.color,
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> Appearance {
        let active = self.active(style);

        Appearance {
            shadow_offset: active.shadow_offset + iced_core::Vector::new(0.0, 1.0),
            ..active
        }
    }

    fn pressed(&self, style: &Self::Style) -> Appearance {
        Appearance {
            shadow_offset: iced_core::Vector::default(),
            ..self.active(style)
        }
    }

    fn disabled(&self, style: &Self::Style) -> Appearance {
        let active = self.active(style);

        Appearance {
            shadow_offset: iced_core::Vector::default(),
            background: active.background.map(|background| match background {
                iced_core::Background::Color(color) => iced_core::Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
                iced_core::Background::Gradient(gradient) => {
                    iced_core::Background::Gradient(gradient.mul_alpha(0.5))
                }
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }
}

impl DirectoryData {
    pub fn view(&self) -> Element<'_, AppMsg, iced::Renderer<Theme>> {
        button(text(self.strip_prefix().as_str()))
            .style(theme::Button::custom(ButtonCustomTheme))
            .on_press(AppMsg::SelectDirectory(self.directory.clone()))
            .width(Length::Fill)
            .into()
    }

    pub fn as_view<'a>(self) -> Element<'a, AppMsg, iced::Renderer<Theme>> {
        button(text(self.strip_prefix().as_str()))
            .style(theme::Button::custom(ButtonCustomTheme))
            .on_press(AppMsg::SelectDirectory(self.directory.clone()))
            .width(Length::Fill)
            .into()
    }
}
