use std::collections::HashMap;

use anyhow::Error;
use camino::{Utf8Path, Utf8PathBuf};
use glib::{user_special_dir, UserDirectory};
use itertools::Itertools;
use relm4::gtk;
use relm4::gtk::glib;
use sea_orm::DatabaseConnection;
use walkdir::WalkDir;

use crate::data::{add_new_images, query_existing_pictures};
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

pub fn map_directory_images(directory: &Utf8Path) -> Vec<PictureData> {
    WalkDir::new(directory)
        // This ensures the filenames are in order
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(is_image)
        .map(|p| p.into_path().try_into().expect("Invalid UTF-8 path."))
        .group_by(|p: &Utf8PathBuf| p.with_extension(""))
        .into_iter()
        .filter_map(|(_key, group)| {
            group.fold(None, |data: Option<PictureData>, path| {
                match (data, path.extension()) {
                    // When we haven't created the image we don't care what the filetype is
                    (None, _) => Some(PictureData::from(path)),
                    // We have created the PictureData from the RAW file so need to re-generate
                    (Some(p), Some("jpg" | "JPG")) => {
                        let mut output = PictureData::from(path);
                        output.raw_extension = Some(
                            p.filepath
                                .extension()
                                .expect("There must be a file extension set.")
                                .to_owned(),
                        );
                        Some(output)
                    }
                    (Some(mut p), Some(e)) => {
                        p.raw_extension = Some(e.to_owned());
                        Some(p)
                    }
                    (Some(_), None) => unreachable!(),
                }
            })
        })
        .map(|mut p: PictureData| {
            p.update_from_exif().expect("File not found");
            p
        })
        .collect()
}

// Copy the files from an exisiting location creating a new folder
// structure.
pub async fn import(
    db: &DatabaseConnection,
    directory: &Utf8PathBuf,
    progress: &gtk::ProgressBar,
) -> Result<(), Error> {
    progress.set_text(Some("Checking for existing images."));
    progress.set_show_text(true);
    // Load all existing pictures from the database. We want to do the checks within rust, rather than
    // potentially having large numbers of database queries.
    // The list of all the pictures that currently exist within the database.
    let pictures_existing = query_existing_pictures(db, &Utf8PathBuf::from(""))
        .await
        .unwrap()
        .into_iter()
        .map(PictureData::from)
        .collect::<Vec<_>>();

    progress.pulse();
    // To make the lookup process simpler, we first want to convert to a hashmap to make the
    // act of looking up whether a picture already exists withih the database a quick proces.
    // Currently this is only using the filename, that is the name given to the file by the camera
    // as the lookup key, however, this lookup process will be improved over time.
    // TODO: Use hashes to check for uniqueness within the database.
    let mut hash_existing: HashMap<String, Vec<PictureData>> = HashMap::new();
    for picture in pictures_existing.into_iter() {
        if let Some(f) = hash_existing.get_mut(&picture.filename()) {
            f.push(picture);
        } else {
            hash_existing.insert(picture.filename(), vec![picture]);
        }
    }
    progress.pulse();

    // Determine whether the new images we are importing already exist within the database.
    let mut new_images: Vec<_> = map_directory_images(directory)
        .into_iter()
        // TODO: Improve this filter beyond being very basic
        .filter(|p| !hash_existing.contains_key(&p.filename()))
        .collect();

    progress.pulse();
    progress.set_text(Some("Importing new images"));
    progress.set_fraction(0.);

    let total_images = new_images.len();

    // Parallel map -> Will need to be careful about the directory creation
    // Spawn async tasks to do the copying?
    for (index, image) in new_images.iter_mut().enumerate() {
        tracing::debug!("{:?}", &image);
        // Create the directory structure
        let structure = ImportStructure::default();

        let new_path = structure.build_filename(image)?;

        let parent = new_path.parent();
        tracing::debug!("Importing {} into {}", image.filepath, &new_path);
        dbg!(&parent);

        // Where the new path is the same as the old one we are actually adding the
        // file rather than importing, so we can skip all the import steps.
        if image.filepath != new_path {
            // Copy to new location

            // Firstly we have to be sure that the directory already exists we are
            // going to be copying to. This creates the entire directory structure
            // where it doesn't already exist.
            std::fs::create_dir_all(parent.unwrap())?;

            // Where there is a file that already exists within the new locaton,
            // we don't want to overwrite it, which does result in the contents
            // of the file being removed.
            if new_path.try_exists().expect("Error checking path exists") {
                tracing::warn!("File {} already exsits, not copying", &new_path);
            } else {
                std::fs::copy(&image.filepath, &new_path).expect("Unable to copy file");
                // Also copy across the raw file
                if let Some(ref ext) = image.raw_extension {
                    std::fs::copy(
                        &image.filepath.with_extension(ext),
                        &new_path.with_extension(ext),
                    )
                    .expect("Unable to copy file");
                }
            }
        }

        image.filepath = new_path;

        progress.set_fraction(index as f64 / total_images as f64);
    }

    progress.set_fraction(1.);

    progress.set_text(Some("Adding images to database"));
    progress.set_fraction(0.);
    // Create entry in the database / import
    add_new_images(db, new_images).await.unwrap();
    progress.pulse();
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

pub async fn find_new_images(db: &DatabaseConnection, directory: &Utf8PathBuf) {
    let existing_pictures = query_existing_pictures(db, directory).await.unwrap();

    tracing::info!(
        "Found {} existing files within directory",
        existing_pictures.len()
    );

    let dir = directory.clone();
    let images: Vec<_> = relm4::spawn_blocking(move || {
        map_directory_images(&dir)
            .into_iter()
            .filter(|p| !existing_pictures.contains(&p.filepath))
            .collect()
    })
    .await
    .unwrap();

    if images.is_empty() {
        tracing::info!("No new images found in directory {directory}");
        return;
    } else {
        tracing::info!("Adding {} new images to the database.", images.len());
    }

    add_new_images(db, images).await.unwrap();
}
