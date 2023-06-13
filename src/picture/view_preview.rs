use camino::Utf8PathBuf;
use gtk::prelude::*;
use relm4::adw::Window;
use relm4::component::{AsyncComponent, AsyncComponentParts};
use relm4::gtk::{PrintOperation, PrintSettings};
use relm4::{gtk, tokio, AsyncComponentSender};
use sea_orm::DatabaseConnection;

use super::Image;
use crate::data::update_selection_state;
use crate::picture::picture_data::*;
use crate::picture::picture_thumbnail::*;
use crate::picture::property_types::*;
use crate::relm_ext::{TypedListItem, TypedListView};
use crate::AppMsg;

#[derive(Debug)]
pub enum ViewPreviewMsg {
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
    SelectionPrint(Window),
    ImageNext,
    ImagePrev,
}

#[derive(Debug)]
pub struct ViewPreview {
    thumbnails: TypedListView<PictureThumbnail, gtk::SingleSelection>,
    preview_image: Image,
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
            gtk::Box {
                set_vexpand: true,
                set_hexpand: true,
                gtk::Picture {
                    #[watch]
                    set_paintable: model.preview_image.preview.as_ref(),
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
                sender.input(ViewPreviewMsg::SelectPreview(Some(s.selected())))
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
        root: &Self::Root,
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
                self.preview_image = relm4::spawn_local(async {
                    filepath.map_or_else(Image::default, |f| Image::from_file(f, None).unwrap())
                })
                .await
                .unwrap()
            }
            ViewPreviewMsg::FilterPick(value) => {
                let index = 0;
                self.thumbnails.set_filter_status(index, value);
            }
            ViewPreviewMsg::FilterOrdinary(value) => {
                let index = 1;
                self.thumbnails.set_filter_status(index, value);
            }
            ViewPreviewMsg::FilterIgnore(value) => {
                let index = 2;
                self.thumbnails.set_filter_status(index, value);
            }
            ViewPreviewMsg::FilterHidden(value) => {
                let index = 3;
                self.thumbnails.set_filter_status(index, value);
            }
            ViewPreviewMsg::SelectionPick => {
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
            ViewPreviewMsg::SelectionOrdinary => {
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
            ViewPreviewMsg::SelectionIgnore => {
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
            ViewPreviewMsg::SelectionPrint(window) => self.print(&window),
        }
    }
}
impl ViewPreview {
    fn print(&self, window: &Window) {
        let settings = PrintSettings::new();
        settings.set_quality(gtk::PrintQuality::High);
        settings.set_media_type(&"photographic");
        let print_operation = PrintOperation::new();
        print_operation.set_print_settings(Some(&settings));

        let preview_image = self.preview_image.clone();
        print_operation.connect_draw_page(move |_, print_context, _| {
            if let Some(image) = preview_image
                .scale_to_fit(print_context.width() as u32, print_context.height() as u32)
            {
                let cairo_context = print_context.cairo_context();
                cairo_context.set_source_pixbuf(
                    &image,
                    (print_context.width() - image.width() as f64) / 2.0,
                    (print_context.height() - image.height() as f64) / 2.0,
                );
                if let Err(error) = cairo_context.paint() {
                    tracing::error!("Couldn't print current image: {}", error);
                }
            }
        });

        print_operation.set_allow_async(true);
        print_operation
            .run(gtk::PrintOperationAction::PrintDialog, Some(window))
            .unwrap();
    }
}
