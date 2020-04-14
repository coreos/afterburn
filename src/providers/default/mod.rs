//! Default provider.
//!
//! This is a generic provider which is used as a fallback whenever
//! it is not possible to match to a relevant one. It is intended not
//! to perform any actions nor to have any side effects.

use std::collections::HashMap;

use error_chain::bail;
use openssh_keys::PublicKey;
use slog_scope::warn;

use crate::errors::*;
use crate::network;
use crate::providers::MetadataProvider;

/// Default provider.
#[derive(Clone, Debug)]
pub struct DefaultProvider {
    /// External provider name.
    name: String,
}

impl DefaultProvider {
    pub fn try_new(name: &str) -> Result<Self> {
        if name.is_empty() {
            bail!("empty provider name");
        };

        warn!(
            "unknown provider '{}', using default provider instead",
            name
        );
        let provider = Self {
            name: name.to_string(),
        };
        Ok(provider)
    }
}

impl MetadataProvider for DefaultProvider {
    fn attributes(&self) -> Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }

    fn hostname(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn ssh_keys(&self) -> Result<Vec<PublicKey>> {
        Ok(vec![])
    }

    fn networks(&self) -> Result<Vec<network::Interface>> {
        Ok(vec![])
    }

    fn virtual_network_devices(&self) -> Result<Vec<network::VirtualNetDev>> {
        Ok(vec![])
    }

    fn boot_checkin(&self) -> Result<()> {
        Ok(())
    }
}
