mod picture_data;
mod picture_thumbnail;
mod property_types;

use std::io::Seek;
use std::path::Path;

use anyhow::Result;
use exif::{In, Tag};
use image::imageops::{flip_horizontal, flip_vertical, rotate180, rotate270, rotate90, FilterType};
use image::io::Reader;
use image::RgbaImage;
pub use picture_data::*;
pub use picture_thumbnail::*;
pub use property_types::*;

pub fn is_image(entry: &walkdir::DirEntry) -> bool {
    match entry.path().extension().and_then(|s| s.to_str()) {
        Some("jpg" | "JPG" | "raw" | "RAW" | "ARW" | "arw" | "raf" | "RAF") => true,
        Some("tiff" | "png" | "gif" | "webp" | "heif" | "heic") => false,
        _ => false,
    }
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
