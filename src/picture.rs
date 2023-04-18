mod picture_data;
mod picture_thumbnail;
mod property_types;

use gtk::prelude::*;
pub use picture_data::*;
pub use picture_thumbnail::*;
pub use property_types::*;
use relm4::component::{AsyncComponent, AsyncComponentParts};
use relm4::gtk::gdk::Texture;
use relm4::typed_list_view::TypedListView;
use relm4::{gtk, AsyncComponentSender};
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
    preview_id: Option<Uuid>,
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

                    // connect_row_selected[sender] => move |_, row| {
                    //     let index = row.map(|r| r.index());
                    //     sender.input(PictureViewMsg::SelectPreview(index));
                    // }
                }
            }
        }
    }

    async fn init(
        _: (),
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut thumbnails: TypedListView<PictureThumbnail, gtk::SingleSelection> =
            TypedListView::with_sorting();

        thumbnails.add_filter(|item| item.picture.selection == Selection::Pick);
        thumbnails.add_filter(|item| item.picture.selection == Selection::Ordinary);
        thumbnails.add_filter(|item| item.picture.selection == Selection::Ignore);

        thumbnails.set_filter_status(0, false);
        thumbnails.set_filter_status(1, false);
        thumbnails.set_filter_status(2, false);

        let model = Self {
            thumbnails,
            preview_id: Default::default(),
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
            PictureViewMsg::SelectPreview(_index) => {
                // if let Some(pic) = index.and_then(|i| self.shown_thumbnails.get(i as usize)) {
                //     self.preview_id = Some(pic.picture.id);
                //     let filepath = pic.picture.filepath.clone();
                //     self.preview_image = Some(
                //         relm4::spawn(async move {
                //             let image = Pixbuf::from_file(filepath)
                //                 .expect("Image not found.")
                //                 .apply_embedded_orientation()
                //                 .expect("Unable to apply orientation.");
                //             Texture::for_pixbuf(&image)
                //         })
                //         .await
                //         .unwrap(),
                //     );
                // } else {
                //     self.preview_id = None;
                //     self.preview_image = None;
                // }
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
