mod picture_data;
mod picture_thumbnail;
mod property_types;

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
use walkdir::DirEntry;

use crate::data::query_directory_pictures;
use crate::directory::Directory;
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
}

#[derive(Debug)]
pub struct PictureView {
    thumbnails: AsyncFactoryVecDeque<PictureThumbnail>,
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
                thumbnail_list -> gtk::ListBox {
                    set_width_request: 260,
                    set_show_separators: true,
                    set_selection_mode: gtk::SelectionMode::Single,

                    connect_row_selected[sender] => move |_, row| {
                        let index = row.map(|r| r.index());
                        println!("{index:?}");
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
        let thumbnails = AsyncFactoryVecDeque::new(gtk::ListBox::default(), sender.input_sender());
        let model = Self {
            thumbnails,
            preview_image: None,
        };
        let thumbnail_list = model.thumbnails.widget();
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            PictureViewMsg::SelectPictures(pictures) => {
                let mut thumbnail_guard = self.thumbnails.guard();
                thumbnail_guard.clear();
                for pic in pictures {
                    thumbnail_guard.push_back(pic);
                }
            }
            PictureViewMsg::SelectPreview(index) => {
                self.preview_image =
                    if let Some(pic) = index.and_then(|i| self.thumbnails.get(i as usize)) {
                        let filepath = pic.picture.filepath.clone();
                        Some(
                            relm4::spawn(async move {
                                let image = Pixbuf::from_file(filepath)
                                    .expect("Image not found.")
                                    .apply_embedded_orientation()
                                    .expect("Unable to apply orientation.");
                                Texture::for_pixbuf(&image)
                            })
                            .await
                            .unwrap(),
                        )
                    } else {
                        None
                    }
            }
        }
    }
}
