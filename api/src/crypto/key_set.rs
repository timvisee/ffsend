use openssl::symm::Cipher;

use super::{b64, rand_bytes};
use super::hdkf::{derive_auth_key, derive_file_key, derive_meta_key};

pub struct KeySet {
    /// A secret.
    secret: [u8; 16],

    /// Input vector.
    iv: [u8; 12],

    /// A derived file encryption key.
    file_key: Option<Vec<u8>>,

    /// A derived authentication key.
    auth_key: Option<Vec<u8>>,

    /// A derived metadata key.
    meta_key: Option<Vec<u8>>,
}

impl KeySet {
    /// Construct a new key, with the given `secret` and `iv`.
    pub fn new(secret: [u8; 16], iv: [u8; 12]) -> Self {
        Self {
            secret,
            iv,
            file_key: None,
            auth_key: None,
            meta_key: None,
        }
    }

    /// Generate a secure new key.
    ///
    /// If `derive` is `true`, file, authentication and metadata keys will be
    /// derived from the generated secret. 
    pub fn generate(derive: bool) -> Self {
        // Allocate two keys
        let mut secret = [0u8; 16];
        let mut iv = [0u8; 12];

        // Generate the secrets
        rand_bytes(&mut secret)
            .expect("failed to generate crypto secure random secret");
        rand_bytes(&mut iv)
            .expect("failed to generate crypto secure random input vector");

        // Create the key
        let mut key = Self::new(secret, iv);

        // Derive
        if derive {
            key.derive();
        }

        key
    }

    /// Derive a file, authentication and metadata key.
    pub fn derive(&mut self) {
        self.file_key = Some(derive_file_key(&self.secret));
        self.auth_key = Some(derive_auth_key(&self.secret, None, None));
        self.meta_key = Some(derive_meta_key(&self.secret));
    }

    /// Get the secret key.
    pub fn secret(&self) -> &[u8] {
        &self.secret
    }

    /// Get the secret key as URL-safe base64 encoded string.
    pub fn secret_encoded(&self) -> String {
       b64::encode(self.secret())
    }

    /// Get the input vector.
    pub fn iv(&self) -> &[u8] {
        &self.iv
    }

    /// Get the file encryption key, if derived.
    pub fn file_key(&self) -> Option<&Vec<u8>> {
        self.file_key.as_ref()
    }

    /// Get the authentication encryption key, if derived.
    pub fn auth_key(&self) -> Option<&Vec<u8>> {
        self.auth_key.as_ref()
    }

    /// Get the authentication encryption key, if derived,
    /// as URL-safe base64 encoded string.
    pub fn auth_key_encoded(&self) -> Option<String> {
        self.auth_key().map(|key| b64::encode(key))
    }

    /// Get the metadata encryption key, if derived.
    pub fn meta_key(&self) -> Option<&Vec<u8>> {
        self.meta_key.as_ref()
    }

    /// Get the cipher type to use in combination with these keys.
    pub fn cipher() -> Cipher {
        Cipher::aes_128_gcm()
    }
}
