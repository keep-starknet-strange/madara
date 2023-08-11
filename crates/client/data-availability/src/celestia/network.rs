use std::net::{IpAddr, Ipv4Addr};

pub struct BaseConfig {
    pub http_endpoint: Option<String>,
    pub ws_endpoint: Option<String>,
    pub auth_token: Option<String>,
}

impl Default for BaseConfig {
    fn default() -> Self {
        BaseConfig {
            http_endpoint: Some("http://".to_string() + &IpAddr::V4(Ipv4Addr::LOCALHOST).to_string() + ":26658"), /* Default to "127.0.0.1" */
            ws_endpoint: Some("ws://".to_string() + &IpAddr::V4(Ipv4Addr::LOCALHOST).to_string() + ":26658"), /* Default to "127.0.0.1" */
            auth_token: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Network {
    LOCAL,
}

impl Network {
    pub fn to_base_config(self) -> BaseConfig {
        match self {
            //  only support localhost for now, celestia has an emphasis on running the client locally
            // which limits the endpoint available to a collection of port choices. We'll assume the celestia
            // client is launched externally for now.
            Self::LOCAL => testnet(),
        }
    }
}

pub fn testnet() -> BaseConfig {
    BaseConfig { ..std::default::Default::default() }
}
