//! openstack metadata fetcher

use std::collections::HashMap;

use anyhow::{anyhow, bail, Context, Result};
use openssh_keys::PublicKey;
use serde::Deserialize;

use crate::providers::MetadataProvider;
use crate::retry;

const EC2_URL: &str = "http://169.254.169.254/latest/meta-data";
const NOVA_URL: &str = "http://169.254.169.254/openstack/2012-08-10/meta_data.json";

/// Partial object for openstack `meta_data.json`
#[derive(Debug, Deserialize, Default)]
pub struct MetadataOpenstackJSON {
    /// Instance ID.
    pub uuid: Option<String>,
}

#[derive(Clone, Debug)]
pub struct OpenstackProviderNetwork {
    pub(crate) client: retry::Client,
}

impl OpenstackProviderNetwork {
    pub fn try_new() -> Result<OpenstackProviderNetwork> {
        let client = retry::Client::try_new()?.return_on_404(true);
        Ok(OpenstackProviderNetwork { client })
    }

    fn ec2_endpoint_for(key: &str) -> String {
        format!("{EC2_URL}/{key}")
    }

    /// The metadata is stored as JSON in openstack/<version>/meta_data.json file
    fn fetch_metadata_openstack(&self) -> Result<MetadataOpenstackJSON> {
        let metadata: Option<String> =
            self.client.get(retry::Raw, String::from(NOVA_URL)).send()?;

        if let Some(metadata) = metadata {
            let metadata: MetadataOpenstackJSON =
                serde_json::from_str(&metadata).context("failed to parse JSON metadata")?;
            Ok(metadata)
        } else {
            Ok(MetadataOpenstackJSON::default())
        }
    }

    fn fetch_keys(&self) -> Result<Vec<String>> {
        let keys_list: Option<String> = self
            .client
            .get(
                retry::Raw,
                OpenstackProviderNetwork::ec2_endpoint_for("public-keys"),
            )
            .send()?;
        let mut keys = Vec::new();
        if let Some(keys_list) = keys_list {
            for l in keys_list.lines() {
                let tokens: Vec<&str> = l.split('=').collect();
                if tokens.len() != 2 {
                    bail!("error parsing keyID");
                }
                let key: String = self
                    .client
                    .get(
                        retry::Raw,
                        OpenstackProviderNetwork::ec2_endpoint_for(&format!(
                            "public-keys/{}/openssh-key",
                            tokens[0]
                        )),
                    )
                    .send()?
                    .ok_or_else(|| anyhow!("missing ssh key"))?;
                keys.push(key);
            }
        }
        Ok(keys)
    }
}

impl MetadataProvider for OpenstackProviderNetwork {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        let mut out = HashMap::with_capacity(6);

        let openstack_metadata = self.fetch_metadata_openstack()?;

        let add_value = |map: &mut HashMap<_, _>, key: &str, name| -> Result<()> {
            let value = self
                .client
                .get(retry::Raw, OpenstackProviderNetwork::ec2_endpoint_for(name))
                .send()?;
            if let Some(value) = value {
                map.insert(key.to_string(), value);
            }
            Ok(())
        };

        add_value(&mut out, "OPENSTACK_HOSTNAME", "hostname")?;
        add_value(&mut out, "OPENSTACK_INSTANCE_ID", "instance-id")?;
        if let Some(instance_uuid) = openstack_metadata.uuid {
            out.insert("OPENSTACK_INSTANCE_UUID".to_string(), instance_uuid);
        };
        add_value(&mut out, "OPENSTACK_INSTANCE_TYPE", "instance-type")?;
        add_value(&mut out, "OPENSTACK_IPV4_LOCAL", "local-ipv4")?;
        add_value(&mut out, "OPENSTACK_IPV4_PUBLIC", "public-ipv4")?;

        Ok(out)
    }

    fn hostname(&self) -> Result<Option<String>> {
        self.client
            .get(
                retry::Raw,
                OpenstackProviderNetwork::ec2_endpoint_for("hostname"),
            )
            .send()
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        let mut out = Vec::new();

        for key in &self.fetch_keys()? {
            let key = PublicKey::parse(key)?;
            out.push(key);
        }

        Ok(out)
    }
}
