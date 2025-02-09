use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "done.pest"] // This is the path to your .pest file
pub struct ConfigParser;

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

fn parse_proxy_config(pair: pest::iterators::Pair<Rule>) -> ProxyConfig {
    let mut listener = String::new();
    let mut tls_certificate = None;
    let mut tls_certificate_key = None;
    let mut servers = HashMap::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::listener => listener = pair.as_str().to_string(),
            Rule::tls_certificate => tls_certificate = Some(pair.as_str().to_string()),
            Rule::tls_certificate_key => tls_certificate_key = Some(pair.as_str().to_string()),
            Rule::proxy_domain_base_config => {
                let host_config = parse_proxy_host_config(pair);
                servers.insert(host_config.proxy_addr.clone(), host_config);
            }
            _ => {}
        }
    }

    ProxyConfig {
        listener,
        tls_certificate,
        tls_certificate_key,
        servers,
    }
}

fn parse_proxy_host_config(pair: pest::iterators::Pair<Rule>) -> ProxyHostConfig {
    let mut proxy_addr = String::new();
    let mut proxy_tls = false;
    let mut proxy_headers = None;
    let mut proxy_uds = false;
    let mut routes = HashMap::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::proxy_addr => proxy_addr = pair.as_str().to_string(),
            Rule::proxy_tls => proxy_tls = pair.as_str() == "true",
            Rule::proxy_headers => {
                proxy_headers = Some(parse_headers(pair));
            }
            Rule::proxy_uds => proxy_uds = pair.as_str() == "true",
            Rule::proxy_route_base_config => {
                let route = parse_proxy_path_base_host_config(pair);
                routes.insert(route.proxy_addr.clone().unwrap_or_default(), route);
            }
            _ => {}
        }
    }

    ProxyHostConfig {
        proxy_addr,
        proxy_tls,
        proxy_headers,
        proxy_uds,
        routes,
    }
}

fn parse_proxy_path_base_host_config(pair: pest::iterators::Pair<Rule>) -> ProxyPathBaseHostConfig {
    let mut proxy_addr = None;
    let mut proxy_tls = false;
    let mut proxy_headers = None;
    let mut proxy_uds = false;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::proxy_addr => proxy_addr = Some(pair.as_str().to_string()),
            Rule::proxy_tls => proxy_tls = pair.as_str() == "true",
            Rule::proxy_headers => {
                proxy_headers = Some(parse_headers(pair));
            }
            Rule::proxy_uds => proxy_uds = pair.as_str() == "true",
            _ => {}
        }
    }

    ProxyPathBaseHostConfig {
        proxy_addr,
        proxy_tls,
        proxy_headers,
        proxy_uds,
    }
}

fn parse_headers(pair: pest::iterators::Pair<Rule>) -> Vec<(String, String)> {
    pair.into_inner()
        .map(|header_pair| {
            let mut inner = header_pair.into_inner();
            let key = inner.next().unwrap().as_str().to_string();
            let value = inner.next().unwrap().as_str().to_string();
            (key, value)
        })
        .collect()
}

// Add similar functions for LoadBalancerConfig and LBHostConfig

fn main() {
    let input = std::fs::read_to_string("config.txt").unwrap();
    let parsed = ConfigParser::parse(Rule::file, &input).expect("Failed to parse input");

    for pair in parsed {
        match pair.as_rule() {
            Rule::main_proxy_config => {
                let proxy_config = parse_proxy_config(pair);
                println!("{:?}", proxy_config);
            }
            Rule::main_lb_config => {
                // Implement similar parsing for load balancer config
            }
            _ => {}
        }
    }
}
