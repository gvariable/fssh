use ssh2_config::{ParseRule, SshConfig};
use whoami::username;

#[derive(Clone, Debug)]
pub struct SshConfigItem {
    pub host: String,
    pub user: String,
    pub hostname: String,
}

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
