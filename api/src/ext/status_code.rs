use reqwest::StatusCode;

/// Reqwest status code extention, to easily retrieve an error message.
pub trait StatusCodeExt {
    /// Build a basic error message based on the status code.
    fn err_text(&self) -> String;
}

impl StatusCodeExt for StatusCode {
    fn err_text(&self) -> String {
        self.canonical_reason()
            .map(|text| format!("{} {}", self.as_u16(), text))
            .unwrap_or_else(|| format!("{}", self.as_u16()))
    }
}
