use std::path::{Path, PathBuf};
use failure::Error;

pub(crate) fn remove_img_metadata(filepath: &Path) -> Result<PathBuf, Error> {

    #[derive(Debug, Fail)]
    enum ExifError {
        #[fail(display = "Exif is unsupported on this filetype")]
        ExifUnsupported,
    }

    // Check if file exists

    let metadata = rexiv2::Metadata::new_from_path(filepath)?;
    println!("Supports exif: {} -> {}", filepath.clone().display(), metadata.supports_exif());

    match metadata.supports_exif() {
        true => {
            metadata.clear_exif();
            metadata.save_to_file(filepath)?;

            println!("Stripped Metadata");
            Ok(PathBuf::from(filepath))
        }
        _ => {
            println!("Passed file does not support exif");
            Err(ExifError::ExifUnsupported)?
        }
    }
}
