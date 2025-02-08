use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0, multispace1},
    combinator::{map, opt},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, separated_pair, terminated},
};
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

// Parse a key-value pair, e.g., `key = value`
fn parse_key_value(input: &str) -> IResult<&str, (String, String)> {
    let (input, key) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, value) = take_while1(|c: char| c != '\n' && c != ',')(input)?;

    Ok((input, (key.to_string(), value.trim().to_string())))
}

// Parse a section header, e.g., `[proxy]`
fn parse_section_header(input: &str) -> IResult<&str, String> {
    let (input, _) = char('[')(input)?;
    let (input, section) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    let (input, _) = char(']')(input)?;
    Ok((input, section.to_string()))
}

// Parse a server block, e.g., `[["domain.com"]]`
fn parse_server_header(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("[[\"")(input)?;
    let (input, domain) = take_while1(|c: char| c != '"')(input)?;
    let (input, _) = tag("\"]]")(input)?;
    Ok((input, domain.to_string()))
}

// Parse the main config structure
fn parse_config(input: &str) -> IResult<&str, Config> {
    let (input, prometheus_addr) = opt(preceded(
        tag("prometheus_addr"),
        preceded(
            char('='),
            delimited(multispace0, take_while1(|c| c != '\n'), multispace0),
        ),
    ))
    .parse(input)?;

    // Parse sections (e.g., [proxy], [load_balancer])
    let (input, proxy) = many0(parse_proxy_section).parse(input)?;
    let (input, load_balancer) = many0(parse_load_balancer_section).parse(input)?;

    Ok((input, Config {
        prometheus_addr: prometheus_addr.map(|s| s.to_string()),
        proxy: Some(proxy),
        load_balancer: Some(load_balancer),
    }))
}

// Parse a proxy section
fn parse_proxy_section(input: &str) -> IResult<&str, ProxyConfig> {
    let (input, _) = parse_section_header(input)?;
    // Parse proxy-specific details here
    unimplemented!()
}

// Parse a load balancer section
fn parse_load_balancer_section(input: &str) -> IResult<&str, LoadBalancerConfig> {
    let (input, _) = parse_section_header(input)?;
    // Parse load-balancer-specific details here
    unimplemented!()
}

pub fn main2() {
    let input = r#"prometheus_addr = \"0.0.0.0:9090\"

[proxy]
listener = \"0.0.0.0:8080\"

[[\"example.com\"]]
proxy_addr = \"/tmp/example.sock\"
proxy_tls = false
proxy_headers = [[\"X-Example-Header\", \"value\"]]
proxy_uds = true
"#;

    match parse_config(input) {
        Ok((_, config)) => println!("{:?}", config),
        Err(err) => eprintln!("Failed to parse config: {:?}", err),
    }
}
