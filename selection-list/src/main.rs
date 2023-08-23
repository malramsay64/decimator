use iced::widget::{horizontal_space, row, text};
use iced::{Application, Command, Length, Settings, Theme};
use selection_list::SelectionList;

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
        row![
            SelectionList::new(
                self.items.clone(),
                self.items.iter().enumerate().map(|(i, _)| i).collect(),
                |_| AppMsg::None,
                |t| text(format!("{t}")).into()
            )
            .width(400)
            .height(Length::Fill),
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
