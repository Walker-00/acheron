// mod combined_parser;
// mod load_balancer;
//
// use combined_parser::parse_combined_config;
// use load_balancer::main6;
// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
//
// #[derive(Debug)]
// enum Tokens {
//     ProxySectionStart,
//     ProxySectionEnd,
// }
//
// #[derive(Debug)]
// enum Error {
//     ConfigNotFound,
//     ListenerNotFound,
//     ServerNotFound,
//     ServerAddressNotFound,
//     RouteNotFound,
//     RouteAddressNotFound,
// }
//
// #[derive(Serialize, Deserialize, Default, Clone, Debug)]
// struct ProxyConfig {
//     listener: String,
//     tls_certificate: Option<String>,
//     tls_certificate_key: Option<String>,
//     servers: HashMap<String, ProxyHostConfig>,
// }
//
// #[derive(Serialize, Deserialize, Default, Clone, Debug)]
// pub struct ProxyHostConfig {
//     pub proxy_addr: String,
//     pub proxy_tls: bool,
//     pub proxy_headers: Option<Vec<(String, String)>>,
//     pub proxy_uds: bool,
//     pub routes: HashMap<String, ProxyPathBaseHostConfig>,
// }
//
// #[derive(Serialize, Deserialize, Default, Clone, Debug)]
// pub struct ProxyPathBaseHostConfig {
//     pub proxy_addr: Option<String>,
//     pub proxy_tls: bool,
//     pub proxy_headers: Option<Vec<(String, String)>>,
//     pub proxy_uds: bool,
// }
//
// #[derive(Serialize, Deserialize, Default, Clone, Debug)]
// struct LoadBalancerConfig {
//     listener: String,
//     upstreams: Vec<String>,
//     health_check: Option<bool>,
//     health_check_frequency: Option<u64>,
//     parallel_health_check: Option<bool>,
//     tls_certificate: Option<String>,
//     tls_certificate_key: Option<String>,
//     servers: HashMap<String, LBHostConfig>,
// }
//
// #[derive(Serialize, Deserialize, Default, Clone, Debug)]
// pub struct LBHostConfig {
//     pub load_balancer_tls: bool,
//     pub load_balancer_headers: Option<Vec<(String, String)>>,
// }
//
// fn parse_proxy_config(input: &str) -> Result<Vec<ProxyConfig>, String> {
//     let mut proxy_configs = Vec::new();
//     let mut current_proxy_config: Option<ProxyConfig> = None;
//     let mut current_host_config: Option<ProxyHostConfig> = None;
//     let mut current_route_config: Option<ProxyPathBaseHostConfig> = None;
//     let mut current_domain: Option<String> = None;
//     let mut current_route_name: Option<String> = None;
//
//     for line in input.lines() {
//         let line = line.trim();
//
//         if line.is_empty() {
//             continue;
//         } else if line == "[proxy]" {
//             if let Some(config) = current_proxy_config.take() {
//                 proxy_configs.push(config);
//             }
//             current_proxy_config = Some(ProxyConfig::default());
//             current_host_config = None;
//             current_domain = None;
//         } else if line.starts_with("[[") && line.ends_with("]]") && !line.starts_with("[[[") {
//             if let Some(domain) = current_domain.take() {
//                 if let Some(host_config) = current_host_config.take() {
//                     if let Some(proxy_config) = current_proxy_config.as_mut() {
//                         proxy_config.servers.insert(domain, host_config);
//                     }
//                 }
//             }
//             current_domain = Some(
//                 line.trim_matches('[')
//                     .trim_matches(']')
//                     .to_string()
//                     .trim()
//                     .trim_matches('"')
//                     .trim_matches(' ')
//                     .to_string(),
//             );
//             current_host_config = Some(ProxyHostConfig::default());
//             current_route_config = None;
//         } else if line.starts_with("[[[") && line.ends_with("]]]") {
//             let route = line
//                 .trim_matches('[')
//                 .trim_matches(']')
//                 .to_string()
//                 .trim()
//                 .trim_matches('"')
//                 .trim_matches(' ')
//                 .to_string();
//             current_route_name = Some(route.clone());
//             if let Some(host_config) = current_host_config.as_mut() {
//                 if let Some(route_config) = current_route_config.take() {
//                     // Add the completed route config to the host's routes
//                     host_config.routes.insert(route.clone(), route_config);
//                 }
//                 current_route_config = Some(ProxyPathBaseHostConfig::default());
//             } else {
//                 return Err("Found route section without a domain section".into());
//             }
//         } else if let Some((key, value)) = line.split_once('=') {
//             let key = key.trim();
//             let value = value.trim().trim_matches('"');
//
//             if let Some(route_config) = current_route_config.as_mut() {
//                 match key {
//                     "proxy_addr" => route_config.proxy_addr = Some(value.to_string()),
//                     "proxy_tls" => route_config.proxy_tls = value.parse().unwrap_or(false),
//                     "proxy_headers" => {
//                         let headers: Vec<(String, String)> = parse_headers(value)?;
//                         route_config.proxy_headers = Some(headers);
//                     }
//                     "proxy_uds" => route_config.proxy_uds = value.parse().unwrap_or(false),
//                     _ => {}
//                 }
//             } else if let Some(host_config) = current_host_config.as_mut() {
//                 match key {
//                     "proxy_addr" => host_config.proxy_addr = value.to_string(),
//                     "proxy_tls" => host_config.proxy_tls = value.parse().unwrap_or(false),
//                     "proxy_headers" => {
//                         let headers: Vec<(String, String)> = parse_headers(value)?;
//                         host_config.proxy_headers = Some(headers);
//                     }
//                     "proxy_uds" => host_config.proxy_uds = value.parse().unwrap_or(false),
//                     _ => {}
//                 }
//             } else if let Some(proxy_config) = current_proxy_config.as_mut() {
//                 match key {
//                     "listener" => proxy_config.listener = value.to_string(),
//                     "tls_certificate" => proxy_config.tls_certificate = Some(value.to_string()),
//                     "tls_certificate_key" => {
//                         proxy_config.tls_certificate_key = Some(value.to_string())
//                     }
//                     _ => {}
//                 }
//             }
//         }
//     }
//
//     if let Some(domain) = current_domain.take() {
//         if let Some(mut host_config) = current_host_config.take() {
//             if let Some(proxy_config) = current_proxy_config.as_mut() {
//                 if let Some(route_config) = current_route_config.take() {
//                     // Inherit proxy_addr from domain if missing
//                     let route_addr = route_config
//                         .proxy_addr
//                         .clone()
//                         .unwrap_or_else(|| host_config.proxy_addr.clone());
//                     let mut completed_route = route_config.clone();
//                     completed_route.proxy_addr = Some(route_addr);
//                     host_config
//                         .routes
//                         .insert(current_route_name.unwrap(), completed_route);
//                 }
//                 proxy_config.servers.insert(domain, host_config);
//             }
//         }
//     }
//
//     if let Some(config) = current_proxy_config {
//         proxy_configs.push(config);
//     }
//
//     Ok(proxy_configs)
// }
//
// fn parse_headers(input: &str) -> Result<Vec<(String, String)>, String> {
//     let mut headers = Vec::new();
//     let entries: Vec<&str> = input
//         .trim_matches('[')
//         .trim_matches(']')
//         .split(',')
//         .map(|s| s.trim())
//         .collect();
//
//     for entry in entries {
//         if let Some((key, value)) = entry.split_once(':') {
//             headers.push((key.trim().to_string(), value.trim().to_string()));
//         } else {
//             return Err(format!("Invalid header format: {}", entry));
//         }
//     }
//
//     Ok(headers)
// }
//
// fn main() {
//     let input = r#"[proxy]
//     listener = "127.0.0.1:8080"
//     tls_certificate = "path/to/cert1"
//     tls_certificate_key = "path/to/key1"
//
//     [[ "domain1.com" ]]
//     proxy_addr = "/tmp/proxy.sock"
//     proxy_tls = true
//     proxy_headers = [["Header1": "Value1"]]
//     proxy_uds = true
//
//     [[[ "/route1" ]]]
//     proxy_addr = "192.168.1.2"
//     proxy_tls = false
//     proxy_headers = [["Header2": "Value2"]]
//     "#;
//
//     match parse_proxy_config(input) {
//         Ok(configs) => println!("Parsed configs: {:#?}", configs),
//         Err(err) => eprintln!("Error: {}", err),
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_valid_single_proxy_config() {
//         let input = r#"
//         [proxy]
//         listener = "127.0.0.1:8080"
//         tls_certificate = "path/to/cert1"
//         tls_certificate_key = "path/to/key1"
//
//         [["domain1.com"]]
//         proxy_addr = "/tmp/proxy.sock"
//         proxy_tls = true
//         proxy_headers = [["Header1": "Value1"]]
//         proxy_uds = true
//
//         [[[ "/route1" ]]]
//         proxy_addr = "192.168.1.2"
//         proxy_tls = false
//         proxy_headers = [["Header2": "Value2"]]
//         "#;
//
//         let result = parse_proxy_config(input);
//         assert!(result.is_ok());
//         let configs = result.unwrap();
//         assert_eq!(configs.len(), 1);
//         let config = &configs[0];
//         assert_eq!(config.listener, "127.0.0.1:8080");
//         assert!(config.servers.contains_key("domain1.com"));
//         let domain = config.servers.get("domain1.com").unwrap();
//         assert_eq!(domain.proxy_addr, "/tmp/proxy.sock");
//         assert!(domain.routes.contains_key("/route1"));
//     }
//
//     #[test]
//     fn test_missing_proxy_addr_in_route() {
//         let input = r#"
//         [proxy]
//         listener = "127.0.0.1:8081"
//
//         [["domain1.com"]]
//         proxy_addr = "10.0.0.1"
//         proxy_tls = true
//
//         [[[ "/route1" ]]]
//         proxy_tls = false
//         "#;
//
//         let result = parse_proxy_config(input);
//         println!("{result:#?}");
//         assert!(result.is_ok());
//         let configs = result.unwrap();
//         let config = &configs[0];
//         let domain = config.servers.get("domain1.com").unwrap();
//         let route = domain.routes.get("/route1").unwrap();
//         assert_eq!(route.proxy_addr, Some("10.0.0.1".to_string())); // Inherited address
//     }
//
//     #[test]
//     fn test_multiple_domains() {
//         let input = r#"
//         [proxy]
//         listener = "127.0.0.1:8082"
//
//         [["domain1.com"]]
//         proxy_addr = "10.0.0.2"
//         proxy_tls = true
//
//         [["domain2.com"]]
//         proxy_addr = "10.0.0.3"
//         proxy_tls = false
//         "#;
//
//         let result = parse_proxy_config(input);
//         assert!(result.is_ok());
//         let configs = result.unwrap();
//         let config = &configs[0];
//         assert!(config.servers.contains_key("domain1.com"));
//         assert!(config.servers.contains_key("domain2.com"));
//     }
//
//     #[test]
//     fn test_invalid_headers_format() {
//         let input = r#"
//         [proxy]
//         listener = "127.0.0.1:8083"
//
//         [["domain1.com"]]
//         proxy_addr = "10.0.0.4"
//         proxy_tls = true
//         proxy_headers = ["Header1", "Value1"]
//         "#;
//
//         let result = parse_proxy_config(input);
//         assert!(result.is_err());
//         assert!(result.err().unwrap().contains("Invalid header format"));
//     }
//
//     #[test]
//     fn test_empty_config() {
//         let input = r#"
//         [proxy]
//         listener = "127.0.0.1:8084"
//         "#;
//
//         let result = parse_proxy_config(input);
//         assert!(result.is_ok());
//         let configs = result.unwrap();
//         assert_eq!(configs.len(), 1);
//         let config = &configs[0];
//         assert!(config.servers.is_empty());
//     }
//
//     #[test]
//     fn test_multiple_proxies() {
//         let input = r#"
//         [proxy]
//         listener = "127.0.0.1:8085"
//
//         [["domain1.com"]]
//         proxy_addr = "10.0.0.5"
//
//         [proxy]
//         listener = "127.0.0.1:8086"
//
//         [["domain2.com"]]
//         proxy_addr = "10.0.0.6"
//         "#;
//
//         let result = parse_proxy_config(input);
//         assert!(result.is_ok());
//         let configs = result.unwrap();
//         assert_eq!(configs.len(), 2);
//         assert_eq!(configs[0].listener, "127.0.0.1:8085");
//         assert_eq!(configs[1].listener, "127.0.0.1:8086");
//     }
//
//     #[test]
//     fn test_missing_listener() {
//         let input = r#"
//         [proxy]
//         tls_certificate = "path/to/cert3"
//         "#;
//
//         let result = parse_proxy_config(input);
//         assert!(result.is_ok()); // Parser should handle this gracefully.
//         let configs = result.unwrap();
//         assert_eq!(configs.len(), 1);
//         assert_eq!(configs[0].listener, ""); // Default empty listener
//     }
// }

use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "proxy_config.pest"] // Path to the grammar file
struct ProxyConfigParser;

#[derive(Debug, Default)]
struct ProxyConfig {
    listener: String,
    tls_certificate: Option<String>,
    tls_certificate_key: Option<String>,
    servers: HashMap<String, ProxyHostConfig>,
}

#[derive(Debug, Default)]
struct ProxyHostConfig {
    proxy_addr: String,
    proxy_tls: bool,
    proxy_headers: Option<Vec<(String, String)>>,
    proxy_uds: bool,
    routes: HashMap<String, ProxyPathBaseHostConfig>,
}

#[derive(Debug, Default)]
struct ProxyPathBaseHostConfig {
    proxy_addr: Option<String>,
    proxy_tls: bool,
    proxy_headers: Option<Vec<(String, String)>>,
    proxy_uds: bool,
}

fn parse_proxy_config(input: &str) -> Result<Vec<ProxyConfig>, String> {
    let parsed = ProxyConfigParser::parse(Rule::document, input)
        .map_err(|e| format!("Parsing error: {}", e))?;

    let mut proxy_configs = Vec::new();
    let mut current_proxy = ProxyConfig::default();

    for record in parsed {
        match record.as_rule() {
            Rule::proxy_section => {
                if !current_proxy.listener.is_empty() {
                    proxy_configs.push(current_proxy);
                    current_proxy = ProxyConfig::default();
                }

                for pair in record.into_inner() {
                    match pair.as_rule() {
                        Rule::key_value_line => {
                            let mut inner = pair.into_inner();
                            let key = inner.next().unwrap().as_str();
                            let value = inner.next().unwrap().as_str();

                            match key {
                                "listener" => current_proxy.listener = value.to_string(),
                                "tls_certificate" => {
                                    current_proxy.tls_certificate = Some(value.to_string())
                                }
                                "tls_certificate_key" => {
                                    current_proxy.tls_certificate_key = Some(value.to_string())
                                }
                                _ => {}
                            }
                        }
                        Rule::domain_section => {
                            // Parse domains
                            // Similar logic applies here for nesting into domains and routes
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if !current_proxy.listener.is_empty() {
        proxy_configs.push(current_proxy);
    }

    Ok(proxy_configs)
}

fn main() {
    let input = r#"
    [proxy]
    listener = "127.0.0.1:8080"
    tls_certificate = "cert.pem"
    tls_certificate_key = "key.pem"
    
    [[ "example.com" ]]
    proxy_addr = "/tmp/proxy.sock"
    proxy_tls = true
    proxy_headers = [["Header1": "Value1"]]
    proxy_uds = true
    
    [[[ "/route" ]]]
    proxy_addr = "192.168.1.2"
    "#;

    match parse_proxy_config(input) {
        Ok(configs) => println!("Parsed configs: {:#?}", configs),
        Err(e) => eprintln!("Error: {}", e),
    }
}
