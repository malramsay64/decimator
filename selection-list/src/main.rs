use std::collections::HashMap;

use iced::advanced::{layout, renderer, Widget};
use iced::widget::container;
use iced::{Application, Color, Command, Element, Length, Settings, Size, Theme};

enum SelectionListMsg {
    Set(Option<u64>),
}

#[derive(Default, Clone, Copy, Debug)]
enum Order {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Default, Clone)]
struct SelectionList<Message>
where
    Message: Clone,
{
    items: HashMap<u64, String>,
    positions: Vec<u64>,
    selected: Option<u64>,

    on_click: Message,

    sort: Order,
    version: u64,
}

impl<Message, Renderer> Widget<Message, Renderer> for SelectionList
where
    Renderer: renderer::Renderer,
{
    fn width(&self) -> iced::Length {
        Length::Shrink
    }

    fn height(&self) -> iced::Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        // let limits = limits.width(self.width()).height(self.height());
        layout::Node::new(limits.resolve(Size::ZERO))
    }

    fn draw(
        &self,
        state: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced::advanced::Renderer>::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border_radius: 0.0.into(),
                border_width: 5.,
                border_color: Color::from_rgb(1.0, 0.0, 0.0),
            },
            Color::BLACK,
        );
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
}
impl<'a, Message, Renderer> From<SelectionList> for Element<'a, Message, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(list: SelectionList) -> Self {
        Self::new(list)
    }
}

#[derive(Debug)]
enum AppMsg {}

#[derive(Default, Debug)]
struct App {
    selection_list: SelectionList,
}

impl Application for App {
    type Flags = ();
    type Message = AppMsg;
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: Self::Flags) -> (Self, Command<AppMsg>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        "Selection List Demo".into()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        container(self.selection_list.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn main() -> Result<(), iced::Error> {
    App::run(Settings {
        ..Default::default()
    })
}
