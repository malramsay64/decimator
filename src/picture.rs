mod picture_data;
// mod picture_object;
// mod picture_preview;
mod picture_thumbnail;
mod property_types;

pub use picture_data::*;
// pub use picture_object::*;
// pub use picture_preview::*;
pub use picture_thumbnail::*;
pub use property_types::*;
use walkdir::DirEntry;

pub fn is_image(entry: &DirEntry) -> bool {
    match entry.path().extension().and_then(|s| s.to_str()) {
        Some("jpg" | "JPG" | "raw" | "RAW" | "ARW" | "arw" | "raf" | "RAF") => true,
        Some("tiff" | "png" | "gif" | "webp" | "heif" | "heic") => false,
        _ => false,
    }
}
