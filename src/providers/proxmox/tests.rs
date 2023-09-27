use super::ProxmoxCloudConfig;
use crate::{
    network::{self, NetworkRoute},
    providers::MetadataProvider,
};
use ipnetwork::IpNetwork;
use openssh_keys::PublicKey;
use pnet_base::MacAddr;
use std::{net::IpAddr, path::Path, str::FromStr};

#[test]
fn test_attributes() {
    let config = ProxmoxCloudConfig::try_new(Path::new("tests/fixtures/proxmox/static"))
        .expect("cannot parse config");
    let attributes = config.attributes().expect("cannot get hostname");

    assert_eq!(
        attributes["AFTERBURN_PROXMOX_HOSTNAME"],
        "dummy".to_string()
    );

    assert_eq!(
        attributes["AFTERBURN_PROXMOX_INSTANCE_ID"],
        "15a9919cb91024fbd1d70fa07f0efa749cbba03b".to_string()
    );

    assert_eq!(
        attributes["AFTERBURN_PROXMOX_IPV4"],
        "192.168.1.1".to_string()
    );

    assert_eq!(
        attributes["AFTERBURN_PROXMOX_IPV6"],
        "2001:db8:85a3::8a2e:370:0".to_string()
    );
}

#[test]
fn test_hostname() {
    let config = ProxmoxCloudConfig::try_new(Path::new("tests/fixtures/proxmox/dhcp"))
        .expect("cannot parse config");

    assert_eq!(
        config.hostname().expect("cannot get hostname"),
        Some("dummy".to_string())
    );
}

#[test]
fn test_ssh_keys() {
    let test_ssh_key = PublicKey::from_str("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDd1hElre4j44sbmULXyO5j6dRnkRFCMjEGtRSy2SuvFD8WyB5uectcEMvz7ORhQIVbPlz94wFjpSX5wl/gmSKL/7GOyerJo0Y2cvyjJJahuDn+JnIL0tT0HS1pJ5iJqQpxXeOAzMK5Heum+uGw9BzbiUHnRzjJr8Ltx4CAGMfubevD4SX32Q8BTQiaU4ZnGtdHo16pWwRsq1f6/UtL4gDCni9vm8QmmGDRloi/pBn1csjKw+volFyu/kSEmGLWow6NuT6TrhGAbMKas5HfYq0Mn3LGPZL7XjqJQ6CO0TzkG/BNplZT2tiwHtsvXsbePTp4ZUi4dkCMz2xR4eikaI1V dummy@dummy.local").unwrap();
    let config = ProxmoxCloudConfig::try_new(Path::new("tests/fixtures/proxmox/dhcp"))
        .expect("cannot parse config");

    assert_eq!(
        config.ssh_keys().expect("cannot get ssh keys"),
        vec![test_ssh_key]
    );
}

#[test]
fn test_network_dhcp() {
    let config = ProxmoxCloudConfig::try_new(Path::new("tests/fixtures/proxmox/dhcp"))
        .expect("cannot parse config");

    assert_eq!(
        config.networks().expect("cannot get networks"),
        vec![network::Interface {
            name: Some("eth0".to_owned()),
            mac_address: Some(MacAddr::from_str("01:23:45:67:89:00").unwrap()),
            path: None,
            priority: 20,
            nameservers: vec![
                IpAddr::from_str("1.1.1.1").unwrap(),
                IpAddr::from_str("8.8.8.8").unwrap()
            ],
            ip_addresses: vec![],
            routes: vec![],
            bond: None,
            unmanaged: false,
            required_for_online: None
        }]
    );
}

#[test]
fn test_network_static() {
    let config = ProxmoxCloudConfig::try_new(Path::new("tests/fixtures/proxmox/static"))
        .expect("cannot parse config");

    assert_eq!(
        config.networks().expect("cannot get networks"),
        vec![
            network::Interface {
                name: Some("eth0".to_owned()),
                mac_address: Some(MacAddr::from_str("01:23:45:67:89:00").unwrap()),
                path: None,
                priority: 20,
                nameservers: vec![
                    IpAddr::from_str("1.1.1.1").unwrap(),
                    IpAddr::from_str("8.8.8.8").unwrap()
                ],
                ip_addresses: vec![
                    IpNetwork::from_str("192.168.1.1/24").unwrap(),
                    IpNetwork::from_str("2001:0db8:85a3:0000:0000:8a2e:0370:0/24").unwrap(),
                ],
                routes: vec![
                    NetworkRoute {
                        destination: IpNetwork::from_str("0.0.0.0/0").unwrap(),
                        gateway: IpAddr::from_str("192.168.1.254").unwrap(),
                    },
                    NetworkRoute {
                        destination: IpNetwork::from_str("::/0").unwrap(),
                        gateway: IpAddr::from_str("2001:0db8:85a3:0000:0000:8a2e:0370:9999")
                            .unwrap(),
                    },
                ],
                bond: None,
                unmanaged: false,
                required_for_online: None
            },
            network::Interface {
                name: Some("eth1".to_owned()),
                mac_address: Some(MacAddr::from_str("01:23:45:67:89:99").unwrap()),
                path: None,
                priority: 20,
                nameservers: vec![],
                ip_addresses: vec![
                    IpNetwork::from_str("192.168.42.1/24").unwrap(),
                    IpNetwork::from_str("2001:0db8:85a3:0000:0000:8a2e:4242:0/24").unwrap(),
                ],
                routes: vec![
                    NetworkRoute {
                        destination: IpNetwork::from_str("0.0.0.0/0").unwrap(),
                        gateway: IpAddr::from_str("192.168.42.254").unwrap(),
                    },
                    NetworkRoute {
                        destination: IpNetwork::from_str("::/0").unwrap(),
                        gateway: IpAddr::from_str("2001:0db8:85a3:0000:0000:8a2e:4242:9999")
                            .unwrap(),
                    },
                ],
                bond: None,
                unmanaged: false,
                required_for_online: None
            },
        ]
    );
}
