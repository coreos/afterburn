//! Helpers for mounting and unmounting.

use crate::retry;
use anyhow::{Context, Result};
use nix::mount;
use slog_scope::{debug, warn};
use std::path::Path;
use std::process::Command;

/// Try to unmount an existing target mountpoint.
///
/// This can internally retry in case of transient errors.
pub(crate) fn unmount(target: &Path, retries: u8) -> Result<()> {
    let driver = retry::Retry::new().max_retries(retries);
    driver.retry(|attempt| {
        debug!(
            "unmounting '{}': attempt #{}",
            target.display(),
            attempt + 1
        );
        let res = mount::umount(target)
            .with_context(|| format!("failed to unmount '{}'", target.display()));

        if let Err(ref e) = res {
            debug!("{:?}", e);
        };
        res
    })
}

/// Try to mount a filesystem.
///
/// This can internally wait for udev events settling and retry in case of transient errors.
pub(crate) fn mount_ro(source: &Path, target: &Path, fstype: &str, retries: u8) -> Result<()> {
    let driver = retry::Retry::new().max_retries(retries);
    driver.retry(|attempt| {
        debug!("mounting '{}': attempt #{}", source.display(), attempt + 1);
        let res = mount::mount(
            Some(source),
            target,
            Some(fstype),
            mount::MsFlags::MS_RDONLY,
            None::<&str>,
        )
        .with_context(|| {
            format!(
                "failed to mount (read-only) source '{}' to target '{}', with type '{}'",
                source.display(),
                target.display(),
                fstype
            )
        });

        // If mounting failed, yield back and give a chance to any
        // pending udev events to be processed.
        if let Err(ref e) = res {
            debug!("{:?}", e);
            settle_udev(None)
        };
        res
    })
}

/// Wait for udev queue to settle, ignoring any errors.
fn settle_udev(timeout: Option<u8>) {
    let mut cmd = Command::new("udevadm");
    cmd.arg("settle");
    // If none, udevadm default is 120s.
    if let Some(t) = timeout {
        cmd.arg(format!("--timeout={}", t));
    }

    match cmd.output() {
        Err(e) => warn!("failed to run udevadm settle: {}", e),
        Ok(out) => {
            if !out.status.success() {
                warn!(
                    "udevadm settle failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
        }
    };
}
