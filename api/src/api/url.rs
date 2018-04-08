use url::Url;

use file::remote_file::RemoteFile;

/// A struct, that helps building URLs for communicating with a remote host.
pub struct UrlBuilder;

impl UrlBuilder {
    /// Get the download URL of the given file.
    /// This URL is identical to the share URL, a term used in this API.
    /// Set `secret` to `true`, to include it in the URL if known.
    pub fn download(file: &RemoteFile, secret: bool) -> Url {
        // Get the share URL, and update the secret fragment
        let mut url = file.url().clone();
        if secret && file.has_secret() {
            url.set_fragment(Some(&file.secret()));
        } else {
            url.set_fragment(None);
        }

        url
    }

    /// Generate an API file URL, with the given endpoint.
    /// The endpoint should not contain any slashes.
    ///
    /// Valid endpoints may be 'metadata', 'download' or for example
    /// 'password'.
    fn api(endpoint: &str, file: &RemoteFile) -> Url {
        // Get the share URL, and add the secret fragment
        let mut url = file.url().clone();
        url.set_path(format!("/api/{}/{}", endpoint, file.id()).as_str());
        url.set_fragment(None);

        url
    }

    /// Get the API metadata URL for the given file.
    pub fn api_metadata(file: &RemoteFile) -> Url {
        Self::api("metadata", file)
    }

    /// Get the API download URL for the given file.
    pub fn api_download(file: &RemoteFile) -> Url {
        Self::api("download", file)
    }

    /// Get the API password URL for the given file.
    pub fn api_password(file: &RemoteFile) -> Url {
        Self::api("password", file)
    }

    /// Get the API params URL for the given file.
    pub fn api_params(file: &RemoteFile) -> Url {
        Self::api("params", file)
    }

    /// Get the API info URL for the given file.
    pub fn api_info(file: &RemoteFile) -> Url {
        Self::api("info", file)
    }

    /// Get the API exists URL for the given file.
    pub fn api_exists(file: &RemoteFile) -> Url {
        Self::api("exists", file)
    }

    /// Get the API delete URL for the given file.
    pub fn api_delete(file: &RemoteFile) -> Url {
        Self::api("delete", file)
    }
}
