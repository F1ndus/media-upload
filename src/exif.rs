pub trait StripMetadata {
    fn remove_metadata(file: &Path) -> Option<Path>;
}

pub struct VideoFile<'a> {
    path: &'a str,
}

impl StripMetadata for VideoFile {
    fn remove_metadata(file: &_) -> _ {
        unimplemented!()
    }
}

pub struct Image<'a> {
    path: &'a str,
}

impl StripMetadata for Image {
    fn remove_metadata(file: &Path) -> Option<Path> {
        unimplemented!()
    }
}

