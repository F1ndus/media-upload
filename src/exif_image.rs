use std::path::Path;
use rexiv2::Rexiv2Error;

pub(crate) fn remove_img_metadata(filepath: &Path) -> Option<&Path> {

    // Check if file exists
    match filepath.exists() {
        true => {
            let metadata = rexiv2::Metadata::new_from_path(filepath).unwrap();
            println!("Supports exif: {} -> {}", filepath.clone().display(), metadata.supports_exif());
            match metadata.supports_exif() {
                true => {
                    metadata.clear_exif();
                    if let Ok(res) = metadata.save_to_file(filepath) {
                        println!("Stripped Metadata");
                        Some((filepath))
                    } else {
                        None
                    }

                }
                _ => {
                    println!("Passed file does not support exif");
                    None
                }
            }
        }
        _ => {
            None
        }
    }

}