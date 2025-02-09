use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
enum Tokens {
    ProxySectionStart,
    ProxySectionEnd,
}

#[derive(Debug)]
enum Error {
    ConfigNotFound,
    ListenerNotFound,
    ServerNotFound,
    ServerAddressNotFound,
    RouteNotFound,
    RouteAddressNotFound,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct ProxyConfig {
    listener: String,
    tls_certificate: Option<String>,
    tls_certificate_key: Option<String>,
    servers: HashMap<String, ProxyHostConfig>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ProxyHostConfig {
    pub proxy_addr: String,
    pub proxy_tls: bool,
    pub proxy_headers: Option<Vec<(String, String)>>,
    pub proxy_uds: bool,
    pub routes: HashMap<String, ProxyPathBaseHostConfig>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ProxyPathBaseHostConfig {
    pub proxy_addr: Option<String>,
    pub proxy_tls: bool,
    pub proxy_headers: Option<Vec<(String, String)>>,
    pub proxy_uds: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct LoadBalancerConfig {
    listener: String,
    upstreams: Vec<String>,
    health_check: Option<bool>,
    health_check_frequency: Option<u64>,
    parallel_health_check: Option<bool>,
    tls_certificate: Option<String>,
    tls_certificate_key: Option<String>,
    servers: HashMap<String, LBHostConfig>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct LBHostConfig {
    pub load_balancer_tls: bool,
    pub load_balancer_headers: Option<Vec<(String, String)>>,
}

fn parse_load_balancer_config(input: &str) -> Result<Vec<LoadBalancerConfig>, String> {
    let mut lb_configs = Vec::new();
    let mut current_lb_config: Option<LoadBalancerConfig> = None;
    let mut current_host_config: Option<LBHostConfig> = None;
    let mut current_domain: Option<String> = None;

    for line in input.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        } else if line == "[load_balancer]" {
            if let Some(config) = current_lb_config.take() {
                lb_configs.push(config);
            }
            current_lb_config = Some(LoadBalancerConfig::default());
            current_host_config = None;
            current_domain = None;
        } else if line.starts_with("[[") && line.ends_with("]]") {
            if let Some(domain) = current_domain.take() {
                if let Some(host_config) = current_host_config.take() {
                    if let Some(lb_config) = current_lb_config.as_mut() {
                        lb_config.servers.insert(domain, host_config);
                    }
                }
            }
            current_domain = Some(
                line.trim_matches('[')
                    .trim_matches(']')
                    .to_string()
                    .trim()
                    .to_string(),
            );
            current_host_config = Some(LBHostConfig::default());
        } else if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            if let Some(host_config) = current_host_config.as_mut() {
                match key {
                    "load_balancer_tls" => {
                        host_config.load_balancer_tls = value.parse().unwrap_or(false)
                    }
                    "load_balancer_headers" => {
                        let headers: Vec<(String, String)> = parse_headers(value)?;
                        host_config.load_balancer_headers = Some(headers);
                    }
                    _ => {}
                }
            } else if let Some(lb_config) = current_lb_config.as_mut() {
                match key {
                    "listener" => lb_config.listener = value.to_string(),
                    "upstreams" => {
                        lb_config.upstreams =
                            value.split(',').map(|s| s.trim().to_string()).collect();
                    }
                    "health_check" => lb_config.health_check = Some(value.parse().unwrap_or(false)),
                    "health_check_frequency" => {
                        lb_config.health_check_frequency = value.parse().ok();
                    }
                    "parallel_health_check" => {
                        lb_config.parallel_health_check = value.parse().ok();
                    }
                    "tls_certificate" => lb_config.tls_certificate = Some(value.to_string()),
                    "tls_certificate_key" => {
                        lb_config.tls_certificate_key = Some(value.to_string())
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(domain) = current_domain.take() {
        if let Some(host_config) = current_host_config.take() {
            if let Some(lb_config) = current_lb_config.as_mut() {
                lb_config.servers.insert(domain, host_config);
            }
        }
    }

    if let Some(config) = current_lb_config {
        lb_configs.push(config);
    }

    Ok(lb_configs)
}

fn parse_headers(input: &str) -> Result<Vec<(String, String)>, String> {
    let mut headers = Vec::new();
    let entries: Vec<&str> = input
        .trim_matches('[')
        .trim_matches(']')
        .split(',')
        .map(|s| s.trim())
        .collect();

    for entry in entries {
        if let Some((key, value)) = entry.split_once(':') {
            headers.push((key.trim().to_string(), value.trim().to_string()));
        } else {
            return Err(format!("Invalid header format: {}", entry));
        }
    }

    Ok(headers)
}

pub fn main6() {
    let input = r#"
[load_balancer]
upstreams = "127.0.0.1:8080,127.0.0.1:8081"
health_check = true
health_check_frequency = 30
parallel_health_check = 5
tls_certificate = "path/to/lb-cert.pem"
tls_certificate_key = "path/to/lb-key.pem"

"#;

    match parse_load_balancer_config(input) {
        Ok(configs) => println!("Parsed Load Balancer configs: {:#?}", configs),
        Err(err) => eprintln!("Error: {}", err),
    }
}
