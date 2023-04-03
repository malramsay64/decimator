mod filter;
mod picture_data;
mod picture_thumbnail;
mod property_types;
mod sort;
use std::collections::HashMap;

use filter::*;
use gtk::prelude::*;
pub use picture_data::*;
pub use picture_thumbnail::*;
pub use property_types::*;
use relm4::component::{AsyncComponent, AsyncComponentParts};
use relm4::factory::{AsyncFactoryComponent, AsyncFactoryVecDeque};
use relm4::gtk::gdk::Texture;
use relm4::gtk::gdk_pixbuf::Pixbuf;
use relm4::prelude::*;
use relm4::{gtk, AsyncComponentSender};
pub use sort::*;

use uuid::Uuid;
use walkdir::DirEntry;



use crate::AppMsg;

pub fn is_image(entry: &DirEntry) -> bool {
    match entry.path().extension().and_then(|s| s.to_str()) {
        Some("jpg" | "JPG" | "raw" | "RAW" | "ARW" | "arw" | "raf" | "RAF") => true,
        Some("tiff" | "png" | "gif" | "webp" | "heif" | "heic") => false,
        _ => false,
    }
}

#[derive(Debug)]
pub enum PictureViewMsg {
    SelectPictures(Vec<PictureData>),
    SelectPreview(Option<i32>),
    FilterTogglePick,
    FilterToggleOrdinary,
    FilterToggleIgnore,
    SelectionPick,
    SelectionOrdinary,
    SelectionIgnore,
}

#[derive(Debug)]
pub struct PictureView {
    thumbnails: HashMap<Uuid, PictureData>,
    shown_thumbnails: AsyncFactoryVecDeque<PictureThumbnail>,
    preview_id: Option<Uuid>,
    preview_image: Option<Texture>,
    filter_settings: FilterSettings,
    sort_settings: SortSettings,
}

impl PictureView {
    fn get_thumbnails(&self) -> Vec<PictureData> {
        let mut values = self
            .thumbnails
            .values()
            .filter(|v| self.filter_settings.filter(v))
            .map(|v| v.to_owned())
            .collect::<Vec<_>>();
        values.sort_by(|a, b| self.sort_settings.sort(a, b));
        values
    }

    async fn update_visible(&mut self) {
        // This needs to be first due to the mutable borrow of the thumbnail guard
        let thumbnails = self.get_thumbnails();
        let mut thumbnail_guard = self.shown_thumbnails.guard();
        thumbnail_guard.clear();
        for pic in thumbnails {
            thumbnail_guard.push_back(pic.clone());
        }
    }
}

#[relm4::component(async, pub)]
impl AsyncComponent for PictureView {
    type Init = ();
    type Input = PictureViewMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            gtk::Box {
                set_vexpand: true,
                set_hexpand: true,
                gtk::Picture {
                    #[watch]
                    set_paintable: model.preview_image.as_ref(),

                }
            },
            gtk::ScrolledWindow {
                set_propagate_natural_width: true,
                set_has_frame: true,

                #[local_ref]
                thumbnail_list -> gtk::ListBox {
                    set_width_request: 260,
                    set_show_separators: true,
                    set_selection_mode: gtk::SelectionMode::Single,

                    connect_row_selected[sender] => move |_, row| {
                        let index = row.map(|r| r.index());
                        sender.input(PictureViewMsg::SelectPreview(index));
                    }
                }
            }
        }
    }

    async fn init(
        _: (),
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let shown_thumbnails =
            AsyncFactoryVecDeque::new(gtk::ListBox::default(), sender.input_sender());

        let model = Self {
            thumbnails: Default::default(),
            shown_thumbnails,
            preview_id: Default::default(),
            preview_image: Default::default(),
            filter_settings: Default::default(),
            sort_settings: Default::default(),
        };
        let thumbnail_list = model.shown_thumbnails.widget();
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            PictureViewMsg::SelectPictures(pictures) => {
                self.thumbnails = pictures.into_iter().map(|pic| (pic.id, pic)).collect();
                self.update_visible().await;
            }
            PictureViewMsg::SelectPreview(index) => {
                if let Some(pic) = index.and_then(|i| self.shown_thumbnails.get(i as usize)) {
                    self.preview_id = Some(pic.picture.id);
                    let filepath = pic.picture.filepath.clone();
                    self.preview_image = Some(
                        relm4::spawn(async move {
                            let image = Pixbuf::from_file(filepath)
                                .expect("Image not found.")
                                .apply_embedded_orientation()
                                .expect("Unable to apply orientation.");
                            Texture::for_pixbuf(&image)
                        })
                        .await
                        .unwrap(),
                    );
                } else {
                    self.preview_id = None;
                    self.preview_image = None;
                }
            }
            PictureViewMsg::FilterTogglePick => {
                self.filter_settings.toggle_pick();
                self.update_visible().await;
            }
            PictureViewMsg::FilterToggleOrdinary => {
                self.filter_settings.toggle_ordinary();
                self.update_visible().await;
            }
            PictureViewMsg::FilterToggleIgnore => {
                self.filter_settings.toggle_ignore();
                self.update_visible().await;
            }
            PictureViewMsg::SelectionPick => {
                if let Some(index) = self.preview_id && let Some(pic) = self.thumbnails.get_mut(&index) {
                    pic.selection = Selection::Pick;
                }
            }
            PictureViewMsg::SelectionOrdinary => {
                if let Some(index) = self.preview_id && let Some(pic) = self.thumbnails.get_mut(&index) {
                    pic.selection = Selection::Ordinary;
                }
            }
            PictureViewMsg::SelectionIgnore => {
                if let Some(index) = self.preview_id && let Some(pic) = self.thumbnails.get_mut(&index) {
                    pic.selection = Selection::Ignore;
                }
            }
        }
    }
}
