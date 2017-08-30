//! openstack metadata fetcher

use errors::*;
use metadata::Metadata;
use retry;

const URL: &'static str = "http://169.254.169.254/latest/meta-data";

fn url_for_key(key: &str) -> String {
    format!("{}/{}", URL, key)
}

pub fn fetch_metadata() -> Result<Metadata> {
    let client = retry::Client::new()
        .chain_err(|| "openstack: failed to create http client")?;

    let hostname: Option<String> = client.get(retry::Raw, url_for_key("hostname")).send()?;

    Ok(Metadata::builder()
        .add_attribute_if_exists("OPENSTACK_INSTANCE_ID".to_owned(), client.get(retry::Raw, url_for_key("instance-id")).send()?)
        .add_attribute_if_exists("OPENSTACK_IPV4_LOCAL".to_owned(), client.get(retry::Raw, url_for_key("local-ipv4")).send()?)
        .add_attribute_if_exists("OPENSTACK_IPV4_PUBLIC".to_owned(), client.get(retry::Raw, url_for_key("public-ipv4")).send()?)
        .add_attribute_if_exists("OPENSTACK_HOSTNAME".to_owned(), hostname.clone())
        .set_hostname_if_exists(hostname)
        .add_ssh_keys(fetch_keys(&client)?)?
        .build())
}

fn fetch_keys(client: &retry::Client) -> Result<Vec<String>> {
    let keys_list: Option<String> = client.get(retry::Raw, url_for_key("public-keys")).send()?;
    let mut keys = Vec::new();
    if let Some(keys_list) = keys_list {
        for l in keys_list.lines() {
            let tokens: Vec<&str> = l.split('=').collect();
            if tokens.len() != 2 {
                return Err("error parsing keyID".into());
            }
            let key: String = client.get(retry::Raw, url_for_key(&format!("public-keys/{}/openssh-key", tokens[0]))).send()?
                .ok_or("missing ssh key")?;
            keys.push(key);
        }
    }
    Ok(keys)
}
