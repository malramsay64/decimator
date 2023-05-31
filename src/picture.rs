mod picture_data;
mod picture_grid;
mod picture_preview;
mod picture_thumbnail;
mod property_types;

use camino::Utf8PathBuf;
use gtk::prelude::*;
pub use picture_data::*;
pub use picture_grid::*;
pub use picture_preview::*;
pub use picture_thumbnail::*;
pub use property_types::*;
use relm4::component::{AsyncComponent, AsyncComponentParts};
use relm4::gtk::gdk::Texture;
use relm4::gtk::gdk_pixbuf::Pixbuf;
use relm4::typed_list_view::{TypedListItem, TypedListView};
use relm4::{gtk, tokio, AsyncComponentSender};
use sea_orm::DatabaseConnection;
use walkdir::DirEntry;

use crate::data::update_selection_state;
use crate::AppMsg;

pub fn is_image(entry: &DirEntry) -> bool {
    match entry.path().extension().and_then(|s| s.to_str()) {
        Some("jpg" | "JPG" | "raw" | "RAW" | "ARW" | "arw" | "raf" | "RAF") => true,
        Some("tiff" | "png" | "gif" | "webp" | "heif" | "heic") => false,
        _ => false,
    }
}
