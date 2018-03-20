use std::cmp::{max, min};
use std::fs::File;
use std::io::{
    self,
    BufReader,
    Cursor,
    Error as IoError,
    Read,
    Write,
};
use std::sync::{Arc, Mutex};

use openssl::symm::{
    Cipher,
    Crypter,
    Mode as CrypterMode,
};

/// The length in bytes of crytographic tags that are used.
const TAG_LEN: usize = 16;

// TODO: create a generic reader/writer wrapper for the the encryptor/decryptor.

/// A lazy file reader, that encrypts the file with the given `cipher`
/// and appends the cryptographic tag to the end of it.
///
/// This reader is lazy because the file data loaded from the system
/// and encrypted when it is read from the reader.
/// This greatly reduces memory usage for large files.
///
/// This reader encrypts the file data with an appended cryptographic tag.
///
/// The reader uses a small internal buffer as data is encrypted in blocks,
/// which may output more data than fits in the given buffer while reading.
/// The excess data is then returned on the next read.
pub struct EncryptedFileReader {
    /// The raw file that is read from.
    file: File,

    /// The cipher type used for encrypting.
    cipher: Cipher,

    /// The crypter used for encrypting the read file.
    crypter: Crypter,

    /// A tag cursor that reads the tag to append,
    /// when the file is fully read and the tag is known.
    tag: Option<Cursor<Vec<u8>>>,

    /// The internal buffer, containing encrypted data that has yet to be
    /// outputted to the reader. This data is always outputted before any new
    /// data is produced.
    internal_buf: Vec<u8>,
}

impl EncryptedFileReader {
    /// Construct a new reader for the given `file` with the given `cipher`.
    ///
    /// This method consumes twice the size of the file in memory while
    /// constructing, and constructs a reader that has a size similar to the
    /// file.
    ///
    /// It is recommended to wrap this reader in some sort of buffer, such as:
    /// `std::io::BufReader`
    pub fn new(file: File, cipher: Cipher, key: &[u8], iv: &[u8])
        -> Result<Self, io::Error>
    {
        // Build the crypter
        let crypter = Crypter::new(
            cipher,
            CrypterMode::Encrypt,
            key,
            Some(iv),
        )?;

        // Construct the encrypted reader
        Ok(
            EncryptedFileReader {
                file,
                cipher,
                crypter,
                tag: None,
                internal_buf: Vec::new(),
            }
        )
    }

    /// Read data from the internal buffer if there is any data in it, into
    /// the given `buf`.
    ///
    /// The number of bytes that were read into `buf` is returned.
    ///
    /// If there is no data to be read, or `buf` has a zero size, `0` is always
    /// returned.
    fn read_internal(&mut self, buf: &mut [u8]) -> usize {
        // Return if there is no data to read
        if self.internal_buf.is_empty() || buf.len() == 0 {
            return 0;
        }

        // Determine how much data will be read
        let len = min(buf.len(), self.internal_buf.len());

        // Slice the section we will read from, copy to the reader
        {
            let (out, _) = self.internal_buf.split_at(len);
            let (buf, _) = buf.split_at_mut(len);
            buf.copy_from_slice(out);
        }

        // Drain the read data from the internal buffer
        self.internal_buf.drain(..len);

        len
    }

    /// Read data directly from the file, and encrypt it.
    ///
    /// Because data may be encrypted in blocks, it is possible more data
    /// is produced than fits in the given `buf`. In that case the excess data
    /// is stored in an internal buffer, and is ouputted the next time being
    /// read from the reader.
    ///
    /// The number of bytes that is read into `buf` is returned.
    fn read_file_encrypted(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        // Get the block size, determine the buffer size, create a data buffer
        let block_size = self.cipher.block_size();
        let mut data = vec![0u8; buf.len()];

        // Read the file, return if nothing was read
        let len = self.file.read(&mut data)?;
        if len == 0 {
            return Ok(0);
        }

        // Create an encrypted buffer, truncate the data buffer
        let mut encrypted = vec![0u8; len + block_size];

        // Encrypt the data that was read
        let len = self.crypter.update(&data[..len], &mut encrypted)?;

        // Calculate how many bytes will be copied to the reader
        let out_len = min(buf.len(), len);

        // Fill the reader buffer
        let (out, remaining) = encrypted.split_at(out_len);
        let (buf, _) = buf.split_at_mut(out_len);
        buf.copy_from_slice(out);

        // Splice to the actual remaining bytes, store it for later
        let (store, _) = remaining.split_at(len - out_len);
        self.internal_buf.extend(store.iter());

        // Return the number of bytes read to the reader
        Ok(out_len)
    }

    /// Finalize the crypter once it is done encrypthing the whole file.
    /// This finalization step produces a tag that is placed after the
    /// encrypted file data.
    ///
    /// This step must be invoked to start reading the tag,
    /// and after it has been invoked no data must be encrypted anymore.
    ///
    /// This method must only be invoked once.
    fn finalize_file(&mut self) -> Result<(), io::Error> {
        // Finalize the crypter, catch any remaining output
        let mut output = vec![0u8; self.cipher.block_size()];
        let len = self.crypter.finalize(&mut output)?;

        // Move additional output in the internal buffer
        if len > 0 {
            self.internal_buf.extend(output.iter().take(len));
        }

        // Fetch the encryption tag, and create an internal reader for it
        let mut tag = vec![0u8; TAG_LEN];
        self.crypter.get_tag(&mut tag)?;
        self.tag = Some(Cursor::new(tag));

        Ok(())
    }
}

impl ExactLengthReader for EncryptedFileReader {
    /// Calculate the total length of the encrypted file with the appended
    /// tag.
    /// Useful in combination with some progress monitor, to determine how much
    /// of the file is read or for example; sent over the network.
    fn len(&self) -> Result<u64, io::Error> {
        Ok(self.file.metadata()?.len() + TAG_LEN as u64)
    }
}

/// The reader trait implementation.
impl Read for EncryptedFileReader {
    /// Read from the encrypted file, and then the encryption tag.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        // Read from the internal buffer, return full or splice to empty
        let len = self.read_internal(buf);
        if len >= buf.len() {
            return Ok(len);
        }
        let (_, buf) = buf.split_at_mut(len);

        // Keep track of the total number of read bytes, to return
        let mut total = len;

        // If the tag reader has been created, only read from that one
        if let Some(ref mut tag) = self.tag {
            return Ok(tag.read(buf)? + total);
        }

        // Read the encrypted file, return full or splice to empty
        let len = self.read_file_encrypted(buf)?;
        total += len;
        if len >= buf.len() {
            return Ok(total);
        }
        let (_, buf) = buf.split_at_mut(len);

        // Finalize the file crypter, and build the tag
        self.finalize_file()?;

        // Try to fill the remaining part of the buffer
        Ok(self.read(buf)? + total)
    }
}

// TODO: implement this some other way
unsafe impl Send for EncryptedFileReader {}

/// A reader wrapper, that measures the reading process for a reader with a
/// known length.
///
/// If the reader exceeds the initially specified length,
/// the reader will continue to allow reads.
/// The length property will grow accordingly.
///
/// The reader will only start producing `None` if the wrapped reader is doing
/// so.
pub struct ProgressReader<R> {
    /// The wrapped reader.
    inner: R,

    /// The total length of the reader.
    len: u64,

    /// The current reading progress.
    progress: u64,

    /// A reporter, to report the progress status to.
    reporter: Option<Arc<Mutex<ProgressReporter>>>,
}

impl<R: Read> ProgressReader<R> {
    /// Wrap the given reader with an exact length, in a progress reader.
    pub fn new(inner: R) -> Result<Self, IoError>
        where
            R: ExactLengthReader
    {
        Ok(
            Self {
                len: inner.len()?,
                inner,
                progress: 0,
                reporter: None,
            }
        )
    }

    /// Wrap the given reader with the given length in a progress reader.
    pub fn from(inner: R, len: u64) -> Self {
        Self {
            inner,
            len,
            progress: 0,
            reporter: None,
        }
    }

    /// Set the reporter to report the status to.
    pub fn set_reporter(&mut self, reporter: Arc<Mutex<ProgressReporter>>) {
        self.reporter = Some(reporter);
    }

    /// Get the current progress.
    pub fn progress(&self) -> u64 {
        self.progress
    }
}

impl<R: Read> Read for ProgressReader<R> {
    /// Read from the encrypted file, and then the encryption tag.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        // Read from the wrapped reader, increase the progress
        let len = self.inner.read(buf)?;
        self.progress += len as u64;

        // Keep the specified length in-bound
        if self.progress > self.len {
            self.len = self.progress;
        }

        // Report
        if let Some(reporter) = self.reporter.as_mut() {
            let progress = self.progress;
            let _ = reporter.lock().map(|mut r| r.progress(progress));
        }

        Ok(len)
    }
}

impl<R: Read> ExactLengthReader for ProgressReader<R> {
    // Return the specified length.
    fn len(&self) -> Result<u64, io::Error> {
        Ok(self.len)
    }
}

/// A progress reporter.
pub trait ProgressReporter: Send {
    /// Start the progress with the given total.
    fn start(&mut self, total: u64);

    /// A progress update.
    fn progress(&mut self, progress: u64);

    /// Finish the progress.
    fn finish(&mut self);
}

/// A trait for readers, to get the exact length of a reader.
pub trait ExactLengthReader {
    /// Get the exact length of the reader in bytes.
    fn len(&self) -> Result<u64, io::Error>;
}

impl<R: ExactLengthReader + Read> ExactLengthReader for BufReader<R> {
    fn len(&self) -> Result<u64, io::Error> {
        self.get_ref().len()
    }
}

/// A lazy file writer, that decrypt the file with the given `cipher`
/// and verifies it with the tag appended to the end of the input data.
///
/// This writer is lazy because the input data is decrypted and written to the
/// specified file on the fly, instead of buffering all the data first.
/// This greatly reduces memory usage for large files.
///
/// The length of the input data (including the appended tag) must be given
/// when this reader is initialized. When all data including the tag is read,
/// the decrypted data is verified with the tag. If the tag doesn't match the
/// decrypted data, a write error is returned on the last write.
/// This writer will never write more bytes than the length initially
/// specified.
///
/// This reader encrypts the input data with the given key and input vector.
///
/// A failed writing implies that no data could be written, or that the data
/// wasn't successfully decrypted because of an decryption or tag matching
/// error. Such a fail means that the file will be incomplete or corrupted,
/// and should therefore be removed from the disk.
///
/// It is highly recommended to invoke the `verified()` method after writing
/// the file, to ensure the written file is indeed complete and fully verified.
pub struct EncryptedFileWriter {
    /// The file to write the decrypted data to.
    file: File,

    /// The number of bytes that have currently been written to this writer.
    cur: usize,

    /// The length of all the data, which includes the file data and the
    /// appended tag.
    len: usize,

    /// The cipher type used for decrypting.
    cipher: Cipher,

    /// The crypter used for decrypting the data.
    crypter: Crypter,

    /// A buffer for the tag.
    tag_buf: Vec<u8>,

    /// A boolean that defines whether the decrypted data has successfully
    /// been verified.
    verified: bool,
}

impl EncryptedFileWriter {
    /// Construct a new encrypted file writer.
    ///
    /// The file to write to must be given to `file`, which must be open for
    /// writing. The total length of the input data in bytes must be given to
    /// `len`, which includes both the file bytes and the appended tag.
    ///
    /// For decryption, a `cipher`, `key` and `iv` must also be given.
    pub fn new(file: File, len: usize, cipher: Cipher, key: &[u8], iv: &[u8])
        -> Result<Self, io::Error>
    {
        // Build the crypter
        let crypter = Crypter::new(
            cipher,
            CrypterMode::Decrypt,
            key,
            Some(iv),
        )?;

        // Construct the encrypted reader
        Ok(
            EncryptedFileWriter {
                file,
                cur: 0,
                len,
                cipher,
                crypter,
                tag_buf: Vec::with_capacity(TAG_LEN),
                verified: false,
            }
        )
    }

    /// Check wheher the complete tag is buffered.
    pub fn has_tag(&self) -> bool {
        self.tag_buf.len() >= TAG_LEN
    }

    /// Check whether the decrypted data is succesfsully verified.
    ///
    /// If this method returns true the following is implied:
    /// - The complete file has been written.
    /// - The complete file was successfully decrypted.
    /// - The included tag matches the decrypted file.
    ///
    /// It is highly recommended to invoke this method and check the
    /// verification after writing the file using this writer.
    pub fn verified(&self) -> bool {
        self.verified
    }
}

impl ExactLengthReader for EncryptedFileWriter {
    fn len(&self) -> Result<u64, IoError> {
        Ok(self.len as u64)
    }
}

/// The writer trait implementation.
impl Write for EncryptedFileWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        // Do not write anything if the tag was already written
        if self.verified() || self.has_tag() {
            return Ok(0);
        }

        // Determine how many file and tag bytes we still need to process
        let file_bytes = max(self.len - TAG_LEN - self.cur, 0);
        let tag_bytes = TAG_LEN - self.tag_buf.len();

        // Split the input buffer
        let (file_buf, tag_buf) = buf.split_at(min(file_bytes, buf.len()));

        // Read from the file buf
        if !file_buf.is_empty() {
            // Create a decrypted buffer, with the proper size
            let block_size = self.cipher.block_size();
            let mut decrypted = vec![0u8; file_bytes + block_size];

            // Decrypt bytes
            // TODO: catch error in below statement
            let len = self.crypter.update(
                file_buf,
                &mut decrypted,
            )?;

            // Write to the file
            self.file.write_all(&decrypted[..len])?;
        }

        // Read from the tag part to fill the tag buffer
        if !tag_buf.is_empty() {
            self.tag_buf.extend(tag_buf.iter().take(tag_bytes));
        }

        // Verify the tag once it has been buffered completely
        if self.has_tag() {
            // Set the tag
            self.crypter.set_tag(&self.tag_buf)?;

            // Create a buffer for any remaining data
            let block_size = self.cipher.block_size();
            let mut extra = vec![0u8; block_size];

            // Finalize, write all remaining data
            let len = self.crypter.finalize(&mut extra)?;
            self.file.write_all(&extra[..len])?;

            // Set the verified flag
            self.verified = true;
        }

        // Compute how many bytes were written
        let len = file_buf.len() + min(tag_buf.len(), TAG_LEN);
        self.cur += len;
        Ok(len)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.file.flush()
    }
}
















/// A writer wrapper, that measures the reading process for a writer with a
/// known length.
///
/// If the writer exceeds the initially specified length,
/// the writer will continue to allow reads.
/// The length property will grow accordingly.
///
/// The writer will only start producing `None` if the wrapped writer is doing
/// so.
pub struct ProgressWriter<W> {
    /// The wrapped writer.
    inner: W,

    /// The total length of the writer.
    len: u64,

    /// The current reading progress.
    progress: u64,

    /// A reporter, to report the progress status to.
    reporter: Option<Arc<Mutex<ProgressReporter>>>,
}

impl<W: Write> ProgressWriter<W> {
    /// Wrap the given writer with an exact length, in a progress writer.
    pub fn new(inner: W) -> Result<Self, IoError>
        where
            W: ExactLengthReader
    {
        Ok(
            Self {
                len: inner.len()?,
                inner,
                progress: 0,
                reporter: None,
            }
        )
    }

    /// Wrap the given writer with the given length in a progress writer.
    pub fn from(inner: W, len: u64) -> Self {
        Self {
            inner,
            len,
            progress: 0,
            reporter: None,
        }
    }

    /// Set the reporter to report the status to.
    pub fn set_reporter(&mut self, reporter: Arc<Mutex<ProgressReporter>>) {
        self.reporter = Some(reporter);
    }

    /// Get the current progress.
    pub fn progress(&self) -> u64 {
        self.progress
    }

    /// Unwrap the inner from the progress writer.
    pub fn unwrap(self) -> W {
        self.inner
    }
}

impl<W: Write> Write for ProgressWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        // Write from the wrapped writer, increase the progress
        let len = self.inner.write(buf)?;
        self.progress += len as u64;

        // Keep the specified length in-bound
        if self.progress > self.len {
            self.len = self.progress;
        }

        // Report
        if let Some(reporter) = self.reporter.as_mut() {
            let progress = self.progress;
            let _ = reporter.lock().map(|mut r| r.progress(progress));
        }

        Ok(len)
    }

    fn flush(&mut self) -> Result<(), IoError> {
        self.inner.flush()
    }
}

impl<W: Write> ExactLengthReader for ProgressWriter<W> {
    // Return the specified length.
    fn len(&self) -> Result<u64, io::Error> {
        Ok(self.len)
    }
}
