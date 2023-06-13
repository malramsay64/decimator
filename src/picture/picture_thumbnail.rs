use camino::Utf8PathBuf;
use gtk::gdk::Texture;
use gtk::prelude::*;
use image::RgbaImage;
use relm4::binding::{BoolBinding, StringBinding};
use relm4::gtk::gdk_pixbuf::{Colorspace, Pixbuf};
use relm4::gtk::glib::Bytes;
use relm4::{gtk, view, RelmObjectExt};
use uuid::Uuid;

use super::{DateTime, PictureData};
use crate::relm_ext::RelmListItem;

#[derive(Debug)]
pub struct PictureThumbnail {
    pub id: Uuid,
    pub filepath: Utf8PathBuf,
    pub raw_extension: Option<String>,
    pub capture_time: Option<DateTime>,
    pub selection: StringBinding,
    pub rating: StringBinding,
    pub flag: StringBinding,
    pub hidden: BoolBinding,
    thumbnail: Option<Texture>,
    thumbnail_data: Option<RgbaImage>,
}

impl PartialEq for PictureThumbnail {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for PictureThumbnail {}

impl PartialOrd for PictureThumbnail {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.capture_time?.partial_cmp(&other.capture_time?)
    }
}

impl Ord for PictureThumbnail {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.capture_time, other.capture_time) {
            (Some(s), Some(o)) => s.cmp(&o),
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    }
}

impl From<PictureData> for PictureThumbnail {
    #[tracing::instrument(name = "Converting PictureData to PictureThumbnail")]
    fn from(picture: PictureData) -> Self {
        PictureThumbnail {
            id: picture.id,
            filepath: picture.filepath,
            raw_extension: picture.raw_extension,
            capture_time: picture.capture_time,
            selection: StringBinding::new(String::from(picture.selection)),
            rating: StringBinding::new(String::from(picture.rating)),
            flag: StringBinding::new(String::from(picture.flag)),
            hidden: BoolBinding::new(picture.hidden),
            thumbnail: None,
            thumbnail_data: picture.thumbnail,
        }
    }
}

pub struct Widgets {
    thumbnail: gtk::Picture,
    rating: gtk::Label,
    selection: gtk::Label,
}

impl RelmListItem for PictureThumbnail {
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn setup(_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
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
                    #[name(selection)]
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
            selection,
        };

        (root, widgets)
    }

    #[tracing::instrument(name = "Binding Widget", level = "trace", skip(widgets, _root))]
    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        let Widgets {
            thumbnail,
            rating,
            selection,
        } = widgets;

        if self.thumbnail.is_none() {
            self.thumbnail = self.thumbnail_data.as_ref().map(|data| {
                let colorspace = Colorspace::Rgb;
                let has_alpha = true;
                let bits_per_sample = 8_u32;
                let width = data.width();
                let height = data.height();
                let rowstride = bits_per_sample * 4 * width / 8;
                Texture::for_pixbuf(&Pixbuf::from_bytes(
                    &Bytes::from(data.as_ref()),
                    colorspace,
                    has_alpha,
                    bits_per_sample as i32,
                    width as i32,
                    height as i32,
                    rowstride as i32,
                ))
            })
        }

        rating.add_write_only_binding(&self.rating, "label");
        selection.add_write_only_binding(&self.selection, "label");
        thumbnail.set_paintable(self.thumbnail.as_ref());
    }

    fn unbind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {}
}
