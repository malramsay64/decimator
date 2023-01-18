mod picture_object;
mod picture_preview;
mod picture_thumbnail;
mod property_types;

pub use picture_object::*;
pub use picture_preview::*;
pub use picture_thumbnail::*;
pub use property_types::*;
use walkdir::DirEntry;

pub fn is_image(entry: &DirEntry) -> bool {
    match entry.path().extension().and_then(|s| s.to_str()) {
        Some("jpg" | "JPG") => true,
        Some("tiff" | "png" | "gif" | "RAW" | "webp" | "heif" | "heic" | "arw" | "ARW") => false,
        _ => false,
    }
}
