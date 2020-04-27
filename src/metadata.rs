use std::path::{Path, PathBuf};

use rexiv2::Rexiv2Error;
use crate::exif_image;
use crate::exif_ffmpeg;

pub trait MetaData {
    fn remove_metadata(&self) -> Option<PathBuf>;
}

pub struct VideoFile<'a> {
    pub path: &'a str,
}

impl MetaData for VideoFile<'_> {
    fn remove_metadata(&self) -> Option<PathBuf> {
        exif_ffmpeg::remove_video_metadata(self.path.as_ref())
    }
}

pub struct Image<'a> {
    pub path: &'a str,
}
impl MetaData for Image<'_> {
    fn remove_metadata(&self) -> Option<PathBuf> {
        exif_image::remove_img_metadata(self.path.as_ref())
    }
}

pub struct Noop<'a> {
    pub path: &'a str,
}

impl MetaData for Noop<'_> {
    fn remove_metadata(&self) -> Option<PathBuf> {
        Some(PathBuf::new())
    }
}