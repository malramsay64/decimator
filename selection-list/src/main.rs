use iced::widget::{horizontal_space, row, text};
use iced::{Application, Command, Length, Settings, Theme};
use selection_list::SelectionListBuilder;

#[derive(Debug, Clone)]
enum AppMsg {
    None,
}

struct App {
    items: Vec<String>,
}

impl App {
    fn new() -> Self {
        Self {
            items: (1..100).map(|f| format!("Item {f}")).collect(),
        }
    }
}

impl Application for App {
    type Flags = ();
    type Message = AppMsg;
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: Self::Flags) -> (Self, Command<AppMsg>) {
        (Self::new(), Command::none())
    }

    fn title(&self) -> String {
        "Selection List Demo".into()
    }

    fn update(&mut self, _message: Self::Message) -> iced::Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let items: Vec<_> = self
            .items
            .iter()
            .map(|i| (i.clone(), text(i).into()))
            .collect();
        row![
            SelectionListBuilder::new(items, |_| AppMsg::None,)
                .item_height(30.)
                .item_width(200.)
                .width(200)
                .height(Length::Fill)
                .direction(selection_list::Direction::Vertical)
                .build(),
            horizontal_space(Length::Fill)
        ]
        .into()
    }
}

fn main() -> Result<(), iced::Error> {
    App::run(Settings {
        ..Default::default()
    })
}
