//! Logic to interact with WireServer `goalstate` endpoint.

use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct GoalState {
    #[serde(rename = "Container")]
    pub container: Container,
    #[serde(rename = "Incarnation")]
    incarnation: String,
}

impl GoalState {
    /// Return the certificates endpoint (if any).
    pub(crate) fn certs_endpoint(&self) -> Option<String> {
        let role = match self.container.role_instance_list.role_instances.get(0) {
            Some(r) => r,
            None => return None,
        };

        role.configuration.certificates.clone()
    }

    /// Return this instance `ContainerId`.
    pub(crate) fn container_id(&self) -> &str {
        &self.container.container_id
    }

    /// Return this instance `InstanceId`.
    pub(crate) fn instance_id(&self) -> Result<&str> {
        Ok(&self
            .container
            .role_instance_list
            .role_instances
            .get(0)
            .ok_or_else(|| anyhow!("empty RoleInstanceList"))?
            .instance_id)
    }

    /// Return the current `Incarnation` count for this instance.
    pub(crate) fn incarnation(&self) -> &str {
        &self.incarnation
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(dead_code)]
pub(crate) struct Container {
    #[serde(rename = "ContainerId")]
    pub container_id: String,
    #[serde(rename = "RoleInstanceList")]
    pub role_instance_list: RoleInstanceList,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(dead_code)]
pub(crate) struct RoleInstanceList {
    #[serde(rename = "RoleInstance", default)]
    pub role_instances: Vec<RoleInstance>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub(crate) struct RoleInstance {
    #[serde(rename = "Configuration")]
    pub configuration: Configuration,
    #[serde(rename = "InstanceId")]
    pub instance_id: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub(crate) struct Configuration {
    #[serde(rename = "Certificates")]
    pub certificates: Option<String>,
    #[serde(rename = "SharedConfig", default)]
    pub shared_config: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub(crate) struct CertificatesFile {
    #[serde(rename = "Data", default)]
    pub data: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub(crate) struct SharedConfig {
    #[serde(rename = "Incarnation")]
    pub incarnation: Incarnation,
    #[serde(rename = "Instances")]
    pub instances: Instances,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub(crate) struct Incarnation {
    pub instance: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub(crate) struct Instances {
    #[serde(rename = "Instance", default)]
    pub instances: Vec<Instance>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub(crate) struct Instance {
    pub id: String,
    pub address: String,
    #[serde(rename = "InputEndpoints")]
    pub input_endpoints: InputEndpoints,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub(crate) struct InputEndpoints {
    #[serde(rename = "Endpoint", default)]
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub(crate) struct Endpoint {
    #[serde(rename = "loadBalancedPublicAddress", default)]
    pub load_balanced_public_address: String,
}
