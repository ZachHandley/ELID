//! Progress bar utilities for long-running operations.

use indicatif::{ProgressBar, ProgressStyle};

/// Creates a progress bar for operations with known total size.
///
/// Only displays a progress bar if:
/// - `enabled` is true
/// - `total` is >= 1000 (to avoid cluttering output for small operations)
///
/// # Arguments
///
/// * `total` - Total number of items to process
/// * `enabled` - Whether progress bars are enabled (typically from CLI flag)
///
/// # Returns
///
/// `Some(ProgressBar)` if conditions are met, `None` otherwise
pub fn create_progress_bar(total: u64, enabled: bool) -> Option<ProgressBar> {
    if !enabled || total < 1000 {
        return None;
    }

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({per_sec}, ETA {eta})")
            .unwrap()
            .progress_chars("=>-"),
    );
    Some(pb)
}

/// Updates the progress bar by incrementing the current position.
///
/// If the progress bar is `None`, this is a no-op.
///
/// # Arguments
///
/// * `pb` - Optional progress bar reference
/// * `amount` - Amount to increment (typically 1 per item processed)
pub fn update_progress(pb: &Option<ProgressBar>, amount: u64) {
    if let Some(pb) = pb {
        pb.inc(amount);
    }
}

/// Finishes the progress bar with a completion message.
///
/// If the progress bar is `None`, this is a no-op.
///
/// # Arguments
///
/// * `pb` - Optional progress bar to finish
pub fn finish_progress(pb: Option<ProgressBar>) {
    if let Some(pb) = pb {
        pb.finish_with_message("done");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_disabled() {
        let pb = create_progress_bar(10000, false);
        assert!(pb.is_none());
    }

    #[test]
    fn test_progress_bar_small_total() {
        let pb = create_progress_bar(999, true);
        assert!(pb.is_none());
    }

    #[test]
    fn test_progress_bar_enabled() {
        let pb = create_progress_bar(1000, true);
        assert!(pb.is_some());
    }

    #[test]
    fn test_update_none() {
        update_progress(&None, 1);
        // Should not panic
    }

    #[test]
    fn test_finish_none() {
        finish_progress(None);
        // Should not panic
    }
}
