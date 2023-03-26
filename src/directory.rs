use camino::Utf8PathBuf;
use gtk::prelude::*;
use relm4::factory::AsyncFactoryComponent;
use relm4::loading_widgets::LoadingWidgets;
use relm4::prelude::DynamicIndex;
use relm4::{gtk, view, AsyncFactorySender};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};

use crate::AppMsg;

pub struct DirectoryData {
    directory: String,
}

impl From<DirectoryData> for String {
    fn from(d: DirectoryData) -> Self {
        d.directory
    }
}
impl FromRow<'_, SqliteRow> for DirectoryData {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let directory: &str = row.try_get("directory")?;
        Ok(Self {
            directory: directory.to_owned(),
        })
    }
}

#[derive(Debug)]
pub struct Directory {
    pub path: Utf8PathBuf,
}

#[derive(Debug)]
pub enum DirectoryOutput {
    Select(DynamicIndex),
}

#[relm4::factory(async, pub)]
impl AsyncFactoryComponent for Directory {
    type Init = Utf8PathBuf;
    type Input = ();
    type Output = DirectoryOutput;
    type CommandOutput = ();
    type ParentInput = AppMsg;
    type ParentWidget = gtk::Box;

    view! {
        root = gtk::Box {
            #[name(label)]
            gtk::Button {
                set_width_request: 320,
                #[watch]
                set_label: self.path.as_str(),
                connect_clicked[sender, index] => move |_| {
                    sender.output(DirectoryOutput::Select(index.clone()))
                }
            }
        }
    }

    fn init_loading_widgets(root: &mut Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local_ref]
            root {
                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_halign: gtk::Align::Center,
                    set_height_request: 34,
                }
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    fn output_to_parent_input(output: Self::Output) -> Option<AppMsg> {
        Some(match output {
            DirectoryOutput::Select(index) => AppMsg::SelectDirectory(index),
        })
    }

    async fn init_model(
        path: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self { path }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        println!("Directory with path {} was destroyed", self.path);
    }
}
