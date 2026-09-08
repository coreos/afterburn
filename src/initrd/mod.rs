//! Agent logic running at early boot.
//!
//! This is run early-on in initrd, possibly before networking and other
//! services are configured, so it may not be able to use all usual metadata
//! fetcher.

use crate::providers::kubevirt;
use crate::providers::proxmoxve;
use crate::providers::vmware::VmwareProvider;
use crate::providers::MetadataProvider;
use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Path to cmdline.d fragment for network kernel arguments.
static KARGS_PATH: &str = "/etc/cmdline.d/50-afterburn-network-kargs.conf";

/// Path to the environment file consumed by systemd-network-generator.
///
/// systemd-network-generator only parses /proc/cmdline (or the command line
/// provided through the SYSTEMD_PROC_CMDLINE environment variable), so the
/// kargs we inject into Dracut's cmdline.d fragment above are invisible to it.
/// We therefore expose the augmented command line through this file, which a
/// shipped generator drop-in references via EnvironmentFile=.
static GENERATOR_ENV_PATH: &str = "/run/afterburn/network-generator.env";

/// Path to the kernel command line.
static PROC_CMDLINE_PATH: &str = "/proc/cmdline";

/// Fetch network kargs for the given provider.
pub(crate) fn fetch_network_kargs(provider: &str) -> Result<Option<String>> {
    match provider {
        "vmware" => VmwareProvider::try_new()?.rd_network_kargs(),
        "proxmoxve" => proxmoxve::try_config_drive_else_leave()?.rd_network_kargs(),
        "kubevirt" => kubevirt::try_new_provider_else_noop()?.rd_network_kargs(),
        _ => Ok(None),
    }
}

/// Write network kargs into a cmdline.d fragment and hand the augmented kernel
/// command line to systemd-network-generator.
pub(crate) fn write_network_kargs(kargs: &str) -> Result<()> {
    write_cmdline_fragment(KARGS_PATH, kargs)?;
    write_generator_env(GENERATOR_ENV_PATH, PROC_CMDLINE_PATH, kargs)
}

/// Write network kargs into a Dracut cmdline.d fragment.
fn write_cmdline_fragment(path: &str, kargs: &str) -> Result<()> {
    let mut fragment_file =
        File::create(path).with_context(|| format!("failed to create file {path:?}"))?;

    fragment_file
        .write_all(kargs.as_bytes())
        .context("failed to write network arguments fragment")?;
    fragment_file
        .write_all(b"\n")
        .context("failed to write trailing newline")?;

    Ok(())
}

/// Write the augmented kernel command line for systemd-network-generator.
///
/// The generator does not read Dracut's cmdline.d fragments, so we combine the
/// real kernel command line with the injected kargs and expose the result via
/// SYSTEMD_PROC_CMDLINE in an environment file consumed by the generator's
/// drop-in.
fn write_generator_env(path: &str, proc_cmdline_path: &str, kargs: &str) -> Result<()> {
    let kargs = kargs.trim();
    if kargs.is_empty() {
        return Ok(());
    }

    let base = fs::read_to_string(proc_cmdline_path)
        .with_context(|| format!("failed to read {proc_cmdline_path:?}"))?;
    let base = base.trim();
    let cmdline = if base.is_empty() {
        kargs.to_string()
    } else {
        format!("{base} {kargs}")
    };

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {parent:?}"))?;
    }

    let mut env_file =
        File::create(path).with_context(|| format!("failed to create file {path:?}"))?;
    writeln!(env_file, "SYSTEMD_PROC_CMDLINE={cmdline}")
        .context("failed to write network generator environment file")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_proc_cmdline(dir: &Path, contents: &str) -> String {
        let path = dir.join("proc_cmdline");
        fs::write(&path, contents).unwrap();
        path.to_str().unwrap().to_owned()
    }

    #[test]
    fn cmdline_fragment_appends_newline() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("50-afterburn-network-kargs.conf");
        let path_str = path.to_str().unwrap();

        write_cmdline_fragment(path_str, "ip=dhcp rd.neednet=1").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "ip=dhcp rd.neednet=1\n");
    }

    #[test]
    fn generator_env_augments_proc_cmdline() {
        let dir = tempdir().unwrap();
        let proc_cmdline = write_proc_cmdline(dir.path(), "root=/dev/sda1 console=ttyS0\n");
        let env_path = dir.path().join("network-generator.env");
        let env_path_str = env_path.to_str().unwrap();

        write_generator_env(env_path_str, &proc_cmdline, "ip=dhcp").unwrap();

        assert_eq!(
            fs::read_to_string(&env_path).unwrap(),
            "SYSTEMD_PROC_CMDLINE=root=/dev/sda1 console=ttyS0 ip=dhcp\n"
        );
    }

    #[test]
    fn generator_env_trims_kargs_and_cmdline() {
        let dir = tempdir().unwrap();
        let proc_cmdline = write_proc_cmdline(dir.path(), "  root=/dev/sda1  \n");
        let env_path = dir.path().join("network-generator.env");
        let env_path_str = env_path.to_str().unwrap();

        write_generator_env(env_path_str, &proc_cmdline, "  ip=dhcp  ").unwrap();

        assert_eq!(
            fs::read_to_string(&env_path).unwrap(),
            "SYSTEMD_PROC_CMDLINE=root=/dev/sda1 ip=dhcp\n"
        );
    }

    #[test]
    fn generator_env_handles_empty_proc_cmdline() {
        let dir = tempdir().unwrap();
        let proc_cmdline = write_proc_cmdline(dir.path(), "");
        let env_path = dir.path().join("network-generator.env");
        let env_path_str = env_path.to_str().unwrap();

        write_generator_env(env_path_str, &proc_cmdline, "ip=dhcp").unwrap();

        assert_eq!(
            fs::read_to_string(&env_path).unwrap(),
            "SYSTEMD_PROC_CMDLINE=ip=dhcp\n"
        );
    }

    #[test]
    fn generator_env_skips_when_kargs_empty() {
        let dir = tempdir().unwrap();
        // A non-existent proc cmdline path proves it is never read when the
        // kargs are empty (the early return happens before the read).
        let proc_cmdline = dir.path().join("does-not-exist");
        let env_path = dir.path().join("network-generator.env");
        let env_path_str = env_path.to_str().unwrap();

        write_generator_env(env_path_str, proc_cmdline.to_str().unwrap(), "   ").unwrap();

        assert!(!env_path.exists());
    }

    #[test]
    fn generator_env_creates_parent_directory() {
        let dir = tempdir().unwrap();
        let proc_cmdline = write_proc_cmdline(dir.path(), "root=/dev/sda1\n");
        let env_path = dir.path().join("afterburn").join("network-generator.env");
        let env_path_str = env_path.to_str().unwrap();

        write_generator_env(env_path_str, &proc_cmdline, "ip=dhcp").unwrap();

        assert!(env_path.exists());
        assert_eq!(
            fs::read_to_string(&env_path).unwrap(),
            "SYSTEMD_PROC_CMDLINE=root=/dev/sda1 ip=dhcp\n"
        );
    }
}
