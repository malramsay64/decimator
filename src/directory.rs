use camino::Utf8PathBuf;
use gtk::prelude::*;
use relm4::prelude::DynamicIndex;
use relm4::typed_list_view::RelmListItem;
use relm4::{gtk, view, AsyncFactorySender};

use crate::AppMsg;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirectoryData {
    pub directory: Utf8PathBuf,
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

pub struct Widgets {
    directory: gtk::Label,
}

impl RelmListItem for DirectoryData {
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        view! {
            root = gtk::Box {
                #[name(directory)]
                gtk::Label {
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                }
            }
        }

        let widgets = Widgets { directory };
        (root, widgets)
    }

    #[tracing::instrument(name = "Binding Widget", level = "trace", skip(widgets, _root))]
    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let Widgets { directory } = widgets;

        directory.set_label(&format!("{}", self.directory));
    }

    fn unbind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {}
}
