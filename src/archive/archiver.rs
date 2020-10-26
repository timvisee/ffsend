use std::fs::File;
use std::io::{Error as IoError, Write};
use std::path::Path;

use tar::Builder as TarBuilder;

pub type Result<T> = ::std::result::Result<T, IoError>;

pub struct Archiver<W: Write> {
    /// The tar builder.
    inner: TarBuilder<W>,
}

impl<W: Write> Archiver<W> {
    /// Construct a new archive builder.
    pub fn new(writer: W) -> Archiver<W> {
        Archiver {
            inner: TarBuilder::new(writer),
        }
    }

    /// Add the entry at the given `src` path, to the given relative `path` in the archive.
    ///
    /// If a directory path is given, the whole directory including it's contents is added to the
    /// archive.
    ///
    /// If no entry exists at the given `src_path`, an error is returned.
    pub fn append_path<P, Q>(&mut self, path: P, src_path: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        // Append the path as file or directory
        if src_path.as_ref().is_file() {
            self.append_file(path, &mut File::open(src_path)?)
        } else if src_path.as_ref().is_dir() {
            self.append_dir(path, src_path)
        } else {
            // TODO: return a IO NotFound error here!
            panic!("Unable to append path to archive, not a file or directory");
        }
    }

    /// Append a file to the archive builder.
    pub fn append_file<P>(&mut self, path: P, file: &mut File) -> Result<()>
    where
        P: AsRef<Path>,
    {
        self.inner.append_file(path, file)
    }

    /// Append a directory to the archive builder.
    // TODO: Define a flag to add recursively or not
    pub fn append_dir<P, Q>(&mut self, path: P, src_path: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        self.inner.append_dir_all(path, src_path)
    }

    // TODO: some description
    pub fn finish(mut self) -> Result<()> {
        self.inner.finish()
    }
}
