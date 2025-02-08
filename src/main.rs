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

fn parse_proxy_config(input: &str) -> Result<Vec<ProxyConfig>, String> {
    let mut proxy_configs = Vec::new();
    let mut current_proxy_config: Option<ProxyConfig> = None;
    let mut current_host_config: Option<ProxyHostConfig> = None;
    let mut current_route_config: Option<ProxyPathBaseHostConfig> = None;
    let mut current_domain: Option<String> = None;

    for line in input.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        } else if line == "[proxy]" {
            if let Some(config) = current_proxy_config.take() {
                proxy_configs.push(config);
            }
            current_proxy_config = Some(ProxyConfig::default());
            current_host_config = None;
            current_domain = None;
        } else if line.starts_with("[[") && line.ends_with("]]") && !line.starts_with("[[[") {
            if let Some(domain) = current_domain.take() {
                if let Some(host_config) = current_host_config.take() {
                    if let Some(proxy_config) = current_proxy_config.as_mut() {
                        proxy_config.servers.insert(domain, host_config);
                    }
                }
            }
            current_domain = Some(line.trim_matches('[').trim_matches(']').to_string());
            current_host_config = Some(ProxyHostConfig::default());
            current_route_config = None;
        } else if line.starts_with("[[[") && line.ends_with("]]]") {
            let route = line.trim_matches('[').trim_matches(']').to_string();
            if let Some(host_config) = current_host_config.as_mut() {
                if let Some(route_config) = current_route_config.take() {
                    // Add the completed route config to the host's routes
                    host_config.routes.insert(route.clone(), route_config);
                }
                current_route_config = Some(ProxyPathBaseHostConfig::default());
            } else {
                return Err("Found route section without a domain section".into());
            }
        } else if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            if let Some(route_config) = current_route_config.as_mut() {
                match key {
                    "proxy_addr" => route_config.proxy_addr = Some(value.to_string()),
                    "proxy_tls" => route_config.proxy_tls = value.parse().unwrap_or(false),
                    "proxy_headers" => {
                        let headers: Vec<(String, String)> = parse_headers(value)?;
                        route_config.proxy_headers = Some(headers);
                    }
                    "proxy_uds" => route_config.proxy_uds = value.parse().unwrap_or(false),
                    _ => {}
                }
            } else if let Some(host_config) = current_host_config.as_mut() {
                match key {
                    "proxy_addr" => host_config.proxy_addr = value.to_string(),
                    "proxy_tls" => host_config.proxy_tls = value.parse().unwrap_or(false),
                    "proxy_headers" => {
                        let headers: Vec<(String, String)> = parse_headers(value)?;
                        host_config.proxy_headers = Some(headers);
                    }
                    "proxy_uds" => host_config.proxy_uds = value.parse().unwrap_or(false),
                    _ => {}
                }
            } else if let Some(proxy_config) = current_proxy_config.as_mut() {
                match key {
                    "listener" => proxy_config.listener = value.to_string(),
                    "tls_certificate" => proxy_config.tls_certificate = Some(value.to_string()),
                    "tls_certificate_key" => {
                        proxy_config.tls_certificate_key = Some(value.to_string())
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(domain) = current_domain.take() {
        if let Some(mut host_config) = current_host_config.take() {
            if let Some(proxy_config) = current_proxy_config.as_mut() {
                if let Some(route_config) = current_route_config.take() {
                    // Inherit proxy_addr from domain if missing
                    let route_addr = route_config
                        .proxy_addr
                        .clone()
                        .unwrap_or_else(|| host_config.proxy_addr.clone());
                    let mut completed_route = route_config.clone();
                    completed_route.proxy_addr = Some(route_addr);
                    host_config.routes.insert(domain.clone(), completed_route);
                }
                proxy_config.servers.insert(domain, host_config);
            }
        }
    }

    if let Some(config) = current_proxy_config {
        proxy_configs.push(config);
    }

    Ok(proxy_configs)
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

fn main() {
    let input = r#"[proxy]
listener = "127.0.0.1:8080"
tls_certificate = "path/to/cert1"
tls_certificate_key = "path/to/key1"

[["domain1.com"]]
proxy_addr = "/tmp/proxy.sock"
proxy_tls = true
proxy_headers = [["Header1": "Value1"]]
proxy_uds = true

[[["/route1"]]]
proxy_addr = "192.168.1.2"
proxy_tls = false
proxy_headers = [["Header2": "Value2"]]
"#;

    match parse_proxy_config(input) {
        Ok(configs) => println!("Parsed configs: {:#?}", configs),
        Err(err) => eprintln!("Error: {}", err),
    }
}
