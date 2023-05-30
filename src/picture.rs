mod picture_data;
mod picture_thumbnail;
mod property_types;

use camino::Utf8PathBuf;
use gtk::prelude::*;
pub use picture_data::*;
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

#[derive(Debug)]
pub enum PictureViewMsg {
    SelectPictures(Vec<PictureData>),
    SelectPreview(Option<u32>),
    FilterPick(bool),
    FilterOrdinary(bool),
    FilterIgnore(bool),
    FilterHidden(bool),
    SelectionPick,
    SelectionOrdinary,
    SelectionIgnore,
    SelectionExport(Utf8PathBuf),
    ImageNext,
    ImagePrev,
}

#[derive(Debug)]
pub struct PictureView {
    thumbnails: TypedListView<PictureThumbnail, gtk::SingleSelection>,
    preview_image: Option<Texture>,
    database: DatabaseConnection,
}

impl PictureView {
    pub fn get_selected(&self) -> Option<TypedListItem<PictureThumbnail>> {
        let index = self.thumbnails.selection_model.selected();
        self.thumbnails.get_visible(index)
    }
}

#[relm4::component(async, pub)]
impl AsyncComponent for PictureView {
    type Init = DatabaseConnection;
    type Input = PictureViewMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            gtk::Box {
                set_vexpand: true,
                set_hexpand: true,
                gtk::Picture {
                    #[watch]
                    set_paintable: model.preview_image.as_ref(),
                    set_halign: gtk::Align::Center,
                    set_hexpand: true,

                }
            },
            gtk::ScrolledWindow {
                set_propagate_natural_height: true,
                set_has_frame: true,

                #[local_ref]
                thumbnail_list -> gtk::ListView {
                    set_height_request: 260,
                    set_show_separators: true,
                    set_enable_rubberband: true,
                    set_orientation: gtk::Orientation::Horizontal,
                }
            }
        }
    }

    async fn init(
        database: DatabaseConnection,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut thumbnails: TypedListView<PictureThumbnail, gtk::SingleSelection> =
            TypedListView::with_sorting();

        thumbnails.add_filter(|item| item.selection.value() != String::from(Selection::Pick));
        thumbnails.add_filter(|item| item.selection.value() != String::from(Selection::Ordinary));
        thumbnails.add_filter(|item| item.selection.value() != String::from(Selection::Ignore));
        thumbnails.add_filter(|item| !item.hidden.value());

        thumbnails.set_filter_status(0, false);
        thumbnails.set_filter_status(1, false);
        thumbnails.set_filter_status(2, false);
        thumbnails.set_filter_status(3, true);

        thumbnails
            .selection_model
            .connect_selected_notify(move |s| {
                sender.input(PictureViewMsg::SelectPreview(Some(s.selected())))
            });

        let model = Self {
            thumbnails,
            preview_image: Default::default(),
            database,
        };
        let thumbnail_list = &model.thumbnails.view;
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
                self.thumbnails.clear();
                self.thumbnails
                    .extend_from_iter(pictures.into_iter().map(PictureThumbnail::from));
            }
            PictureViewMsg::SelectPreview(index) => {
                let filepath = index.and_then(|i| {
                    self.thumbnails
                        .get_visible(i)
                        .map(|t| t.borrow().filepath.clone())
                });
                self.preview_image = relm4::spawn_blocking(|| {
                    filepath.map(|p| {
                        let image = Pixbuf::from_file(p)
                            .expect("Image not found.")
                            .apply_embedded_orientation()
                            .expect("Unable to apply orientation.");
                        Texture::for_pixbuf(&image)
                    })
                })
                .await
                .unwrap();
            }
            PictureViewMsg::FilterPick(value) => {
                let index = 0;
                self.thumbnails.set_filter_status(index, value);
            }
            PictureViewMsg::FilterOrdinary(value) => {
                let index = 1;
                self.thumbnails.set_filter_status(index, value);
            }
            PictureViewMsg::FilterIgnore(value) => {
                let index = 2;
                self.thumbnails.set_filter_status(index, value);
            }
            PictureViewMsg::FilterHidden(value) => {
                let index = 3;
                self.thumbnails.set_filter_status(index, value);
            }
            PictureViewMsg::SelectionPick => {
                if let Some(thumbnail_item) = self.get_selected() {
                    let id = {
                        let thumbnail = thumbnail_item.borrow();
                        thumbnail.selection.set_value(String::from(Selection::Pick));
                        thumbnail.id
                    };
                    update_selection_state(&self.database, id, Selection::Pick)
                        .await
                        .unwrap();
                }
            }
            PictureViewMsg::SelectionOrdinary => {
                if let Some(thumbnail_item) = self.get_selected() {
                    let id = {
                        let thumbnail = thumbnail_item.borrow();
                        thumbnail
                            .selection
                            .set_value(String::from(Selection::Ordinary));
                        thumbnail.id
                    };
                    update_selection_state(&self.database, id, Selection::Ordinary)
                        .await
                        .unwrap();
                }
            }
            PictureViewMsg::SelectionIgnore => {
                if let Some(thumbnail_item) = self.get_selected() {
                    let id = {
                        let thumbnail = thumbnail_item.borrow();
                        thumbnail
                            .selection
                            .set_value(String::from(Selection::Ignore));
                        thumbnail.id
                    };
                    update_selection_state(&self.database, id, Selection::Ignore)
                        .await
                        .unwrap();
                }
            }
            PictureViewMsg::SelectionExport(dir) => {
                if let Some(pic) = self.get_selected() {
                    let origin = pic.borrow().filepath.clone();
                    let destination = dir.join(origin.file_name().unwrap());
                    tracing::info!("Copying file from {origin} to {destination}");
                    tokio::fs::copy(&origin, destination)
                        .await
                        .expect("Unable to copy image from {path}");
                }
            }
            PictureViewMsg::ImageNext => {
                let model = &self.thumbnails.selection_model;
                let index = model.selected();
                if index < model.n_items() {
                    model.set_selected(index + 1)
                }
            }
            PictureViewMsg::ImagePrev => {
                let model = &self.thumbnails.selection_model;
                let index = model.selected();
                if index > 0 {
                    model.set_selected(index - 1)
                }
            }
        }
    }
}
