mod picture_data;
mod picture_thumbnail;
mod property_types;
mod view_grid;
mod view_preview;

pub use picture_data::*;
pub use picture_thumbnail::*;
pub use property_types::*;
pub use view_grid::*;
pub use view_preview::*;

pub fn is_image(entry: &walkdir::DirEntry) -> bool {
    match entry.path().extension().and_then(|s| s.to_str()) {
        Some("jpg" | "JPG" | "raw" | "RAW" | "ARW" | "arw" | "raf" | "RAF") => true,
        Some("tiff" | "png" | "gif" | "webp" | "heif" | "heic") => false,
        _ => false,
    }
}
