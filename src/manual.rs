use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct ProxyConfig {
    listener: String,
    tls_certificate: Option<String>,
    tls_certificate_key: Option<String>,
    servers: HashMap<String, ProxyHostConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    prometheus_addr: Option<String>,
    proxy: Option<Vec<ProxyConfig>>,
    load_balancer: Option<Vec<LoadBalancerConfig>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LBHostConfig {
    pub load_balancer_tls: bool,
    pub load_balancer_headers: Option<Vec<(String, String)>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyHostConfig {
    pub proxy_addr: String,
    pub proxy_tls: bool,
    pub proxy_headers: Option<Vec<(String, String)>>,
    pub proxy_uds: Option<bool>,
}

impl Config {
    pub fn from_str(input: &str) -> Result<Self, String> {
        let mut config = Config {
            prometheus_addr: None,
            proxy: Some(vec![]),
            load_balancer: Some(vec![]),
        };

        let mut current_section = String::new();
        let mut current_proxy: Option<ProxyConfig> = None;
        let mut current_lb: Option<LoadBalancerConfig> = None;

        for line in input.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') {
                if let Some(proxy) = current_proxy.take() {
                    config.proxy.as_mut().unwrap().push(proxy);
                }
                if let Some(lb) = current_lb.take() {
                    config.load_balancer.as_mut().unwrap().push(lb);
                }

                current_section = line.trim_matches(['[', ']', '"'].as_ref()).to_string();
                if current_section == "proxy" {
                    current_proxy = Some(ProxyConfig {
                        listener: String::new(),
                        tls_certificate: None,
                        tls_certificate_key: None,
                        servers: HashMap::new(),
                    });
                } else if current_section == "load_balancer" {
                    current_lb = Some(LoadBalancerConfig {
                        listener: String::new(),
                        upstreams: vec![],
                        health_check: None,
                        health_check_frequency: None,
                        parallel_health_check: None,
                        tls_certificate: None,
                        tls_certificate_key: None,
                        servers: HashMap::new(),
                    });
                }

                continue;
            }

            if current_section.is_empty() {
                if let Some((key, value)) = parse_key_value(line) {
                    if key == "prometheus_addr" {
                        config.prometheus_addr = Some(value);
                    }
                }
            } else if current_section == "proxy" {
                if let Some(proxy) = current_proxy.as_mut() {
                    if let Some((key, value)) = parse_key_value(line) {
                        match key.as_str() {
                            "listener" => proxy.listener = value,
                            "tls_certificate" => proxy.tls_certificate = Some(value),
                            "tls_certificate_key" => proxy.tls_certificate_key = Some(value),
                            _ => (),
                        }
                    } else if let Some((domain, server_config)) = parse_proxy_server_block(line) {
                        proxy.servers.insert(domain, server_config);
                    }
                }
            } else if current_section == "load_balancer" {
                if let Some(lb) = current_lb.as_mut() {
                    if let Some((key, value)) = parse_key_value(line) {
                        match key.as_str() {
                            "listener" => lb.listener = value,
                            "health_check" => lb.health_check = Some(value == "true"),
                            "health_check_frequency" => {
                                lb.health_check_frequency = value.parse().ok()
                            }
                            "parallel_health_check" => {
                                lb.parallel_health_check = Some(value == "true")
                            }
                            "tls_certificate" => lb.tls_certificate = Some(value),
                            "tls_certificate_key" => lb.tls_certificate_key = Some(value),
                            _ => (),
                        }
                    } else if let Some((domain, server_config)) = parse_lb_server_block(line) {
                        lb.servers.insert(domain, server_config);
                    }
                }
            }
        }

        if let Some(proxy) = current_proxy {
            config.proxy.as_mut().unwrap().push(proxy);
        }
        if let Some(lb) = current_lb {
            config.load_balancer.as_mut().unwrap().push(lb);
        }

        Ok(config)
    }
}

fn parse_key_value(line: &str) -> Option<(String, String)> {
    if let Some((key, value)) = line.split_once('=') {
        Some((key.trim().to_string(), value.trim().to_string()))
    } else {
        None
    }
}

fn parse_proxy_server_block(line: &str) -> Option<(String, ProxyHostConfig)> {
    if line.starts_with("[[\"") && line.ends_with("\"]]") {
        let domain = line.trim_matches(['[', ']', '\"'].as_ref()).to_string();
        Some((domain, ProxyHostConfig {
            proxy_addr: String::new(),
            proxy_tls: false,
            proxy_headers: None,
            proxy_uds: None,
        }))
    } else {
        None
    }
}

fn parse_lb_server_block(line: &str) -> Option<(String, LBHostConfig)> {
    if line.starts_with("[[\"") && line.ends_with("\"]]") {
        let domain = line.trim_matches(['[', ']', '\"'].as_ref()).to_string();
        Some((domain, LBHostConfig {
            load_balancer_tls: false,
            load_balancer_headers: None,
        }))
    } else {
        None
    }
}

pub fn main3() {
    let input = r#"prometheus_addr = "0.0.0.0:9090"

[proxy]
listener = "0.0.0.0:8080"

default_configs = "some_default_value"

[["example.com"]]
proxy_addr = "/tmp/example.sock"
proxy_tls = false
proxy_headers = [["X-Example-Header", "value"]]
proxy_uds = true

[["another.com"]]
proxy_addr = "127.0.0.1:8001"
proxy_tls = true
proxy_headers = [["X-Another-Header", "another-value"]]
proxy_uds = false

[load_balancer]
listener = "0.0.0.0:7070"
upstreams = ["127.0.0.1:7000", "127.0.0.1:7001"]
health_check = true
health_check_frequency = 30
parallel_health_check = true

default_configs = "lb_default_value"

[["example.com"]]
load_balancer_tls = false
load_balancer_headers = [["X-LB-Example", "value"]]

[["another.com"]]
load_balancer_tls = true
load_balancer_headers = [["X-LB-Another", "another-value"]]

[["new-site.com"]]
load_balancer_tls = true
load_balancer_headers = [["X-New-Site", "new-value"]]

[proxy]
listener = "0.0.0.0:9090"
tls_certificate = "cert.pem"
tls_certificate_key = "key.pem"

[["newproxy.com"]]
proxy_addr = "127.0.0.1:9001"
proxy_tls = false
proxy_headers = [["X-New-Proxy-Header", "new-proxy-value"]]
proxy_uds = false

[["proxyexample.com"]]
proxy_addr = "/tmp/proxy.sock"
proxy_tls = true
proxy_headers = [["X-Proxy-Header", "proxy-value"]]
proxy_uds = true

[["extra-site.com"]]
proxy_addr = "192.168.1.1:8000"
proxy_tls = true
proxy_headers = [["X-Extra-Header", "extra-value"]]
proxy_uds = false

"#;
    match Config::from_str(input) {
        Ok(config) => println!("{:#?}", config),
        Err(err) => eprintln!("Failed to parse config: {}", err),
    }
}
