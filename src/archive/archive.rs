use std::io::{Error as IoError, Read};
use std::path::Path;

use tar::Archive as TarArchive;

pub type Result<T> = ::std::result::Result<T, IoError>;

pub struct Archive<R: Read> {
    /// The tar archive
    inner: TarArchive<R>,
}

impl<R: Read> Archive<R> {
    /// Construct a new archive extractor.
    pub fn new(reader: R) -> Archive<R> {
        Archive {
            inner: TarArchive::new(reader),
        }
    }

    /// Extract the archive to the given destination.
    pub fn extract<P: AsRef<Path>>(&mut self, destination: P) -> Result<()> {
        self.inner.unpack(destination)
    }
}
