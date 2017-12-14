//! oracle-oci metadata fetcher

use retry;
use metadata;
use errors::*;

use openssh_keys::PublicKey;

#[derive(Debug, Deserialize, Clone)]
struct InstanceData {
    #[serde(rename = "availabilityDomain")]
    availability_domain: String,
    #[serde(rename = "compartmentId")]
    compartment_id: String,
    #[serde(rename = "displayName")]
    display_name: String,
    id: String,
    image: String,
    region: String,
    shape: String,
    #[serde(rename = "timeCreated")]
    time_created: u64,
    metadata: Metadata,
}

#[derive(Debug, Deserialize, Clone)]
struct Metadata {
    #[serde(default)]
    ssh_authorized_keys: String,
}

pub fn fetch_metadata() -> Result<metadata::Metadata> {
    let client = retry::Client::new()
        .chain_err(|| "oracle-oci: failed to create http client")?;

    let data: InstanceData = client.get(retry::Json, "http://169.254.169.254/opc/v1/instance/".into()).send()
        .chain_err(|| "oracle-oci: failed to get instance metadata from metadata service")?
        .ok_or_else(|| "oracle-oci: failed to get instance metadata from metadata service: no response")?;

    let ssh_keys = PublicKey::read_keys(data.metadata.ssh_authorized_keys.as_bytes())?;

    Ok(metadata::Metadata::builder()
        .add_attribute("ORACLE_OCI_DISPLAY_NAME".into(), data.display_name)
        .add_attribute("ORACLE_OCI_INSTANCE_ID".into(), data.id)
        .add_attribute("ORACLE_OCI_REGION".into(), data.region)
        .add_publickeys(ssh_keys)
        .build())
}
