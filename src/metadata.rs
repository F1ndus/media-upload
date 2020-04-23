use std::path::Path;

use rexiv2::Rexiv2Error;

pub(crate) fn remove_img_metadata(filepath: &Path) -> Result<(), Rexiv2Error> {

    // Check if file exists
    match filepath.exists() {
        true => {
            let metadata = rexiv2::Metadata::new_from_path(filepath).unwrap();
            println!("Supports exif: {} -> {}", filepath.clone().display(), metadata.supports_exif());

            match metadata.supports_exif() {
                true => {
                    metadata.clear_exif();
                    metadata.save_to_file(filepath)?;
                    println!("Stripped Metadata");
                    Ok(())
                }
                _ => {
                    println!("Passed file does not support exif");
                    Err(Rexiv2Error::Internal(Option::from(String::from("File does not exit"))))
                }
            }

        }
        _ => {
            Err(Rexiv2Error::Internal(Option::from(String::from("File does not exit"))))
        }
    }

}