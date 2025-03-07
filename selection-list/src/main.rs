use iced::widget::{column, row, text, vertical_space};
use iced::{Application, Element, Length, Settings, Task, Theme};
use selection_list::SelectionList;

#[derive(Debug, Clone)]
enum Message {
    None,
}

#[derive(Debug, Default)]
struct App {
    items: Vec<String>,
}

impl App {
    fn new() -> Self {
        Self {
            items: (1..100).map(|f| format!("Item {f}")).collect(),
        }
    }

    fn title(&self) -> String {
        "Selection List Demo".into()
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<Message> {
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
            SelectionList::new(items_vertical, |_| Message::None,)
                .item_height(30.)
                .item_width(200.)
                .width(200)
                .height(Length::Fill)
                .direction(selection_list::Direction::Vertical),
            column![
                vertical_space(),
                SelectionList::new(items_horizontal, |_| Message::None,)
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
    iced::application("List", App::update, App::view).run()
}
