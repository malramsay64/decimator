mod picture_data;
mod picture_thumbnail;
mod property_types;

use gtk::glib;
use gtk::prelude::*;
pub use picture_data::*;
pub use picture_thumbnail::*;
pub use property_types::*;
use relm4::component::{AsyncComponent, AsyncComponentParts};
use relm4::gtk::gdk::Texture;
use relm4::gtk::gdk_pixbuf::Pixbuf;
use relm4::typed_list_view::{TypedListItem, TypedListView};
use relm4::{gtk, AsyncComponentSender};
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
    SelectPreview(Option<u32>),
    FilterPick(bool),
    FilterOrdinary(bool),
    FilterIgnore(bool),
    SelectionPick,
    SelectionOrdinary,
    SelectionIgnore,
}

#[derive(Debug)]
pub struct PictureView {
    thumbnails: TypedListView<PictureThumbnail, gtk::SingleSelection>,
    preview_image: Option<Texture>,
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
                thumbnail_list -> gtk::ListView {
                    set_width_request: 260,
                    set_show_separators: true,
                    set_enable_rubberband: true,

                }
            }
        }
    }

    async fn init(
        _: (),
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut thumbnails: TypedListView<PictureThumbnail, gtk::SingleSelection> =
            TypedListView::with_sorting();

        thumbnails.add_filter(|item| item.picture.selection != Selection::Pick);
        thumbnails.add_filter(|item| item.picture.selection != Selection::Ordinary);
        thumbnails.add_filter(|item| item.picture.selection != Selection::Ignore);

        thumbnails.set_filter_status(0, false);
        thumbnails.set_filter_status(1, false);
        thumbnails.set_filter_status(2, false);

        thumbnails
            .selection_model
            .connect_selected_notify(move |s| {
                sender.input(PictureViewMsg::SelectPreview(Some(s.selected())))
            });

        let model = Self {
            thumbnails,
            preview_image: Default::default(),
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
                let picture_data =
                    index.and_then(|i| self.thumbnails.get(i).map(|t| t.borrow().picture.clone()));
                self.preview_image = relm4::spawn_blocking(|| {
                    picture_data.map(|p| {
                        let image = Pixbuf::from_file(p.filepath)
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
            PictureViewMsg::SelectionPick => {}
            PictureViewMsg::SelectionOrdinary => {}
            PictureViewMsg::SelectionIgnore => {}
        }
    }
}
