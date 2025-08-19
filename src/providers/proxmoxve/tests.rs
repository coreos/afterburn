use super::ProxmoxVECloudConfig;
use crate::{
    network::{self, DhcpSetting, NetworkRoute},
    providers::MetadataProvider,
};
use ipnetwork::IpNetwork;
use openssh_keys::PublicKey;
use pnet_base::MacAddr;
use std::{net::IpAddr, path::Path, str::FromStr};

#[test]
fn test_attributes() {
    let config = ProxmoxVECloudConfig::try_new(Path::new("tests/fixtures/proxmoxve/static"))
        .expect("cannot parse config");
    let attributes = config.attributes().expect("cannot get hostname");

    assert_eq!(attributes["PROXMOXVE_HOSTNAME"], "dummy".to_string());

    assert_eq!(
        attributes["PROXMOXVE_INSTANCE_ID"],
        "15a9919cb91024fbd1d70fa07f0efa749cbba03b".to_string()
    );

    assert_eq!(attributes["PROXMOXVE_IPV4"], "192.168.1.1".to_string());

    assert_eq!(
        attributes["PROXMOXVE_IPV6"],
        "2001:db8:85a3::8a2e:370:0".to_string()
    );
}

#[test]
fn test_hostname() {
    let config = ProxmoxVECloudConfig::try_new(Path::new("tests/fixtures/proxmoxve/dhcp"))
        .expect("cannot parse config");

    assert_eq!(
        config.hostname().expect("cannot get hostname"),
        Some("dummy".to_string())
    );
}

#[test]
fn test_ssh_keys() {
    let test_ssh_key = PublicKey::from_str("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDd1hElre4j44sbmULXyO5j6dRnkRFCMjEGtRSy2SuvFD8WyB5uectcEMvz7ORhQIVbPlz94wFjpSX5wl/gmSKL/7GOyerJo0Y2cvyjJJahuDn+JnIL0tT0HS1pJ5iJqQpxXeOAzMK5Heum+uGw9BzbiUHnRzjJr8Ltx4CAGMfubevD4SX32Q8BTQiaU4ZnGtdHo16pWwRsq1f6/UtL4gDCni9vm8QmmGDRloi/pBn1csjKw+volFyu/kSEmGLWow6NuT6TrhGAbMKas5HfYq0Mn3LGPZL7XjqJQ6CO0TzkG/BNplZT2tiwHtsvXsbePTp4ZUi4dkCMz2xR4eikaI1V dummy@dummy.local").unwrap();
    let config = ProxmoxVECloudConfig::try_new(Path::new("tests/fixtures/proxmoxve/dhcp"))
        .expect("cannot parse config");

    assert_eq!(
        config.ssh_keys().expect("cannot get ssh keys"),
        vec![test_ssh_key]
    );
}

#[test]
fn test_network_dhcp() {
    let config = ProxmoxVECloudConfig::try_new(Path::new("tests/fixtures/proxmoxve/dhcp"))
        .expect("cannot parse config");

    assert_eq!(
        config.networks().expect("cannot get networks"),
        vec![network::Interface {
            name: Some("eth0".to_owned()),
            mac_address: Some(MacAddr::from_str("01:23:45:67:89:00").unwrap()),
            path: None,
            priority: 20,
            dhcp: Some(DhcpSetting::V4),
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
    let config = ProxmoxVECloudConfig::try_new(Path::new("tests/fixtures/proxmoxve/static"))
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
                dhcp: None,
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
                nameservers: vec![
                    IpAddr::from_str("1.1.1.1").unwrap(),
                    IpAddr::from_str("8.8.8.8").unwrap()
                ],
                ip_addresses: vec![
                    IpNetwork::from_str("192.168.42.1/24").unwrap(),
                    IpNetwork::from_str("2001:0db8:85a3:0000:0000:8a2e:4242:0/24").unwrap(),
                ],
                dhcp: None,
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

#[test]
fn test_invalid_user_data() {
    let config =
        ProxmoxVECloudConfig::try_new(Path::new("tests/fixtures/proxmoxve/invalid-user-data"))
            .expect("cannot parse config");

    assert!(config.hostname().unwrap().is_none());
    assert_eq!(config.ssh_keys().unwrap(), vec![]);
}

#[test]
fn test_network_kargs() {
    let config = ProxmoxVECloudConfig::try_new(Path::new("tests/fixtures/proxmoxve/static"))
        .expect("cannot parse config");

    let kargs = config.rd_network_kargs().expect("cannot get network kargs");
    assert!(kargs.is_some());
    let kargs = kargs.unwrap();

    // Check static IP configuration with gateway
    assert!(kargs.contains("ip=192.168.1.1::192.168.1.254:255.255.255.0"));
    assert!(kargs.contains("ip=2001:db8:85a3::8a2e:370:0::2001:db8:85a3::8a2e:370:9999:24"));

    // Check nameservers
    assert!(kargs.contains("nameserver=1.1.1.1,8.8.8.8"));
}

#[test]
fn test_network_kargs_dhcp() {
    let config = ProxmoxVECloudConfig::try_new(Path::new("tests/fixtures/proxmoxve/dhcp"))
        .expect("cannot parse config");

    let kargs = config.rd_network_kargs().expect("cannot get network kargs");
    assert!(kargs.is_some());
    let kargs = kargs.unwrap();

    // Check DHCP configuration
    assert!(kargs.contains("ip=dhcp"));

    // Check nameservers
    assert!(kargs.contains("nameserver=1.1.1.1,8.8.8.8"));
}

#[test]
fn test_network_kargs_no_gateway() {
    let config =
        ProxmoxVECloudConfig::try_new(Path::new("tests/fixtures/proxmoxve/static-no-gateway"))
            .expect("cannot parse config");

    let kargs = config.rd_network_kargs().expect("cannot get network kargs");
    assert!(kargs.is_some());
    let kargs = kargs.unwrap();

    // Check static IP configuration without gateway
    assert!(kargs.contains("ip=192.168.1.1:::255.255.255.0"));

    // Check nameservers
    assert!(kargs.contains("nameserver=1.1.1.1,8.8.8.8"));
}

#[test]
fn test_netplan_config_static() {
    let config = ProxmoxVECloudConfig::try_new(Path::new("tests/fixtures/proxmoxve/static"))
        .expect("cannot parse config");

    let netplan = config.netplan_config().expect("cannot get netplan config");
    assert!(netplan.is_some());
    let netplan = netplan.unwrap();

    // Parse the YAML to verify its structure
    let parsed: serde_yaml::Value = serde_yaml::from_str(&netplan).expect("invalid YAML");

    // Check network configuration
    let network = &parsed["network"];
    assert!(network.is_mapping());

    // Check ethernet interfaces
    let ethernets = &network["ethernets"];
    assert!(ethernets.is_mapping());

    // Check eth0 configuration
    let eth0 = &ethernets["eth0"];
    assert!(eth0.is_mapping());

    // Verify static addresses
    let addresses = eth0["addresses"].as_sequence().unwrap();
    assert!(addresses.contains(&serde_yaml::Value::String("192.168.1.1/24".into())));
    assert!(addresses.contains(&serde_yaml::Value::String(
        "2001:db8:85a3::8a2e:370:0/24".into()
    )));

    // Verify nameservers
    let nameservers = &eth0["nameservers"]["addresses"];
    assert!(nameservers
        .as_sequence()
        .unwrap()
        .contains(&serde_yaml::Value::String("1.1.1.1".into())));
    assert!(nameservers
        .as_sequence()
        .unwrap()
        .contains(&serde_yaml::Value::String("8.8.8.8".into())));


    let eth1 = &ethernets["eth1"];
    assert!(eth1.is_mapping());

    let nameservers = &eth1["nameservers"]["addresses"];
    assert!(nameservers
        .as_sequence()
        .unwrap()
        .contains(&serde_yaml::Value::String("1.1.1.1".into())));
    assert!(nameservers
        .as_sequence()
        .unwrap()
        .contains(&serde_yaml::Value::String("8.8.8.8".into())));
}

#[test]
fn test_netplan_config_dhcp() {
    let config = ProxmoxVECloudConfig::try_new(Path::new("tests/fixtures/proxmoxve/dhcp"))
        .expect("cannot parse config");

    let netplan = config.netplan_config().expect("cannot get netplan config");
    assert!(netplan.is_some());
    let netplan = netplan.unwrap();

    // Parse the YAML to verify its structure
    let parsed: serde_yaml::Value = serde_yaml::from_str(&netplan).expect("invalid YAML");

    // Check network configuration
    let network = &parsed["network"];
    let ethernets = &network["ethernets"];
    let eth0 = &ethernets["eth0"];

    // Verify DHCP configuration
    assert_eq!(eth0["dhcp4"], serde_yaml::Value::Bool(true));

    // Verify nameservers
    let nameservers = &eth0["nameservers"]["addresses"];
    assert!(nameservers
        .as_sequence()
        .unwrap()
        .contains(&serde_yaml::Value::String("1.1.1.1".into())));
    assert!(nameservers
        .as_sequence()
        .unwrap()
        .contains(&serde_yaml::Value::String("8.8.8.8".into())));
}
