use glib::Bytes;
use gtk::gdk::Texture;
use gtk::glib;
use gtk::prelude::*;
use image::DynamicImage;
use relm4::gtk::gdk_pixbuf::{Colorspace, Pixbuf};
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
    #[tracing::instrument(name = "Converting PictureData to PictureThumbnail")]
    fn from(picture: PictureData) -> Self {
        let thumbnail = picture.thumbnail.clone().map(|t| {
            let (colorspace, has_alpha, bits_per_sample) = match &t {
                DynamicImage::ImageRgb8(_) => (Colorspace::Rgb, false, 8_u32),
                DynamicImage::ImageRgba8(_) => (Colorspace::Rgb, true, 8_u32),
                _ => unimplemented!(),
            };
            let width = t.width();
            let height = t.height();
            let rowstride = if has_alpha {
                bits_per_sample * 4 * width / 8
            } else {
                bits_per_sample * 3 * width / 8
            };
            Texture::for_pixbuf(&Pixbuf::from_bytes(
                &Bytes::from(&t.into_bytes()),
                colorspace,
                has_alpha,
                bits_per_sample as i32,
                width as i32,
                height as i32,
                rowstride as i32,
            ))
        });
        PictureThumbnail { picture, thumbnail }
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

        rating.set_label(&format!("{}", self.picture.rating));
        selection.set_label(&format!("{}", self.picture.selection));
        thumbnail.set_paintable(self.thumbnail.as_ref());
    }

    fn unbind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {}
}
