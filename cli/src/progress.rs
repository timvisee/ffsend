extern crate pbr;

use std::io::Stdout;

use ffsend_api::reader::ProgressReporter;
use self::pbr::{
    ProgressBar as Pbr,
    Units,
};

/// A progress bar reporter.
pub struct ProgressBar {
    bar: Option<Pbr<Stdout>>,
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            bar: None,
        }
    }
}

impl ProgressReporter for ProgressBar {
    /// Start the progress with the given total.
    fn start(&mut self, total: u64) {
        // Initialize the progress bar
        let mut bar = Pbr::new(total);
        bar.set_units(Units::Bytes);

        self.bar = Some(bar);
    }

    /// A progress update.
    fn progress(&mut self, progress: u64) {
        self.bar.as_mut()
            .expect("progress bar not yet instantiated, cannot set progress")
            .set(progress);
    }

    /// Finish the progress.
    fn finish(&mut self) {
        self.bar.as_mut()
            .expect("progress bar not yet instantiated")
            // TODO: print a proper message here
            .finish_print("DONE");
    }
}
