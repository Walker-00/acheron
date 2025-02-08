use manual::main3;
use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0, multispace1},
    combinator::{map, opt},
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated, tuple},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod idk;
mod manual;

// #[derive(Serialize, Deserialize)]
// struct ProxyConfig {
//     listener: String,
//     tls_certificate: Option<String>,
//     tls_certificate_key: Option<String>,
//     servers: HashMap<String, ProxyHostConfig>,
// }
//
// #[derive(Serialize, Deserialize)]
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
// #[derive(Serialize, Deserialize)]
// struct Config {
//     prometheus_addr: Option<String>,
//     proxy: Option<Vec<ProxyConfig>>,
//     load_balancer: Option<Vec<LoadBalancerConfig>>,
// }
// #[derive(Serialize, Deserialize)]
// pub struct LBHostConfig {
//     pub load_balancer_tls: bool,
//     pub load_balancer_headers: Option<Vec<(String, String)>>,
// }
// #[derive(Serialize, Deserialize)]
// pub struct ProxyHostConfig {
//     pub proxy_addr: String,
//     pub proxy_tls: bool,
//     pub proxy_headers: Option<Vec<(String, String)>>,
//     pub proxy_uds: Option<bool>,
// }

#[derive(Debug)]
struct Config {
    proxy: Option<ProxySection>,
    load_balancer: Option<LoadBalancerSection>,
}

#[derive(Debug)]
struct ProxySection {
    default_configs: HashMap<String, String>,
    domains: HashMap<String, DomainConfig>,
}

#[derive(Debug)]
struct DomainConfig {
    some_config: String,
    routes: HashMap<String, RouteConfig>,
}

#[derive(Debug)]
struct RouteConfig {
    some_config: String,
    more: String,
}

#[derive(Debug)]
struct LoadBalancerSection {
    default_configs: HashMap<String, String>,
    domains: HashMap<String, DomainConfig>,
}

// Parse an identifier (e.g., section names, keys, etc.)
fn parse_identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '.' || c == '_')(input)
}

// Parse a key-value pair
fn parse_key_value(input: &str) -> IResult<&str, (String, String)> {
    map(
        tuple((
            terminated(parse_identifier, multispace0),
            char('='),
            preceded(multispace0, take_while1(|c: char| !c.is_whitespace())),
        )),
        |(key, _, value)| (key.to_string(), value.to_string()),
    )
    .parse(input)
}

// Parse a section header like `[proxy]`
fn parse_section_header(input: &str) -> IResult<&str, &str> {
    preceded(
        multispace0,
        delimited(char('['), parse_identifier, char(']')),
    )
    .parse(input)
}

// Parse a domain header like `[["domain.com"]]`
fn parse_domain_header(input: &str) -> IResult<&str, &str> {
    delimited(tag("[[\""), parse_identifier, tag("\"]]")).parse(input)
}

// Parse a route header like `[[["/route1"]]]`
fn parse_route_header(input: &str) -> IResult<&str, &str> {
    delimited(tag("[[[\""), parse_identifier, tag("\"]]]")).parse(input)
}

// Parse the default config block
fn parse_default_config(input: &str) -> IResult<&str, HashMap<String, String>> {
    map(many0(terminated(parse_key_value, multispace0)), |pairs| {
        pairs.into_iter().collect()
    })
    .parse(input)
}

// Parse a domain section
fn parse_domain_section(input: &str) -> IResult<&str, (String, DomainConfig)> {
    let (input, domain_name) = parse_domain_header(input)?;
    let (input, _) = multispace1(input)?;
    let (input, some_config) = parse_key_value(input)?;
    let (input, _) = multispace1(input)?;

    // Parse route sections
    let (input, routes) = many0(parse_route_section).parse(input)?;

    let domain = DomainConfig {
        some_config: some_config.1,
        routes: routes.into_iter().collect(),
    };

    Ok((input, (domain_name.to_string(), domain)))
}

// Parse a route section
fn parse_route_section(input: &str) -> IResult<&str, (String, RouteConfig)> {
    let (input, route_name) = parse_route_header(input)?;
    let (input, _) = multispace1(input)?;
    let (input, some_config) = parse_key_value(input)?;
    let (input, more) = parse_key_value(input)?;

    let route = RouteConfig {
        some_config: some_config.1,
        more: more.1,
    };

    Ok((input, (route_name.to_string(), route)))
}

// Parse the entire config file
fn parse_config(input: &str) -> IResult<&str, Config> {
    let (_, proxy_section) = opt(parse_proxy_section).parse(input)?;
    let (_, load_balancer_section) = opt(parse_load_balancer_section).parse(input)?;

    Ok((input, Config {
        proxy: proxy_section,
        load_balancer: load_balancer_section,
    }))
}

// Parse the proxy section
fn parse_proxy_section(input: &str) -> IResult<&str, ProxySection> {
    let (input, _) = parse_section_header(input)?;
    let (input, _) = multispace1(input)?;
    let (input, default_configs) = parse_default_config(input)?;
    let (input, domains) = many0(parse_domain_section).parse(input)?;

    Ok((input, ProxySection {
        default_configs,
        domains: domains.into_iter().collect(),
    }))
}

// Parse the load_balancer section
fn parse_load_balancer_section(input: &str) -> IResult<&str, LoadBalancerSection> {
    let (input, _) = parse_section_header(input)?;
    let (input, idk) = multispace1(input)?;
    println!("{input:?}");
    let (input, default_configs) = parse_default_config(input)?;
    let (input, domains) = many0(parse_domain_section).parse(input)?;

    Ok((input, LoadBalancerSection {
        default_configs,
        domains: domains.into_iter().collect(),
    }))
}

fn main1() {
    let dsl = r#"
    [proxy]
    listener = "0.0.0.0:8080"

    [["domain.com"]]
    some_config = "domain-config"

    [[[ "/route1" ]]]
    some_config = "route1-config"
    more = "route1-more"

    [load_balancer]
    listener = "0.0.0.0:7070"
    "#;

    match parse_config(dsl) {
        Ok((_, config)) => println!("{:#?}", config),
        Err(e) => eprintln!("Error parsing config: {:?}", e),
    }
}

fn main() {
    main3();
}
