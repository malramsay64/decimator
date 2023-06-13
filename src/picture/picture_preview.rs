use std::io::Seek;
use std::path::Path;

use anyhow::{anyhow, Result};
use exif::{In, Tag};
use gtk::gdk::Texture;
use image::imageops::{flip_horizontal, flip_vertical, rotate180, rotate270, rotate90, FilterType};
use image::io::Reader;
use image::RgbaImage;
use relm4::gtk;
use relm4::gtk::gdk_pixbuf::{Colorspace, InterpType, Pixbuf};
use relm4::gtk::glib::Bytes;

#[derive(Debug, Default, Clone)]
pub struct Image {
    pub original: Option<Pixbuf>,
    pub preview: Option<Texture>,
}

impl Image {
    pub fn from_file(filepath: impl AsRef<Path>, scale: Option<(i32, i32)>) -> Result<Self> {
        let original = if let Some((scale_x, scale_y)) = scale {
            Pixbuf::from_file_at_scale(filepath, scale_x, scale_y, true)?
        } else {
            Pixbuf::from_file(filepath)?
        }
        .apply_embedded_orientation()
        .ok_or(anyhow!("Unable to apply image orientation."))?;
        let texture = Texture::for_pixbuf(&original);
        Ok(Self {
            original: Some(original),
            preview: Some(texture),
        })
    }

    pub fn update_preview(&mut self) {
        if let Some(image) = &self.original {
            self.preview.replace(Texture::for_pixbuf(&image));
        }
    }

    pub fn scale_to_fit(&self, width: u32, height: u32) -> Option<Pixbuf> {
        if let Some(image) = &self.original {
            let image_width = image.width() as f64;
            let image_height = image.height() as f64;
            let width_ratio = width as f64 / image_width;
            let height_ratio = height as f64 / image_height;
            let scale_ratio = width_ratio.min(height_ratio);
            // Perform the calculations for the scale in f64 to
            // remove as much of the rounding error as possible.
            image.scale_simple(
                (image_width * scale_ratio) as i32,
                (image_height * scale_ratio) as i32,
                InterpType::Nearest,
            )
        } else {
            None
        }
    }
}

fn image_to_texture(image: RgbaImage) -> Texture {
    let width = image.width();
    let height = image.height();
    let has_alpha = true;
    let bits_per_sample = 8;

    let rowstride = 4 * width;

    Texture::for_pixbuf(&Pixbuf::from_bytes(
        &Bytes::from(&image.as_ref()),
        Colorspace::Rgb,
        has_alpha,
        bits_per_sample as i32,
        width as i32,
        height as i32,
        rowstride as i32,
    ))
}

fn load_image(filepath: impl AsRef<Path>, size: Option<(u32, u32)>) -> Result<RgbaImage> {
    let file = std::fs::File::open(filepath)?;
    let mut cursor = std::io::BufReader::new(file);
    let exif_data = exif::Reader::new().read_from_container(&mut cursor)?;

    // Reset the buffer to the start to read the image file
    cursor.rewind()?;
    let mut image = Reader::new(cursor).with_guessed_format()?.decode()?;
    if let Some((scale_x, scale_y)) = size {
        image = image.resize(scale_x, scale_y, FilterType::Triangle)
    }
    // Apply Exif image transformations
    // https://sirv.com/help/articles/rotate-photos-to-be-upright/
    Ok(
        match exif_data
            .get_field(Tag::Orientation, In::PRIMARY)
            .and_then(|e| e.value.get_uint(0))
        {
            Some(1) => image.into_rgba8(),
            Some(2) => flip_horizontal(&image),
            Some(3) => rotate180(&image),
            Some(4) => flip_vertical(&image),
            Some(5) => rotate270(&flip_horizontal(&image)),
            Some(6) => rotate90(&image),
            Some(7) => rotate90(&flip_horizontal(&image)),
            Some(8) => rotate270(&image),
            // Where we can't interpret the exif data, we revert to the base image
            _ => image.into_rgba8(),
        },
    )
}
