use camino::{Utf8Path, Utf8PathBuf};
use iced::widget::{button, text, Container};
use iced::{Border, Color, Element, Length, Padding, Theme};

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

    pub fn add_prefix(path: &Utf8PathBuf) -> Utf8PathBuf {
        Utf8Path::from_path(&dirs::home_dir().unwrap())
            .unwrap()
            .join(path)
            .to_owned()
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

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let palette = style.extended_palette();
        let border = Border {
            color: palette.secondary.base.color,
            width: 1.,
            radius: 0.0.into(),
        };
        button::Appearance {
            background: None,
            text_color: palette.background.base.text,
            border,
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            shadow_offset: active.shadow_offset + iced::Vector::new(0.0, 1.0),
            ..active
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: iced::Vector::default(),
            ..self.active(style)
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            shadow_offset: iced::Vector::default(),
            background: active.background.map(|background| match background {
                iced::Background::Color(color) => iced::Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
                iced::Background::Gradient(gradient) => {
                    iced::Background::Gradient(gradient.mul_alpha(0.5))
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
    pub fn view(&self) -> Element<'_, AppMsg, Theme, iced::Renderer> {
        Container::new(text(self.strip_prefix().as_str()).width(Length::Fill))
            // Top, right, bottom, left
            .padding(Padding::from([0, 10, 0, 10]))
            .align_y(iced::alignment::Vertical::Center)
            .height(Length::Fill)
            .into()
    }
}
