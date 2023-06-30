use std::convert::identity;

use camino::Utf8PathBuf;
use gtk::prelude::*;
use relm4::adw::Window;
use relm4::component::{
    AsyncComponent, AsyncComponentController, AsyncComponentParts, AsyncController,
};
use relm4::gtk::gdk_pixbuf::Pixbuf;
use relm4::{gtk, tokio, AsyncComponentSender};
use sea_orm::DatabaseConnection;

use super::{ImageMsg, ImageWidget};
use crate::data::update_selection_state;
use crate::picture::picture_data::*;
use crate::picture::picture_thumbnail::*;
use crate::picture::Selection;
use crate::relm_ext::{TypedListItem, TypedListView};
use crate::AppMsg;

#[derive(Debug)]
pub enum ViewPreviewMsg {
    SelectPictures(Vec<PictureData>),
    SelectPreview(Option<u32>),
    DisplayPick(bool),
    DisplayOrdinary(bool),
    DisplayIgnore(bool),
    DisplayHidden(bool),
    SetSelection(Selection),
    SelectionExport(Utf8PathBuf),
    SelectionPrint(gtk::Window),
    SelectionZoom(Option<u32>),
    ImageNext,
    ImagePrev,
}

#[derive(Debug)]
pub struct ViewPreview {
    thumbnails: TypedListView<PictureThumbnail, gtk::SingleSelection>,
    preview_image: AsyncController<ImageWidget>,
    database: DatabaseConnection,
}

impl ViewPreview {
    pub fn get_selected(&self) -> Option<TypedListItem<PictureThumbnail>> {
        let index = self.thumbnails.selection_model.selected();
        self.thumbnails.get_visible(index)
    }
}

#[relm4::component(async, pub)]
impl AsyncComponent for ViewPreview {
    type Init = DatabaseConnection;
    type Input = ViewPreviewMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[local_ref]
            preview -> gtk::Box {},

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

        let thumbnail_sender = sender.clone();
        thumbnails
            .selection_model
            .connect_selected_notify(move |s| {
                thumbnail_sender.input(ViewPreviewMsg::SelectPreview(Some(s.selected())))
            });

        let thumbnail_list = &thumbnails.view;

        let image_sender = sender;
        let preview_image = ImageWidget::builder()
            .launch(())
            .forward(image_sender.input_sender(), identity);
        let preview = preview_image.widget();

        let widgets = view_output!();

        let model = Self {
            thumbnails,
            preview_image,
            database,
        };

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            ViewPreviewMsg::SelectPictures(pictures) => {
                self.thumbnails.clear();
                self.thumbnails
                    .extend_from_iter(pictures.into_iter().map(PictureThumbnail::from));
            }
            ViewPreviewMsg::SelectPreview(index) => {
                let filepath = index.and_then(|i| {
                    self.thumbnails
                        .get_visible(i)
                        .map(|t| t.borrow().filepath.clone())
                });
                self.preview_image.emit(ImageMsg::SetImage(
                    relm4::spawn_local(async {
                        filepath.map(|f| {
                            Pixbuf::from_file(f)
                                .expect("Unable to load file")
                                .apply_embedded_orientation()
                                .expect("Unable to apply orientation")
                        })
                    })
                    .await
                    .unwrap(),
                ))
            }
            ViewPreviewMsg::DisplayPick(value) => {
                let index = 0;
                self.thumbnails.set_filter_status(index, !value);
            }
            ViewPreviewMsg::DisplayOrdinary(value) => {
                let index = 1;
                self.thumbnails.set_filter_status(index, !value);
            }
            ViewPreviewMsg::DisplayIgnore(value) => {
                let index = 2;
                self.thumbnails.set_filter_status(index, !value);
            }
            ViewPreviewMsg::DisplayHidden(value) => {
                let index = 3;
                self.thumbnails.set_filter_status(index, !value);
            }
            ViewPreviewMsg::SetSelection(s) => {
                if let Some(thumbnail_item) = self.get_selected() {
                    let id = {
                        let thumbnail = thumbnail_item.borrow();
                        thumbnail.selection.set_value(String::from(s));
                        thumbnail.id
                    };
                    update_selection_state(&self.database, id, s).await.unwrap();
                }
            }
            ViewPreviewMsg::SelectionExport(dir) => {
                if let Some(pic) = self.get_selected() {
                    let origin = pic.borrow().filepath.clone();
                    let destination = dir.join(origin.file_name().unwrap());
                    tracing::info!("Copying file from {origin} to {destination}");
                    tokio::fs::copy(&origin, destination)
                        .await
                        .expect("Unable to copy image from {path}");
                }
            }
            ViewPreviewMsg::ImageNext => {
                let model = &self.thumbnails.selection_model;
                let index = model.selected();
                if index < model.n_items() {
                    model.set_selected(index + 1)
                }
            }
            ViewPreviewMsg::ImagePrev => {
                let model = &self.thumbnails.selection_model;
                let index = model.selected();
                if index > 0 {
                    model.set_selected(index - 1)
                }
            }
            ViewPreviewMsg::SelectionPrint(window) => {
                self.preview_image.emit(ImageMsg::Print(window))
            }
            ViewPreviewMsg::SelectionZoom(scale) => {
                self.preview_image.emit(ImageMsg::Scale(scale));
            }
        }
    }
}
