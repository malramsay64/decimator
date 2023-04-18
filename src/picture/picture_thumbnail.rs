use glib::Bytes;
use gtk::gdk::Texture;

use gtk::glib;
use gtk::prelude::*;


use relm4::prelude::*;
use relm4::typed_list_view::RelmListItem;
use relm4::{gtk, view};

use super::PictureData;

#[derive(Debug)]
pub struct PictureThumbnail {
    pub picture: PictureData,
    thumbnail: Option<Texture>,
}

impl PartialEq for PictureThumbnail {
    fn eq(&self, other: &Self) -> bool {
        self.picture.id == other.picture.id
    }
}

impl Eq for PictureThumbnail {}

impl PartialOrd for PictureThumbnail {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.picture
            .capture_time?
            .partial_cmp(&other.picture.capture_time?)
    }
}

impl Ord for PictureThumbnail {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.picture.capture_time, other.picture.capture_time) {
            (Some(s), Some(o)) => s.cmp(&o),
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    }
}

impl From<PictureData> for PictureThumbnail {
    fn from(picture: PictureData) -> Self {
        let thumbnail = picture.thumbnail.clone().map(|t| Texture::from_bytes(&Bytes::from(&t.into_bytes())).unwrap());
        PictureThumbnail { picture, thumbnail }
    }
}

pub struct Widgets {
    thumbnail: gtk::Picture,
    rating: gtk::Label,
    flag: gtk::Label,
}

impl RelmListItem for PictureThumbnail {
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn setup(_item: &gtk::ListItem) -> (gtk::Box, Widgets) {
        view! {
            root = gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                // set_height_request: 300,
                // set_width_request: 260,
                set_margin_top: 5,
                set_margin_bottom: 15,
                set_margin_start: 5,
                set_margin_end: 5,
                set_focusable: true,

                #[name(thumbnail)]
                gtk::Picture {
                    set_width_request: 240,
                    set_height_request: 240,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    #[name(rating)]
                    gtk::Label {
                        set_hexpand: true,
                        set_margin_start: 10,
                        set_halign: gtk::Align::Start,
                    },
                    #[name(flag)]
                    gtk::Label {
                        set_hexpand: true,
                        set_margin_end: 10,
                        set_halign: gtk::Align::End,
                    }
                }
            }
        }

        let widgets = Widgets {
            thumbnail,
            rating,
            flag,
        };

        (root, widgets)
    }

    #[tracing::instrument(name = "Binding Widget", skip(widgets, _root))]
    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let Widgets {
            thumbnail,
            rating,
            flag,
        } = widgets;

        let _filepath = self.picture.filepath.clone();
        // self.thumbnail = Some(PictureData::load_thumbnail(filepath, 240, 240).unwrap());
        rating.set_label(&format!("{}", self.picture.rating));
        flag.set_label(&format!("{}", self.picture.flag));
        thumbnail.set_paintable(self.thumbnail.as_ref());
    }

    fn unbind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {}
}
