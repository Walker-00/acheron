use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProxyConfig {
    pub listener: String,
    pub tls_certificate: Option<String>,
    pub tls_certificate_key: Option<String>,
    pub servers: HashMap<String, ProxyHostConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct ProxyHostConfig {
    pub proxy_addr: String,
    pub proxy_tls: bool,
    pub proxy_headers: Option<Vec<(String, String)>>,
    pub proxy_uds: Option<bool>,
    pub routes: Option<HashMap<String, ProxyPathBaseHostConfig>>,
}

#[derive(Serialize, Deserialize)]
pub struct ProxyPathBaseHostConfig {
    pub proxy_addr: Option<String>,
    pub proxy_tls: Option<bool>,
    pub proxy_headers: Option<Vec<(String, String)>>,
    pub proxy_uds: Option<bool>,
}
