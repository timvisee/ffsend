use std::cmp::min;
use std::fs::File;
use std::io::{self, Cursor, Read};

use openssl::symm::{
    Cipher,
    Crypter,
    Mode as CrypterMode,
};

/// The length in bytes of crytographic tags that are used.
const TAG_LEN: usize = 16;

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
pub struct EncryptedFileReaderTagged {
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

impl EncryptedFileReaderTagged {
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
            EncryptedFileReaderTagged {
                file,
                cipher,
                crypter,
                tag: None,
                internal_buf: Vec::new(),
            }
        )
    }

    /// Calculate the total length of the encrypted file with the appended
    /// tag.
    /// Useful in combination with some progress monitor, to determine how much
    /// of the file is read or for example; sent over the network.
    pub fn len(&self) -> Result<u64, io::Error> {
        Ok(self.file.metadata()?.len() + TAG_LEN as u64)
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
        data.truncate(len);

        // Encrypt the data that was read
        let len = self.crypter.update(&data, &mut encrypted)?;

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

/// The reader trait implementation.
impl Read for EncryptedFileReaderTagged {
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
unsafe impl Send for EncryptedFileReaderTagged {}
