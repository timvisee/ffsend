use reqwest::StatusCode;

/// Reqwest status code extention, to easily retrieve an error message.
pub trait StatusCodeExt {
    /// Build a basic error message based on the status code.
    fn err_text(&self) -> String;
}

impl StatusCodeExt for StatusCode {
    fn err_text(&self) -> String {
        self.canonical_reason()
            .map(|text| text.to_owned())
            .unwrap_or(format!("{}", self.as_u16()))
    }
}
