use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use glib::{user_special_dir, UserDirectory};
use gtk::glib;
use sqlx::SqlitePool;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use walkdir::{DirEntry, WalkDir};

use crate::data::{query_directory_pictures};
use crate::picture::{is_image, PictureData};

#[derive(Clone, Debug)]
struct ImportStructure {
    base_directory: Utf8PathBuf,
    expansion: String,
}

impl Default for ImportStructure {
    fn default() -> Self {
        Self {
            base_directory: user_special_dir(UserDirectory::Pictures)
                .unwrap()
                .try_into()
                .unwrap(),
            expansion: "{year}/{year}-{month}-{day}/{filename}".into(),
        }
    }
}

// Copy the files from an exisiting location creating a new folder
// structure.
fn import(runtime: &Runtime, db: &SqlitePool, directory: &Utf8Path) {
    // Load all pictures
    let (tx, mut rx) = oneshot::channel();
    runtime.block_on(async move {
        let results = query_directory_pictures(db, String::from("%/"))
            .await
            .unwrap();
        tx.send(results).unwrap()
    });

    let pictures: Vec<PictureData> = rx.try_recv().unwrap();

    let mut hash_map: HashMap<String, Vec<PictureData>> = HashMap::new();

    for picture in pictures.into_iter() {
        if let Some(f) = hash_map.get_mut(&picture.filename()) {
            f.push(picture);
        } else {
            hash_map.insert(picture.filename(), vec![picture]);
        }
    }

    // Quick method

    let new_images: Vec<_> = WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(is_image)
        .map(|p: DirEntry| p.into_path().try_into().expect("Invalid UTF-8 path."))
        .map(Utf8PathBuf::into)
        // TODO: Improve this filter beyond being very basic
        .filter(|p: &PictureData| !hash_map.contains_key(&p.filename()))
        .collect();

    // Parallel map -> Will need to be careful about the directory creation
    // Spawn async tasks to do the copying?
    for image in new_images.into_iter() {
        dbg!(image);
        // Create the directory structure

        // Copy to new location

        // Create entry in the database / import
    }
    // Check the filename
    // Check the capture time

    // Full Method

    // 1. Check the short hash of the files
    //     This involves reading just the first n bytes of the new file. This should
    //     be a very quick operation.
    // 2. Check the long hash of the files
    //     This involves just reading the new file, the hashes of the previous files
    //     should already be computed.
    // 3. Check the files are equal
    //     Now we also need to read in the old file to check.
}
