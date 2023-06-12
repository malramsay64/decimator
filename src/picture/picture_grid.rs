use camino::Utf8PathBuf;
use gtk::prelude::*;
use relm4::component::{AsyncComponent, AsyncComponentParts};
use relm4::{gtk, tokio, AsyncComponentSender};
use sea_orm::DatabaseConnection;

use crate::data::update_selection_state;
use crate::picture::picture_data::*;
use crate::picture::picture_thumbnail::*;
use crate::picture::property_types::*;
use crate::relm_ext::{TypedGridView, TypedListItem};
use crate::AppMsg;

#[derive(Debug)]
pub enum PictureGridMsg {
    SelectPictures(Vec<PictureData>),
    FilterPick(bool),
    FilterOrdinary(bool),
    FilterIgnore(bool),
    FilterHidden(bool),
    SelectionPick,
    SelectionOrdinary,
    SelectionIgnore,
    SelectionExport(Utf8PathBuf),
}

#[derive(Debug)]
pub struct PictureGrid {
    thumbnails: TypedGridView<PictureThumbnail, gtk::MultiSelection>,
    database: DatabaseConnection,
}

impl PictureGrid {
    pub fn get_selected(&self) -> Vec<TypedListItem<PictureThumbnail>> {
        let bitvec = self.thumbnails.selection_model.selection();
        let mut indicies = vec![];
        if let Some((iter, value)) = gtk::BitsetIter::init_first(&bitvec) {
            indicies.push(value);
            for value in iter {
                indicies.push(value);
            }
        }
        indicies
            .into_iter()
            .map(|index| self.thumbnails.get_visible(index).unwrap())
            .collect()
    }
}

#[relm4::component(async, pub)]
impl AsyncComponent for PictureGrid {
    type Init = DatabaseConnection;
    type Input = PictureGridMsg;
    type Output = AppMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::ScrolledWindow {
                set_has_frame: true,
                set_vexpand: true,

                #[local_ref]
                thumbnail_grid -> gtk::GridView {
                    // set_show_separators: true,
                    set_enable_rubberband: true,
                    set_orientation: gtk::Orientation::Vertical,
                }
            }
        }
    }

    async fn init(
        database: DatabaseConnection,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut thumbnails: TypedGridView<PictureThumbnail, gtk::MultiSelection> =
            TypedGridView::with_sorting();

        thumbnails.add_filter(|item| item.selection.value() != String::from(Selection::Pick));
        thumbnails.add_filter(|item| item.selection.value() != String::from(Selection::Ordinary));
        thumbnails.add_filter(|item| item.selection.value() != String::from(Selection::Ignore));
        thumbnails.add_filter(|item| !item.hidden.value());

        thumbnails.set_filter_status(0, false);
        thumbnails.set_filter_status(1, false);
        thumbnails.set_filter_status(2, false);
        thumbnails.set_filter_status(3, true);

        let model = Self {
            thumbnails,
            database,
        };
        let thumbnail_grid = &model.thumbnails.view;
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
            PictureGridMsg::SelectPictures(pictures) => {
                self.thumbnails.clear();
                self.thumbnails
                    .extend_from_iter(pictures.into_iter().map(PictureThumbnail::from));
            }
            PictureGridMsg::FilterPick(value) => {
                let index = 0;
                self.thumbnails.set_filter_status(index, value);
            }
            PictureGridMsg::FilterOrdinary(value) => {
                let index = 1;
                self.thumbnails.set_filter_status(index, value);
            }
            PictureGridMsg::FilterIgnore(value) => {
                let index = 2;
                self.thumbnails.set_filter_status(index, value);
            }
            PictureGridMsg::FilterHidden(value) => {
                let index = 3;
                self.thumbnails.set_filter_status(index, value);
            }
            PictureGridMsg::SelectionPick => {
                for thumbnail_item in self.get_selected() {
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
            PictureGridMsg::SelectionOrdinary => {
                for thumbnail_item in self.get_selected() {
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
            PictureGridMsg::SelectionIgnore => {
                for thumbnail_item in self.get_selected() {
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
            PictureGridMsg::SelectionExport(dir) => {
                for thumbnail_item in self.get_selected() {
                    let origin = thumbnail_item.borrow().filepath.clone();
                    let destination = dir.join(origin.file_name().unwrap());
                    tracing::info!("Copying file from {origin} to {destination}");
                    tokio::fs::copy(&origin, destination)
                        .await
                        .expect("Unable to copy image from {path}");
                }
            }
        }
    }
}
