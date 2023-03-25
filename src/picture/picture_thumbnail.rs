use gtk::gdk::Texture;
use gtk::prelude::*;
use relm4::factory::positions::GridPosition;
use relm4::factory::{AsyncFactoryComponent, AsyncPosition};
use relm4::loading_widgets::LoadingWidgets;
use relm4::prelude::DynamicIndex;
use relm4::{gtk, view, AsyncFactorySender};

use super::PictureData;
use crate::AppMsg;

#[derive(Debug)]
pub struct PictureThumbnail {
    picture: PictureData,
    thumbnail: Option<Texture>,
}

#[derive(Debug)]
pub enum PictureThumbnailMsg {
    SetThumbnail(Option<Texture>),
}

impl AsyncPosition<GridPosition> for PictureThumbnail {
    fn position(index: usize) -> GridPosition {
        let x = index % 3;
        let y = index / 3;
        GridPosition {
            column: x as i32,
            row: y as i32,
            width: 1,
            height: 1,
        }
    }
}

#[relm4::factory(async, pub)]
impl AsyncFactoryComponent for PictureThumbnail {
    type Init = PictureData;
    type Input = PictureThumbnailMsg;
    type Output = PictureThumbnailMsg;
    type CommandOutput = ();
    type ParentInput = AppMsg;
    type ParentWidget = gtk::Grid;

    view! {
        root = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_margin_top: 5,
            set_margin_bottom: 5,
            set_margin_start: 5,
            set_margin_end: 5,
            set_focusable: true,
            // set_has_frame: true,

            #[name(thumbnail_picture)]
            gtk::Picture {
                // set_width_request: 240,
                set_height_request: 240,
                #[watch]
                set_paintable: self.thumbnail.as_ref(),
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                #[name(rating)]
                gtk::Label {
                    #[watch]
                    set_label: &self.picture.rating.to_string(),
                    set_hexpand: true,
                    set_margin_start: 10,
                    set_halign: gtk::Align::Start,
                },
                #[name(flag)]
                gtk::Label {
                    #[watch]
                    set_label: &self.picture.flag.to_string(),
                    set_hexpand: true,
                    set_margin_end: 10,
                    set_halign: gtk::Align::End,
                }
            }
        }
    }

    fn init_loading_widgets(root: &mut Self::Root) -> Option<LoadingWidgets> {
        view! {
            #[local_ref]
            root {
                set_orientation: gtk::Orientation::Vertical,

                #[name(spinner)]
                gtk::Spinner {
                    start: (),
                    set_hexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_height_request: 240,
                }
            }
        }
        Some(LoadingWidgets::new(root, spinner))
    }

    async fn init_model(
        picture: PictureData,
        _index: &DynamicIndex,
        sender: AsyncFactorySender<Self>,
    ) -> Self {
        let filepath = picture.filepath.clone();
        let thumbnail =
            relm4::spawn(async move { Some(PictureData::load_thumbnail(filepath, 240, 240)) })
                .await
                .unwrap();
        Self { picture, thumbnail }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncFactorySender<Self>) {
        match msg {
            PictureThumbnailMsg::SetThumbnail(thumbnail) => self.thumbnail = thumbnail,
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        // self.handle.abort();
        println!("Picture with path {} was destroyed", &self.picture.filepath);
    }
}
