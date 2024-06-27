use serde::{Deserialize, Serialize};
use ssh2_config::{ParseRule, SshConfig};
use whoami::username;

#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq)]
/// Key elements of an SSH configuration.
pub struct SshConfigItem {
    /// Alias name in the configuration.
    pub host: String,
    /// User name.
    pub user: String,
    /// IP or DNS.
    pub hostname: String,
}

/// Reads the default SSH configuration file and retrieves a list of [`SshConfigItem`].
pub fn retrive_ssh_configs() -> Result<Vec<SshConfigItem>, Box<dyn std::error::Error>> {
    let config = SshConfig::parse_default_file(ParseRule::STRICT)?;

    let mut datas = Vec::new();
    for host in config.get_hosts() {
        // if hostname is not set, we can't connect to it
        if let Some(hostname) = host.params.host_name.clone() {
            // if user is not set, we use the current user
            let user = host.params.user.clone().unwrap_or(username());

            for alias in host.pattern.iter() {
                datas.push(SshConfigItem {
                    host: alias.pattern.clone(),
                    user: user.clone(),
                    hostname: hostname.clone(),
                });
            }
        }
    }

    Result::Ok(datas)
}
