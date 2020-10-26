use std::io::{stderr, Stderr};
use std::time::Duration;

use ffsend_api::pipe::ProgressReporter;
use pbr::{ProgressBar as Pbr, Units};

/// The refresh rate of the progress bar, in milliseconds.
const PROGRESS_BAR_FPS_MILLIS: u64 = 200;

/// A progress bar reporter.
pub struct ProgressBar<'a> {
    progress_bar: Option<Pbr<Stderr>>,
    msg_progress: &'a str,
    msg_finish: &'a str,
}

impl<'a> ProgressBar<'a> {
    /// Construct a new progress bar, with the given messages.
    pub fn new(msg_progress: &'a str, msg_finish: &'a str) -> ProgressBar<'a> {
        Self {
            progress_bar: None,
            msg_progress,
            msg_finish,
        }
    }

    /// Construct a new progress bar for uploading.
    pub fn new_upload() -> ProgressBar<'a> {
        Self::new("Encrypt & Upload ", "Upload complete")
    }

    /// Construct a new progress bar for downloading.
    pub fn new_download() -> ProgressBar<'a> {
        Self::new("Download & Decrypt ", "Download complete")
    }
}

impl<'a> ProgressReporter for ProgressBar<'a> {
    /// Start the progress with the given total.
    fn start(&mut self, total: u64) {
        // Initialize the progress bar
        let mut progress_bar = Pbr::on(stderr(), total);
        progress_bar.set_max_refresh_rate(Some(Duration::from_millis(PROGRESS_BAR_FPS_MILLIS)));
        progress_bar.set_units(Units::Bytes);
        progress_bar.message(self.msg_progress);

        self.progress_bar = Some(progress_bar);
    }

    /// A progress update.
    fn progress(&mut self, progress: u64) {
        self.progress_bar
            .as_mut()
            .expect("progress bar not yet instantiated, cannot set progress")
            .set(progress);
    }

    /// Finish the progress.
    fn finish(&mut self) {
        let progress_bar = self
            .progress_bar
            .as_mut()
            .expect("progress bar not yet instantiated");

        #[cfg(not(target_os = "windows"))]
        progress_bar.finish_print(self.msg_finish);
        #[cfg(target_os = "windows")]
        {
            progress_bar.finish_println(self.msg_finish);
            eprintln!();
        }
    }
}
