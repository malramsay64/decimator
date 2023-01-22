use std::collections::HashMap;

use anyhow::Error;
use camino::{Utf8Path, Utf8PathBuf};
use glib::{user_special_dir, UserDirectory};
use gtk::glib;
use sqlx::SqlitePool;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use walkdir::{DirEntry, WalkDir};

use crate::data::{add_new_images, query_directory_pictures};
use crate::picture::{is_image, PictureData};

#[derive(Clone, Debug)]
struct ImportStructure {
    base_directory: Utf8PathBuf,
    expansion: String,
}

impl ImportStructure {
    fn build_filename(&self, image: &PictureData) -> Result<Utf8PathBuf, Error> {
        let capture_time = image.capture_time.unwrap();
        // Get the image capture date
        Ok(format!(
            // TODO: Get this from self
            "{base}/{year:04}/{year:04}-{month:02}-{day:02}/{filename}",
            base = self.base_directory,
            year = capture_time.year(),
            month = capture_time.month(),
            day = capture_time.day(),
            filename = image.filename()
        )
        .try_into()?)
    }
}

impl Default for ImportStructure {
    fn default() -> Self {
        Self {
            base_directory: user_special_dir(UserDirectory::Pictures)
                .unwrap()
                .try_into()
                .unwrap(),
            expansion: "{base_directory}/{year:04}/{year:04}-{month:02}-{day:02}/{filename}".into(),
        }
    }
}

// Copy the files from an exisiting location creating a new folder
// structure.
pub fn import(runtime: &Runtime, db: &SqlitePool, directory: &Utf8Path) -> Result<(), Error> {
    // Load all pictures
    let (tx, mut rx) = oneshot::channel();
    runtime.block_on(async move {
        let results = query_directory_pictures(db, String::from("%/"))
            .await
            .unwrap();
        tx.send(results).unwrap()
    });

    let pictures: Vec<PictureData> = rx.try_recv().unwrap();

    let mut hash_existing: HashMap<String, Vec<PictureData>> = HashMap::new();

    for picture in pictures.into_iter() {
        if let Some(f) = hash_existing.get_mut(&picture.filename()) {
            f.push(picture);
        } else {
            hash_existing.insert(picture.filename(), vec![picture]);
        }
    }

    // Quick method

    let mut new_images: Vec<_> = WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(is_image)
        .map(|p: DirEntry| p.into_path().try_into().expect("Invalid UTF-8 path."))
        .map(Utf8PathBuf::into)
        .map(|mut p: PictureData| {
            p.update_from_exif().unwrap();
            p
        })
        // TODO: Improve this filter beyond being very basic
        .filter(|p| !hash_existing.contains_key(&p.filename()))
        .collect();

    // Parallel map -> Will need to be careful about the directory creation
    // Spawn async tasks to do the copying?
    for mut image in new_images.into_iter() {
        tracing::debug!("{:?}", &image);
        // Create the directory structure
        let structure = ImportStructure::default();

        let new_path = structure.build_filename(&image)?;

        let parent = new_path.parent();
        tracing::debug!("Importing {} into {}", image.filepath, &new_path);
        dbg!(&parent);

        // Copy to new location
        std::fs::create_dir_all(parent.unwrap())?;
        std::fs::copy(&image.filepath, &new_path).expect("Unable to copy file");

        image.filepath = new_path;

        // Create entry in the database / import
        runtime.block_on(async move { add_new_images(db, vec![image]).await.unwrap() });
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
    Ok(())
}
