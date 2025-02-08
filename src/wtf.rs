// enum Tokens {
//     MainSectionStart,
//     MainSectionEnd,
//     DomainSectionStart,
//     DomainSectionEnd,
//     RouteSectionStart,
//     RouteSectionEnd,
//     ProxySection,
//     LoadBalancerSection,
// }
//
// fn parser(input: &str) {
//     let tokens: Vec<Tokens> = vec![];
//     for l in input.lines() {
//
//         if l.starts_with("[[[") && l.ends_with("]]]") {
//             tokens.push(Tokens::RouteSectionStart);
//         }
//     }
// }

// use pest::Parser;
// use pest_derive::Parser;
// use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use std::sync::Arc;
//
// #[derive(Parser)]
// #[grammar = "./dsl.pest"] // The grammar file should be named "dsl.pest"
// struct DSLParser;
//
// #[derive(Serialize, Deserialize, Debug)]
// struct Config {
//     prometheus_addr: Option<String>,
//     proxy: Option<Vec<ProxyConfig>>,
//     load_balancer: Option<Vec<LoadBalancerConfig>>,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// struct ProxyConfig {
//     listener: String,
//     tls_certificate: Option<String>,
//     tls_certificate_key: Option<String>,
//     servers: HashMap<String, ProxyHostConfig>,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// pub struct ProxyHostConfig {
//     pub proxy_addr: String,
//     pub proxy_tls: bool,
//     pub proxy_headers: Option<Vec<(String, String)>>,
//     pub proxy_uds: Option<bool>,
//     pub routes: Option<HashMap<String, ProxyPathBaseHostConfig>>,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// pub struct ProxyPathBaseHostConfig {
//     pub proxy_addr: String,
//     pub proxy_tls: bool,
//     pub proxy_headers: Option<Vec<(String, String)>>,
//     pub proxy_uds: Option<bool>,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
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
// #[derive(Serialize, Deserialize, Debug)]
// pub struct LBHostConfig {
//     pub load_balancer_tls: bool,
//     pub load_balancer_headers: Option<Vec<(String, String)>>,
// }
//
// impl Config {
//     pub fn from_dsl(input: &str) -> Result<Self, String> {
//         let parsed = DSLParser::parse(Rule::file, input).map_err(|e| e.to_string())?;
//
//         let mut prometheus_addr = None;
//         let mut proxies = vec![];
//         let mut load_balancers = vec![];
//
//         for pair in parsed {
//             match pair.as_rule() {
//                 Rule::super_config => {
//                     println!("{:?}", pair.as_span().as_str().trim().to_string());
//                     prometheus_addr = Some(pair.clone().into_inner().as_str().trim().to_string());
//                 }
//                 Rule::proxy_config => {
//                     proxies.push(parse_proxy_config(pair)?);
//                 }
//                 Rule::lb_config => {
//                     load_balancers.push(parse_lb_config(pair)?);
//                 }
//                 _ => {}
//             }
//         }
//
//         Ok(Config {
//             prometheus_addr,
//             proxy: if proxies.is_empty() {
//                 None
//             } else {
//                 Some(proxies)
//             },
//             load_balancer: if load_balancers.is_empty() {
//                 None
//             } else {
//                 Some(load_balancers)
//             },
//         })
//     }
// }
//
// fn parse_proxy_config(pair: pest::iterators::Pair<Rule>) -> Result<ProxyConfig, String> {
//     let mut listener = String::new();
//     let mut tls_certificate = None;
//     let mut tls_certificate_key = None;
//     let mut servers = HashMap::new();
//
//     for inner in pair.into_inner() {
//         match inner.as_rule() {
//             Rule::listener => listener = inner.as_str().trim().to_string(),
//             Rule::tls_certificate => tls_certificate = Some(inner.as_str().trim().to_string()),
//             Rule::tls_certificate_key => {
//                 tls_certificate_key = Some(inner.as_str().trim().to_string())
//             }
//             Rule::domain => {
//                 let (domain_name, domain_config) = parse_proxy_host(inner)?;
//                 servers.insert(domain_name, domain_config);
//             }
//             _ => {}
//         }
//     }
//
//     Ok(ProxyConfig {
//         listener,
//         tls_certificate,
//         tls_certificate_key,
//         servers,
//     })
// }
//
// fn parse_proxy_host(
//     pair: pest::iterators::Pair<Rule>,
// ) -> Result<(String, ProxyHostConfig), String> {
//     let mut domain_name = String::new();
//     let mut proxy_addr = String::new();
//     let mut proxy_tls = false;
//     let mut proxy_headers = None;
//     let mut proxy_uds = None;
//     let mut routes = HashMap::new();
//
//     for inner in pair.into_inner() {
//         match inner.as_rule() {
//             Rule::domain_header => domain_name = inner.as_str().trim().to_string(),
//             Rule::config_entry => {
//                 // Parse additional domain-specific configurations here
//             }
//             Rule::route => {
//                 let (route_name, route_config) = parse_route(inner)?;
//                 routes.insert(route_name, route_config);
//             }
//             _ => {}
//         }
//     }
//
//     Ok((domain_name, ProxyHostConfig {
//         proxy_addr,
//         proxy_tls,
//         proxy_headers,
//         proxy_uds,
//         routes: if routes.is_empty() {
//             None
//         } else {
//             Some(routes)
//         },
//     }))
// }
//
// fn parse_route(
//     pair: pest::iterators::Pair<Rule>,
// ) -> Result<(String, ProxyPathBaseHostConfig), String> {
//     let mut route_name = String::new();
//     let mut proxy_addr = String::new();
//     let mut proxy_tls = false;
//     let mut proxy_headers = None;
//     let mut proxy_uds = None;
//
//     for inner in pair.into_inner() {
//         match inner.as_rule() {
//             Rule::route_header => route_name = inner.as_str().trim().to_string(),
//             Rule::config_entry => {
//                 // Parse additional route-specific configurations here
//             }
//             _ => {}
//         }
//     }
//
//     Ok((route_name, ProxyPathBaseHostConfig {
//         proxy_addr,
//         proxy_tls,
//         proxy_headers,
//         proxy_uds,
//     }))
// }
//
// fn parse_lb_config(pair: pest::iterators::Pair<Rule>) -> Result<LoadBalancerConfig, String> {
//     let mut listener = String::new();
//     let mut upstreams = vec![];
//     let mut health_check = None;
//     let mut health_check_frequency = None;
//     let mut parallel_health_check = None;
//     let mut tls_certificate = None;
//     let mut tls_certificate_key = None;
//     let mut servers = HashMap::new();
//
//     for inner in pair.into_inner() {
//         match inner.as_rule() {
//             Rule::listener => listener = inner.as_str().trim().to_string(),
//             Rule::tls_certificate => tls_certificate = Some(inner.as_str().trim().to_string()),
//             Rule::tls_certificate_key => {
//                 tls_certificate_key = Some(inner.as_str().trim().to_string())
//             }
//             Rule::health_check => health_check = Some(inner.as_str() == "true"),
//             Rule::health_check_frequency => health_check_frequency = inner.as_str().parse().ok(),
//             Rule::parallel_health_check => parallel_health_check = Some(inner.as_str() == "true"),
//             Rule::domain => {
//                 let (domain_name, domain_config) = parse_lb_host(inner)?;
//                 servers.insert(domain_name, domain_config);
//             }
//             _ => {}
//         }
//     }
//
//     Ok(LoadBalancerConfig {
//         listener,
//         upstreams,
//         health_check,
//         health_check_frequency,
//         parallel_health_check,
//         tls_certificate,
//         tls_certificate_key,
//         servers,
//     })
// }
//
// fn parse_lb_host(pair: pest::iterators::Pair<Rule>) -> Result<(String, LBHostConfig), String> {
//     let mut domain_name = String::new();
//     let mut load_balancer_tls = false;
//     let mut load_balancer_headers = None;
//
//     for inner in pair.into_inner() {
//         match inner.as_rule() {
//             Rule::domain_header => domain_name = inner.as_str().trim().to_string(),
//             Rule::config_entry => {
//                 // Parse additional LBHostConfig-specific configurations here
//             }
//             _ => {}
//         }
//     }
//
//     Ok((domain_name, LBHostConfig {
//         load_balancer_tls,
//         load_balancer_headers,
//     }))
// }
//
// pub fn main4() {
//     let dsl = r#"prometheus_addr = "127.0.0.1:9090" [proxy] listener = "0.0.0.0:8080" tls_certificate = "/path/to/cert"
// tls_certificate_key = "/path/to/key"
//
// [["domain.com"]]
// proxy_addr = "http://localhost:8081"
// proxy_tls = true
//
// [[[ "/route1" ]]]
// proxy_addr = "http://localhost:8082"
// proxy_tls = false
//
// [load_balancer]
// listener = "0.0.0.0:9090"
// health_check = true
// health_check_frequency = 10
//
//     "#;
//
//     match Config::from_dsl(dsl) {
//         Ok(config) => println!("{:#?}", config),
//         Err(err) => eprintln!("Failed to parse DSL: {}", err),
//     }
// }

use std::collections::HashMap;
use std::fs;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "idk.rs"]
pub struct INIParser;

fn main() {
    let unparsed_file = fs::read_to_string("config.ini").expect("cannot read file");

    let file = INIParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap(); // get and unwrap the `file` rule; never fails

    let mut properties: HashMap<&str, HashMap<&str, &str>> = HashMap::new();

    let mut current_section_name = "";

    for line in file.into_inner() {
        match line.as_rule() {
            Rule::section => {
                let mut inner_rules = line.into_inner(); // { name }
                current_section_name = inner_rules.next().unwrap().as_str();
            }
            Rule::property => {
                let mut inner_rules = line.into_inner(); // { name ~ "=" ~ value }

                let name: &str = inner_rules.next().unwrap().as_str();
                let value: &str = inner_rules.next().unwrap().as_str();

                // Insert an empty inner hash map if the outer hash map hasn't
                // seen this section name before.
                let section = properties.entry(current_section_name).or_default();
                section.insert(name, value);
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
}
