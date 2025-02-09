use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct ProxyConfig {
    pub listener: String,
    pub tls_certificate: Option<String>,
    pub tls_certificate_key: Option<String>,
    pub servers: HashMap<String, ProxyHostConfig>,
}

#[derive(Default, Debug)]
pub struct ProxyHostConfig {
    pub proxy_addr: String,
    pub proxy_tls: bool,
    pub proxy_headers: Option<Vec<(String, String)>>,
    pub proxy_uds: bool,
    pub routes: HashMap<String, ProxyPathBaseHostConfig>,
}

#[derive(Default, Debug)]
pub struct ProxyPathBaseHostConfig {
    pub proxy_addr: Option<String>,
    pub proxy_tls: bool,
    pub proxy_headers: Option<Vec<(String, String)>>,
    pub proxy_uds: bool,
}

#[derive(Default, Debug)]
pub struct LoadBalancerConfig {
    pub listener: String,
    pub upstreams: Vec<String>,
    pub health_check: Option<bool>,
    pub health_check_frequency: Option<u64>,
    pub parallel_health_check: Option<u64>,
    pub tls_certificate: Option<String>,
    pub tls_certificate_key: Option<String>,
    pub servers: HashMap<String, LBHostConfig>,
}

#[derive(Default, Debug)]
pub struct LBHostConfig {
    pub load_balancer_tls: bool,
    pub load_balancer_headers: Option<Vec<(String, String)>>,
}

#[derive(Debug)]
pub enum ConfigType {
    Proxy,
    LoadBalancer,
}

pub fn parse_combined_config(
    input: &str,
) -> Result<(Vec<ProxyConfig>, Vec<LoadBalancerConfig>), String> {
    let mut proxy_configs = Vec::new();
    let mut lb_configs = Vec::new();
    let mut current_config_type: Option<ConfigType> = None;

    let mut current_proxy_config: Option<ProxyConfig> = None;
    let mut current_lb_config: Option<LoadBalancerConfig> = None;

    let mut current_host_config: Option<ProxyHostConfig> = None;
    let mut current_lb_host_config: Option<LBHostConfig> = None;

    let mut current_domain: Option<String> = None;
    let mut current_route_config: Option<ProxyPathBaseHostConfig> = None;

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line == "[proxy]" {
            if let Some(config) = current_proxy_config.take() {
                proxy_configs.push(config);
            }
            current_config_type = Some(ConfigType::Proxy);
            current_proxy_config = Some(ProxyConfig::default());
            current_host_config = None;
            current_domain = None;
        } else if line == "[load_balancer]" {
            if let Some(config) = current_lb_config.take() {
                lb_configs.push(config);
            }
            current_config_type = Some(ConfigType::LoadBalancer);
            current_lb_config = Some(LoadBalancerConfig::default());
            current_lb_host_config = None;
            current_domain = None;
        } else if line.starts_with("[[") && line.ends_with("]]") {
            if let Some(domain) = current_domain.take() {
                match current_config_type {
                    Some(ConfigType::Proxy) => {
                        if let Some(host_config) = current_host_config.take() {
                            if let Some(proxy_config) = current_proxy_config.as_mut() {
                                proxy_config.servers.insert(domain, host_config);
                            }
                        }
                    }
                    Some(ConfigType::LoadBalancer) => {
                        if let Some(lb_host_config) = current_lb_host_config.take() {
                            if let Some(lb_config) = current_lb_config.as_mut() {
                                lb_config.servers.insert(domain, lb_host_config);
                            }
                        }
                    }
                    _ => {}
                }
            }

            current_domain = Some(
                line.trim_matches('[')
                    .trim_matches(']')
                    .to_string()
                    .trim()
                    .to_string(),
            );

            match current_config_type {
                Some(ConfigType::Proxy) => {
                    current_host_config = Some(ProxyHostConfig::default());
                }
                Some(ConfigType::LoadBalancer) => {
                    current_lb_host_config = Some(LBHostConfig::default());
                }
                _ => {}
            }
        } else if line.starts_with("[[[") && line.ends_with("]]]") {
            if let Some(host_config) = current_host_config.as_mut() {
                if let Some(route_config) = current_route_config.take() {
                    host_config.routes.insert(
                        line.trim_matches('[').trim_matches(']').to_string(),
                        route_config,
                    );
                }
            }
            current_route_config = Some(ProxyPathBaseHostConfig::default());
        } else if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            match current_config_type {
                Some(ConfigType::Proxy) => {
                    if let Some(route_config) = current_route_config.as_mut() {
                        match key {
                            "proxy_addr" => route_config.proxy_addr = Some(value.to_string()),
                            "proxy_tls" => route_config.proxy_tls = value.parse().unwrap_or(false),
                            "proxy_headers" => {
                                let headers = parse_headers(value)?;
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
                                let headers = parse_headers(value)?;
                                host_config.proxy_headers = Some(headers);
                            }
                            "proxy_uds" => host_config.proxy_uds = value.parse().unwrap_or(false),
                            _ => {}
                        }
                    } else if let Some(proxy_config) = current_proxy_config.as_mut() {
                        match key {
                            "listener" => proxy_config.listener = value.to_string(),
                            "tls_certificate" => {
                                proxy_config.tls_certificate = Some(value.to_string())
                            }
                            "tls_certificate_key" => {
                                proxy_config.tls_certificate_key = Some(value.to_string())
                            }
                            _ => {}
                        }
                    }
                }
                Some(ConfigType::LoadBalancer) => {
                    if let Some(lb_host_config) = current_lb_host_config.as_mut() {
                        match key {
                            "load_balancer_tls" => {
                                lb_host_config.load_balancer_tls = value.parse().unwrap_or(false)
                            }
                            "load_balancer_headers" => {
                                let headers = parse_headers(value)?;
                                lb_host_config.load_balancer_headers = Some(headers);
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
                            "health_check" => {
                                lb_config.health_check = Some(value.parse().unwrap_or(false))
                            }
                            "health_check_frequency" => {
                                lb_config.health_check_frequency = value.parse().ok();
                            }
                            "parallel_health_check" => {
                                lb_config.parallel_health_check = value.parse().ok();
                            }
                            "tls_certificate" => {
                                lb_config.tls_certificate = Some(value.to_string())
                            }
                            "tls_certificate_key" => {
                                lb_config.tls_certificate_key = Some(value.to_string())
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(config) = current_proxy_config {
        proxy_configs.push(config);
    }

    if let Some(config) = current_lb_config {
        lb_configs.push(config);
    }

    Ok((proxy_configs, lb_configs))
}

fn parse_headers(input: &str) -> Result<Vec<(String, String)>, String> {
    let mut headers = Vec::new();
    let entries = input
        .trim_matches('[')
        .trim_matches(']')
        .split(',')
        .map(|s| s.trim());

    for entry in entries {
        if let Some((key, value)) = entry.split_once(':') {
            headers.push((key.trim().to_string(), value.trim().to_string()));
        } else {
            return Err(format!("Invalid header format: {}", entry));
        }
    }

    Ok(headers)
}
