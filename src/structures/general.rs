use serde::{Deserialize, Serialize};

use super::{load_balancer_structure::LoadBalancerConfig, proxy_structure::ProxyConfig};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub prometheus_addr: Option<String>,
    pub proxy: Option<Vec<ProxyConfig>>,
    pub load_balancer: Option<Vec<LoadBalancerConfig>>,
}
