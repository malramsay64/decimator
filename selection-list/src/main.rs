use iced::widget::{column, row, text, vertical_space};
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

    fn view(&self) -> iced::Element<'_, Self::Message, Theme, iced::Renderer> {
        let items_vertical: Vec<_> = self
            .items
            .iter()
            .map(|i| (i.clone(), text(i).into()))
            .collect();
        let items_horizontal: Vec<_> = self
            .items
            .iter()
            .map(|i| (i.clone(), text(i).into()))
            .collect();
        row![
            SelectionList::new(items_vertical, |_| AppMsg::None,)
                .item_height(30.)
                .item_width(200.)
                .width(200)
                .height(Length::Fill)
                .direction(selection_list::Direction::Vertical),
            column![
                vertical_space(Length::Fill),
                SelectionList::new(items_horizontal, |_| AppMsg::None,)
                    .item_height(60.)
                    .item_width(100.)
                    .height(70)
                    .direction(selection_list::Direction::Horizontal)
            ],
        ]
        .into()
    }
}

fn main() -> Result<(), iced::Error> {
    App::run(Settings {
        ..Default::default()
    })
}
