use super::ProxmoxCloudConfig;
use crate::providers::MetadataProvider;
use openssh_keys::PublicKey;
use std::{path::Path, str::FromStr};

#[test]
fn test_dhcp() {
    let test_ssh_key = PublicKey::from_str("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDd1hElre4j44sbmULXyO5j6dRnkRFCMjEGtRSy2SuvFD8WyB5uectcEMvz7ORhQIVbPlz94wFjpSX5wl/gmSKL/7GOyerJo0Y2cvyjJJahuDn+JnIL0tT0HS1pJ5iJqQpxXeOAzMK5Heum+uGw9BzbiUHnRzjJr8Ltx4CAGMfubevD4SX32Q8BTQiaU4ZnGtdHo16pWwRsq1f6/UtL4gDCni9vm8QmmGDRloi/pBn1csjKw+volFyu/kSEmGLWow6NuT6TrhGAbMKas5HfYq0Mn3LGPZL7XjqJQ6CO0TzkG/BNplZT2tiwHtsvXsbePTp4ZUi4dkCMz2xR4eikaI1V dummy@dummy.local").unwrap();
    let config = ProxmoxCloudConfig::try_new(Path::new("tests/fixtures/proxmox/dhcp"))
        .expect("cannot parse config");

    assert_eq!(
        config.hostname().expect("cannot get hostname"),
        Some("dummy".to_string())
    );

    assert_eq!(
        config.ssh_keys().expect("cannot get ssh keys"),
        vec![test_ssh_key]
    );

    assert_eq!(config.networks().expect("cannot get networks"), vec![]);
}
