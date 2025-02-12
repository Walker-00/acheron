use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use structures::{
    load_balancer_structure::LoadBalancerConfig,
    proxy_structure::{ProxyConfig, ProxyHostConfig, ProxyPathBaseHostConfig},
};

mod structures;

#[derive(Parser)]
#[grammar = "done.pest"]
pub struct ConfigParser;

fn parse_proxy_config(pair: pest::iterators::Pair<Rule>) -> ProxyConfig {
    let mut listener = String::new();
    let mut tls_certificate = None;
    let mut tls_certificate_key = None;
    let mut servers = HashMap::new();

    for pairs in pair.into_inner() {
        let pair = pairs.clone().into_inner();
        let pair = pair.as_str().trim().trim_matches('"');
        match pairs.as_rule() {
            Rule::listener => listener = pair.to_string(),
            Rule::tls_certificate => tls_certificate = Some(pair.to_string()),
            Rule::tls_certificate_key => tls_certificate_key = Some(pair.to_string()),
            Rule::proxy_domain_base_config => {
                let (key, host_config) = parse_proxy_domain_config(pairs);
                servers.insert(key, host_config);
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

fn parse_proxy_domain_config(pair: pest::iterators::Pair<Rule>) -> (String, ProxyHostConfig) {
    let mut domain = String::new();
    let mut proxy_addr = String::new();
    let mut proxy_tls = false;
    let mut proxy_headers = None;
    let mut proxy_uds = false;
    let mut routes = HashMap::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::domain_section => {
                domain = pair.into_inner().next().unwrap().as_str().to_string();
            }
            Rule::proxy_addr => {
                proxy_addr = pair
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .trim()
                    .trim_matches('"')
                    .to_string()
            }
            Rule::proxy_tls => proxy_tls = pair.as_str() == "true",
            Rule::proxy_headers => {
                proxy_headers = Some(parse_headers(pair));
            }
            Rule::proxy_uds => proxy_uds = pair.as_str() == "true",
            Rule::proxy_route_base_config => {
                let (key, route_config) = parse_proxy_route_config(pair);
                routes.insert(key, route_config);
            }
            _ => {}
        }
    }

    (domain, ProxyHostConfig {
        proxy_addr,
        proxy_tls,
        proxy_headers,
        proxy_uds: Some(proxy_uds),
        routes: Some(routes),
    })
}

fn parse_proxy_route_config(
    pair: pest::iterators::Pair<Rule>,
) -> (String, ProxyPathBaseHostConfig) {
    let mut path = String::new();
    let mut proxy_addr = None;
    let mut proxy_tls = false;
    let mut proxy_headers = None;
    let mut proxy_uds = false;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::path_section => {
                path = pair.into_inner().next().unwrap().as_str().to_string();
            }
            Rule::proxy_addr => {
                proxy_addr = Some(
                    pair.into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .trim()
                        .trim_matches('"')
                        .to_string(),
                )
            }
            Rule::proxy_tls => proxy_tls = pair.as_str() == "true",
            Rule::proxy_headers => {
                proxy_headers = Some(parse_headers(pair));
            }
            Rule::proxy_uds => proxy_uds = pair.as_str() == "true",
            _ => {}
        }
    }

    (path, ProxyPathBaseHostConfig {
        proxy_addr,
        proxy_tls: Some(proxy_tls),
        proxy_headers,
        proxy_uds: Some(proxy_uds),
    })
}

fn parse_load_balancer_config(pair: pest::iterators::Pair<Rule>) -> LoadBalancerConfig {
    let mut listener = String::new();
    let mut upstreams = Vec::new();
    let mut health_check = None;
    let mut health_check_frequency = None;
    let mut parallel_health_check = None;
    let mut tls_certificate = None;
    let mut tls_certificate_key = None;
    let mut servers = HashMap::new();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::listener => {
                listener = pair
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .trim()
                    .trim_matches('"')
                    .to_string()
            }
            Rule::upstreams => {
                upstreams = pair
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .split(',')
                    .map(|x| {
                        x.trim()
                            .trim_matches('[')
                            .trim_matches(']')
                            .trim_matches('"')
                            .to_string()
                    })
                    .collect::<Vec<String>>()
            }
            Rule::health_check => health_check = Some(pair.as_str() == "true"),
            Rule::health_check_frequency => {
                let hcf = pair.as_str().split('=').collect::<Vec<&str>>();
                let hcf = hcf[1].trim().trim_matches(' ');
                health_check_frequency = Some(hcf.parse::<u64>().unwrap())
            }
            Rule::parallel_health_check => parallel_health_check = Some(pair.as_str() == "true"),
            Rule::lb_domain_base_config => {
                let (key, host_config) = parse_lb_host_config(pair);
                servers.insert(key, host_config);
            }
            Rule::tls_certificate => {
                tls_certificate = Some(
                    pair.into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .trim()
                        .trim_matches('"')
                        .to_string(),
                )
            }
            Rule::tls_certificate_key => {
                tls_certificate_key = Some(
                    pair.into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .trim()
                        .trim_matches('"')
                        .to_string(),
                )
            }
            _ => {}
        }
    }

    LoadBalancerConfig {
        listener,
        upstreams,
        health_check,
        health_check_frequency,
        parallel_health_check,
        tls_certificate,
        tls_certificate_key,
        servers,
    }
}

fn parse_lb_host_config(pair: pest::iterators::Pair<Rule>) -> (String, LBHostConfig) {
    let mut domain = String::new();
    let mut load_balancer_tls = false;
    let mut load_balancer_headers = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::domain_section => {
                domain = pair.into_inner().next().unwrap().as_str().to_string();
            }
            Rule::load_balancer_tls => load_balancer_tls = pair.as_str() == "true",
            Rule::load_balancer_headers => {
                load_balancer_headers = Some(parse_headers(pair));
            }
            _ => {}
        }
    }

    (domain, LBHostConfig {
        load_balancer_tls,
        load_balancer_headers,
    })
}

fn parse_headers(pair: pest::iterators::Pair<Rule>) -> Vec<(String, String)> {
    pair.into_inner()
        .map(|header_pair| {
            let mut inner = header_pair.into_inner();
            let bind = inner.next().unwrap().as_str().to_string();
            let key = bind
                .split(':')
                .map(|x| x.trim().trim_matches(' ').trim_matches('"'))
                .collect::<Vec<&str>>();
            (key[0].to_string(), key[1].to_string())
        })
        .collect()
}

fn main() {
    let input = std::fs::read_to_string("config.chr").unwrap();
    let parsed = ConfigParser::parse(Rule::file, &input)
        .expect("Failed to parse input")
        .next()
        .unwrap();
    let mut config = Config::default();

    for pair in parsed.into_inner() {
        match pair.as_rule() {
            Rule::prometheus_addr => {
                config.prometheus_addr = Some(
                    pair.into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .trim()
                        .trim_matches('"')
                        .to_string(),
                );
            }
            Rule::main_proxy_config => {
                let proxy_config = parse_proxy_config(pair);

                if let Some(ref mut proxy_configs) = config.proxy {
                    proxy_configs.push(proxy_config);
                } else {
                    config.proxy = Some(vec![proxy_config]);
                }
            }
            Rule::main_lb_config => {
                let load_balancer_config = parse_load_balancer_config(pair);

                if let Some(ref mut lb_configs) = config.load_balancer {
                    lb_configs.push(load_balancer_config);
                } else {
                    config.load_balancer = Some(vec![load_balancer_config])
                }
            }
            Rule::EOI => {
                println!("{config:#?}");
            }
            _ => {
                println!("No Match {pair}");
            }
        }
    }
}
