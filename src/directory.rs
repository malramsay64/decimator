use camino::Utf8PathBuf;
use gtk::prelude::*;
use relm4::factory::AsyncFactoryComponent;

use relm4::prelude::DynamicIndex;
use relm4::{gtk, AsyncFactorySender};
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

#[relm4::factory(async, pub)]
impl AsyncFactoryComponent for Directory {
    type Init = Utf8PathBuf;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentInput = AppMsg;
    type ParentWidget = gtk::ListBox;

    view! {
        root = gtk::Box {
            #[name(label)]
            gtk::Label {
                set_width_request: 320,
                #[watch]
                set_label: self.path.as_str(),
            }
        }
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
