use std::path::{Path, PathBuf};

use anyhow::Result;
use glib::{clone, Value};
use gtk::builders::PictureBuilder;
use gtk::gdk::Texture;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::{
    gio, Application, ApplicationWindow, Box, Image, Label, ListView, Orientation, Picture,
    PolicyType, ScrolledWindow, SignalListItemFactory, SingleSelection, StringObject,
};
use gtk::{prelude::*, StringList};
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::ImageBuffer;
use log::trace;
use walkdir::{DirEntry, WalkDir};
// mod image_object;
// use image_object::ImageObject;

const APP_ID: &str = "com.malramsay.Decimator";

struct PictureWidgets {
    image: gtk::Picture,
}

fn find_images<P: AsRef<Path>>(directory: &P) -> Result<impl Iterator<Item = PathBuf>> {
    // First we list all the files in the provided directory.
    // This has be done in a single threaded manner.
    Ok(WalkDir::new(directory)
        .into_iter()
        // Ignore any directories we don't have permissions for
        .filter_map(|e| e.ok())
        // This removes the directories from the listing
        .filter(is_image)
        .map(|p| p.into_path()))
}

fn is_image(entry: &DirEntry) -> bool {
    match entry.path().extension().map(|s| s.to_str()).flatten() {
        Some("jpg" | "JPG" | "heic") => true,
        Some("tiff" | "png" | "gif" | "RAW" | "webp" | "heif" | "arw" | "ARW") => false,
        _ => false,
    }
}

fn main() -> Result<()> {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run();
    Ok(())
}

fn build_ui(app: &Application) {
    let path = String::from("/home/malcolm/Pictures/2022/2022-04-14");
    let images: Vec<_> = find_images(&path)
        .expect("Image collection failed")
        .collect();
    let model: StringList = find_images(&path)
        .expect("Image collection failed.")
        .map(|p| {
            p.into_os_string()
                .into_string()
                .expect("Invalid UTF8 path.")
        })
        .collect();

    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
        let image = Picture::new();
        image.set_size_request(320, 320);
        list_item.set_child(Some(&image));
        // let label = Label::new(None);
        // list_item.set_child(Some(&label));
    });

    factory.connect_bind(move |_, list_item| {
        let string_object = list_item
            .item()
            .expect("The item has to exist.")
            .downcast::<StringObject>()
            .expect("The item has to be an `StringObject`.");

        let file_path = string_object.property::<String>("string");
        trace!("Loading image from {file_path}.");

        let image = list_item
            .child()
            .expect("The child has to exist.")
            .downcast::<Picture>()
            .expect("The child has to be a `Picture`.");

        let buffer = Pixbuf::from_file_at_scale(&file_path, 320, 320, true)
            .expect("Image not found")
            .apply_embedded_orientation()
            .expect("Unable to apply image orientation.");

        image.set_pixbuf(Some(&buffer));
    });

    let selection_model = SingleSelection::builder()
        .model(&model)
        .autoselect(true)
        .build();
    let list_view = ListView::new(Some(&selection_model), Some(&factory));

    let detail_image = Picture::new();

    selection_model.connect_selected_item_notify(clone!(@weak detail_image => move |item| {
        dbg!(item);
        let file_path = item
            .selected_item()
            .expect("No items selected")
            .downcast::<StringObject>()
            .expect("The item has to be a `String`.")
            .property::<String>("string");

        dbg!(&file_path);

        let buffer = Pixbuf::from_file(&file_path)
            .expect("Image not found")
            .apply_embedded_orientation()
            .expect("Unable to apply image orientation.");

        detail_image.set_pixbuf(Some(&buffer))
        // let buffer = Texture::for_pixbuf(&buffer);

        // image_buffer.clone_from(&Some(buffer.into()));
    }));

    let scrolled_window = ScrolledWindow::builder()
        // Disable horizontal scrolling
        .hscrollbar_policy(PolicyType::Never)
        .min_content_width(300)
        .child(&list_view)
        .build();

    let image_row = Box::new(Orientation::Horizontal, 10);
    image_row.append(&scrolled_window);
    image_row.append(&detail_image);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Decimator")
        .default_width(600)
        .default_height(300)
        .child(&image_row)
        .build();

    window.present();
}
