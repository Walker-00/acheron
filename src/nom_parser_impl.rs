use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{is_not, tag, take_while1},
    character::complete::{char, digit1, multispace0, space0},
    combinator::{map, map_res, opt},
    multi::separated_list0,
    sequence::{delimited, preceded, tuple},
};
use std::collections::HashMap;
use std::str::FromStr;

//
// Assume these structs come from your modules
//
#[derive(Debug, Default, PartialEq)]
pub struct Config {
    pub prometheus_addr: Option<String>,
    pub proxy: Option<Vec<ProxyConfig>>,
    pub load_balancer: Option<Vec<LoadBalancerConfig>>,
}

#[derive(Debug, Default, PartialEq)]
pub struct LoadBalancerConfig {
    pub listener: String,
    pub upstreams: Vec<String>,
    pub health_check: Option<bool>,
    pub health_check_frequency: Option<u64>,
    pub parallel_health_check: Option<bool>,
    pub tls_certificate: Option<String>,
    pub tls_certificate_key: Option<String>,
    pub servers: HashMap<String, LBHostConfig>,
}

#[derive(Debug, Default, PartialEq)]
pub struct LBHostConfig {
    pub load_balancer_tls: bool,
    pub load_balancer_headers: Option<Vec<(String, String)>>,
}

#[derive(Debug, Default, PartialEq)]
pub struct ProxyConfig {
    pub listener: String,
    pub tls_certificate: Option<String>,
    pub tls_certificate_key: Option<String>,
    pub servers: HashMap<String, ProxyHostConfig>,
}

#[derive(Debug, Default, PartialEq)]
pub struct ProxyHostConfig {
    pub proxy_addr: String,
    pub proxy_tls: bool,
    pub proxy_headers: Option<Vec<(String, String)>>,
    pub proxy_uds: Option<bool>,
    pub routes: Option<HashMap<String, ProxyPathBaseHostConfig>>,
}

#[derive(Debug, Default, PartialEq)]
pub struct ProxyPathBaseHostConfig {
    pub proxy_addr: Option<String>,
    pub proxy_tls: Option<bool>,
    pub proxy_headers: Option<Vec<(String, String)>>,
    pub proxy_uds: Option<bool>,
}

//
// Basic helpers and parsers
//

// name_char: ASCII_ALPHANUMERIC | "." | "_" | "/" | "-"
fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "._/-".contains(c)
}

// value_char: ASCII_ALPHANUMERIC | "." | "_" | "/" | "-" | "\"" | ":" | " "
fn is_value_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "._/ -\":".contains(c)
}

// Parse a name (one or more name_char)
fn parse_name(input: &str) -> IResult<&str, &str> {
    take_while1(is_name_char).parse(input)
}

// Parse a value (one or more value_char)
fn parse_value(input: &str) -> IResult<&str, &str> {
    take_while1(is_value_char).parse(input)
}

// Parse a boolean ("true" or "false")
fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false))).parse(input)
}

// Parse an unsigned integer (for health_check_frequency)
fn parse_u64(input: &str) -> IResult<&str, u64> {
    map_res(digit1, FromStr::from_str).parse(input)
}

//
// Key-value parsers
//

// listener = value
fn parse_listener(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((space0, tag("listener"), space0, char('='), space0)),
        parse_value,
    )
    .parse(input)
}

// tls_certificate = value
fn parse_tls_certificate(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((space0, tag("tls_certificate"), space0, char('='), space0)),
        parse_value,
    )
    .parse(input)
}

// tls_certificate_key = value
fn parse_tls_certificate_key(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((
            space0,
            tag("tls_certificate_key"),
            space0,
            char('='),
            space0,
        )),
        parse_value,
    )
    .parse(input)
}

// proxy_addr = value
fn parse_proxy_addr(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((space0, tag("proxy_addr"), space0, char('='), space0)),
        parse_value,
    )
    .parse(input)
}

// proxy_tls = bool
fn parse_proxy_tls(input: &str) -> IResult<&str, bool> {
    preceded(
        tuple((space0, tag("proxy_tls"), space0, char('='), space0)),
        parse_bool,
    )
    .parse(input)
}

// proxy_uds = bool
fn parse_proxy_uds(input: &str) -> IResult<&str, bool> {
    preceded(
        tuple((space0, tag("proxy_uds"), space0, char('='), space0)),
        parse_bool,
    )
    .parse(input)
}

// prometheus_addr = value
fn parse_prometheus_addr(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((space0, tag("prometheus_addr"), space0, char('='), space0)),
        parse_value,
    )
    .parse(input)
}

// upstreams = vector ([item, item, ...])
fn parse_vector(input: &str) -> IResult<&str, Vec<String>> {
    delimited(
        preceded(space0, char('[')),
        separated_list0(
            delimited(space0, char(','), space0),
            map(parse_value, |s: &str| s.to_string()),
        ),
        preceded(space0, char(']')),
    )
    .parse(input)
}

// health_check = bool
fn parse_health_check(input: &str) -> IResult<&str, bool> {
    preceded(
        tuple((space0, tag("health_check"), space0, char('='), space0)),
        parse_bool,
    )
    .parse(input)
}

// health_check_frequency = ASCII_DIGIT+
fn parse_health_check_frequency(input: &str) -> IResult<&str, u64> {
    preceded(
        tuple((
            space0,
            tag("health_check_frequency"),
            space0,
            char('='),
            space0,
        )),
        parse_u64,
    )
    .parse(input)
}

// parallel_health_check = bool
fn parse_parallel_health_check(input: &str) -> IResult<&str, bool> {
    preceded(
        tuple((
            space0,
            tag("parallel_health_check"),
            space0,
            char('='),
            space0,
        )),
        parse_bool,
    )
    .parse(input)
}

// load_balancer_tls = bool
fn parse_load_balancer_tls(input: &str) -> IResult<&str, bool> {
    preceded(
        tuple((space0, tag("load_balancer_tls"), space0, char('='), space0)),
        parse_bool,
    )
    .parse(input)
}

//
// Parsers for headers and vectors (for proxy_headers, load_balancer_headers, etc.)
// A header is defined as: key ":" key or key "," key (simplified).
//

fn parse_header_key(input: &str) -> IResult<&str, &str> {
    alt((delimited(char('"'), is_not("\""), char('"')), parse_name)).parse(input)
}

fn parse_header(input: &str) -> IResult<&str, (String, String)> {
    let (input, key) = parse_header_key.parse(input)?;
    let (input, _) = delimited(space0, alt((char(':'), char(','))), space0).parse(input)?;
    let (input, value) = parse_header_key.parse(input)?;
    Ok((input, (key.to_string(), value.to_string())))
}

fn parse_headers(input: &str) -> IResult<&str, Vec<(String, String)>> {
    delimited(
        preceded(space0, tag("[[")),
        separated_list0(
            delimited(space0, char(','), space0),
            delimited(char('['), parse_header, char(']')),
        ),
        preceded(space0, tag("]]")),
    )
    .parse(input)
}

//
// Parsers for sections
//

// Proxy section header: [proxy]
fn parse_proxy_section(input: &str) -> IResult<&str, ()> {
    let (input, _) = delimited(
        space0,
        delimited(char('['), tag("proxy"), char(']')),
        space0,
    )
    .parse(input)?;
    Ok((input, ()))
}

// Load balancer section header: [load_balancer]
fn parse_load_balancer_section(input: &str) -> IResult<&str, ()> {
    let (input, _) = delimited(
        space0,
        delimited(char('['), tag("load_balancer"), char(']')),
        space0,
    )
    .parse(input)?;
    Ok((input, ()))
}

// Domain section: [[ domain|host|ip = value ]]
fn parse_domain_section(input: &str) -> IResult<&str, &str> {
    let (input, _) = delimited(space0, tag("[["), space0).parse(input)?;
    let (input, _) = alt((tag("domain"), tag("host"), tag("ip"))).parse(input)?;
    let (input, _) = delimited(space0, char('='), space0).parse(input)?;
    let (input, domain) = parse_value.parse(input)?;
    let (input, _) = delimited(space0, tag("]]"), space0).parse(input)?;
    Ok((input, domain))
}

// Path section: [[[ route|path = value ]]]
fn parse_path_section(input: &str) -> IResult<&str, &str> {
    let (input, _) = delimited(space0, tag("[[["), space0).parse(input)?;
    let (input, _) = alt((tag("route"), tag("path"))).parse(input)?;
    let (input, _) = delimited(space0, char('='), space0).parse(input)?;
    let (input, path) = parse_value.parse(input)?;
    let (input, _) = delimited(space0, tag("]]]"), space0).parse(input)?;
    Ok((input, path))
}

//
// Parsers for proxy configuration
//

// Parse one proxy route config (the items inside a path section)
fn parse_proxy_route_config(input: &str) -> IResult<&str, ProxyPathBaseHostConfig> {
    let mut proxy_path = ProxyPathBaseHostConfig::default();
    let mut input = input;

    // Optionally parse a path section first.
    if let Ok((i, _)) = parse_path_section.parse(input) {
        input = i;
    }
    // Now, try to parse any of the proxy route items:
    let mut done = false;
    while !done {
        let res = alt((
            map(parse_proxy_addr, |s| {
                proxy_path.proxy_addr = Some(s.to_string())
            }),
            map(parse_proxy_tls, |b| proxy_path.proxy_tls = Some(b)),
            map(parse_proxy_uds, |b| proxy_path.proxy_uds = Some(b)),
            map(parse_headers, |hdrs| proxy_path.proxy_headers = Some(hdrs)),
        ))
        .parse(input);
        match res {
            Ok((i, _)) => {
                input = i;
            }
            Err(_) => {
                done = true;
            }
        }
    }
    Ok((input, proxy_path))
}

// Parse one proxy domain config block:
// domain_section ~ proxy configuration items ~ optional route configuration
fn parse_proxy_domain_config(input: &str) -> IResult<&str, (String, ProxyHostConfig)> {
    // First, parse the domain
    let (input, domain) = parse_domain_section.parse(input)?;

    // Then parse one or more proxy configuration items for this domain.
    let mut host = ProxyHostConfig::default();
    let mut input = input;
    let mut done = false;
    while !done {
        let res = alt((
            map(parse_proxy_addr, |s| {
                host.proxy_addr = s.to_string();
            }),
            map(parse_proxy_tls, |b| {
                host.proxy_tls = b;
            }),
            map(parse_proxy_uds, |b| {
                host.proxy_uds = Some(b);
            }),
            map(parse_headers, |hdrs| {
                host.proxy_headers = Some(hdrs);
            }),
        ))
        .parse(input);
        match res {
            Ok((i, _)) => {
                input = i;
            }
            Err(_) => {
                done = true;
            }
        }
    }

    // Optionally, parse route/path configuration
    if let Ok((i, route)) = parse_proxy_route_config.parse(input) {
        let mut routes = HashMap::new();
        // Use the proxy_addr from the route or a default key.
        routes.insert(
            route.proxy_addr.clone().unwrap_or_else(|| "default".into()),
            route,
        );
        host.routes = Some(routes);
        input = i;
    }
    Ok((input, (domain.to_string(), host)))
}

// Parse the proxy main configuration block:
// proxy_section ~ proxy_main_config ~ one or more domain blocks.
fn parse_proxy_config(input: &str) -> IResult<&str, ProxyConfig> {
    let (input, _) = parse_proxy_section.parse(input)?;

    // Parse proxy_main_config items (e.g. listener and tls_certificate/key).
    let mut config = ProxyConfig::default();
    let mut input = input;

    // For simplicity, we assume the listener must appear.
    let (i, listener) = parse_listener.parse(input)?;
    config.listener = listener.to_string();
    input = i;

    // Optionally, parse tls_certificate and tls_certificate_key as a pair.
    if let Ok((i, cert)) = parse_tls_certificate.parse(input) {
        if let Ok((j, key)) = parse_tls_certificate_key.parse(i) {
            config.tls_certificate = Some(cert.to_string());
            config.tls_certificate_key = Some(key.to_string());
            input = j;
        }
    }

    // Parse one or more domain blocks.
    let mut servers = HashMap::new();
    loop {
        if let Ok((i, (domain, host_config))) = parse_proxy_domain_config.parse(input) {
            servers.insert(domain, host_config);
            input = i;
        } else {
            break;
        }
    }
    config.servers = servers;
    Ok((input, config))
}

//
// Parsers for load balancer configuration
//

// Parse one load balancer domain config block:
// domain_section ~ optional load_balancer_tls and load_balancer_headers
fn parse_lb_domain_config(input: &str) -> IResult<&str, (String, LBHostConfig)> {
    let (input, domain) = parse_domain_section.parse(input)?;
    let mut host = LBHostConfig::default();
    let mut input = input;
    if let Ok((i, tls)) = parse_load_balancer_tls.parse(input) {
        host.load_balancer_tls = tls;
        input = i;
    }
    if let Ok((i, hdrs)) = parse_headers.parse(input) {
        host.load_balancer_headers = Some(hdrs);
        input = i;
    }
    Ok((input, (domain.to_string(), host)))
}

// Parse the lb_main_config block.
fn parse_lb_main_config(input: &str) -> IResult<&str, LoadBalancerConfig> {
    let (input, _) = parse_load_balancer_section.parse(input)?;
    let mut config = LoadBalancerConfig::default();
    let mut input = input;

    // listener is required.
    let (i, listener) = parse_listener.parse(input)?;
    config.listener = listener.to_string();
    input = i;

    // Parse upstreams
    if let Ok((i, ups)) = preceded(
        space0,
        preceded(
            tag("upstreams"),
            preceded(tuple((space0, char('='), space0)), parse_vector),
        ),
    )
    .parse(input)
    {
        config.upstreams = ups;
        input = i;
    }

    // Optionally, parse tls_certificate and key.
    if let Ok((i, cert)) = parse_tls_certificate.parse(input) {
        if let Ok((j, key)) = parse_tls_certificate_key.parse(i) {
            config.tls_certificate = Some(cert.to_string());
            config.tls_certificate_key = Some(key.to_string());
            input = j;
        }
    }

    // Optionally, parse health_check, frequency and parallel flag.
    if let Ok((i, check)) = parse_health_check.parse(input) {
        config.health_check = Some(check);
        input = i;
    }
    if let Ok((i, freq)) = parse_health_check_frequency.parse(input) {
        config.health_check_frequency = Some(freq);
        input = i;
    }
    if let Ok((i, par)) = opt(parse_parallel_health_check).parse(input) {
        config.parallel_health_check = par;
        input = i;
    }

    // Optionally, parse a domain config block.
    if let Ok((i, (domain, host))) = parse_lb_domain_config.parse(input) {
        config.servers.insert(domain, host);
        input = i;
    }
    Ok((input, config))
}

//
// Top-level config parser
//
pub fn parse_config(input: &str) -> IResult<&str, Config> {
    let mut config = Config::default();
    let mut input = input;

    // Optionally parse prometheus_addr if it appears before any section.
    if let Ok((i, addr)) = parse_prometheus_addr.parse(input) {
        config.prometheus_addr = Some(addr.to_string());
        input = i;
    }

    // Now parse one or more proxy or load balancer sections.
    loop {
        // Skip any whitespace
        let (i, _) = multispace0.parse(input)?;
        input = i;
        if input.is_empty() {
            break;
        }
        // Try proxy config
        if let Ok((i, proxy_cfg)) = parse_proxy_config.parse(input) {
            config.proxy.get_or_insert(Vec::new()).push(proxy_cfg);
            input = i;
            continue;
        }
        // Try load balancer config
        if let Ok((i, lb_cfg)) = parse_lb_main_config.parse(input) {
            config.load_balancer.get_or_insert(Vec::new()).push(lb_cfg);
            input = i;
            continue;
        }
        // If neither matches, break.
        break;
    }
    Ok((input, config))
}

//
// Tests for all test cases
//
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_prometheus_addr() {
        let input = "prometheus_addr = 127.0.0.1:9090";
        let (rest, addr) = parse_prometheus_addr.parse(input).unwrap();
        assert_eq!(addr, "127.0.0.1:9090");
        assert!(rest.trim().is_empty());
    }

    #[test]
    fn test_parse_proxy_config() {
        let input = r#"
        [proxy]
        listener = 0.0.0.0:8080
        tls_certificate = /path/to/cert
        tls_certificate_key = /path/to/key
        [[domain = example.com]]
        proxy_addr = 192.168.1.1:80
        proxy_tls = true
        proxy_headers = [[["Content-Type" : "application/json"]]]
        "#;
        let (rest, proxy_cfg) = parse_proxy_config.parse(input).unwrap();
        assert_eq!(proxy_cfg.listener, "0.0.0.0:8080");
        assert_eq!(
            proxy_cfg.tls_certificate.unwrap(),
            "/path/to/cert".to_string()
        );
        assert_eq!(
            proxy_cfg.tls_certificate_key.unwrap(),
            "/path/to/key".to_string()
        );
        // Check that domain exists in servers
        let host_cfg = proxy_cfg.servers.get("example.com").unwrap();
        assert_eq!(host_cfg.proxy_addr, "192.168.1.1:80");
        assert_eq!(host_cfg.proxy_tls, true);
        // Check proxy_headers if exist
        assert!(host_cfg.proxy_headers.is_some());
        let headers = host_cfg.proxy_headers.as_ref().unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "Content-Type");
        assert_eq!(headers[0].1, "application/json");
        // Ensure all input was consumed or valid remaining whitespace.
        assert!(rest.trim().is_empty());
    }

    #[test]
    fn test_parse_lb_main_config() {
        let input = r#"
        [load_balancer]
        listener = 0.0.0.0:9090
        upstreams = [backend1, backend2]
        tls_certificate = /path/to/lb/cert
        tls_certificate_key = /path/to/lb/key
        health_check = true
        health_check_frequency = 30
        parallel_health_check = false
        [[domain = lb.example.com]]
        load_balancer_tls = true
        load_balancer_headers = [[["X-Forwarded-Proto" : "https"]]]
        "#;
        let (rest, lb_cfg) = parse_lb_main_config.parse(input).unwrap();
        assert_eq!(lb_cfg.listener, "0.0.0.0:9090");
        assert_eq!(
            lb_cfg.upstreams,
            vec!["backend1".to_string(), "backend2".to_string()]
        );
        assert_eq!(
            lb_cfg.tls_certificate.unwrap(),
            "/path/to/lb/cert".to_string()
        );
        assert_eq!(
            lb_cfg.tls_certificate_key.unwrap(),
            "/path/to/lb/key".to_string()
        );
        assert_eq!(lb_cfg.health_check, Some(true));
        assert_eq!(lb_cfg.health_check_frequency, Some(30));
        assert_eq!(lb_cfg.parallel_health_check, Some(false));
        let host_cfg = lb_cfg.servers.get("lb.example.com").unwrap();
        assert_eq!(host_cfg.load_balancer_tls, true);
        let headers = host_cfg.load_balancer_headers.as_ref().unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "X-Forwarded-Proto");
        assert_eq!(headers[0].1, "https");
        assert!(rest.trim().is_empty());
    }

    #[test]
    fn test_parse_config_combined() {
        let input = r#"
        prometheus_addr = 127.0.0.1:9090

        [proxy]
        listener = 0.0.0.0:8080
        tls_certificate = /path/to/cert
        tls_certificate_key = /path/to/key
        [[domain = example.com]]
        proxy_addr = 192.168.1.1:80
        proxy_tls = false

        [load_balancer]
        listener = 0.0.0.0:9090
        upstreams = [backend1, backend2]
        health_check = true
        health_check_frequency = 60
        [[domain = lb.example.com]]
        load_balancer_tls = false
        "#;
        let (rest, cfg) = parse_config.parse(input).unwrap();
        assert_eq!(cfg.prometheus_addr.unwrap(), "127.0.0.1:9090".to_string());
        // Check proxy config exists
        let proxy_cfgs = cfg.proxy.unwrap();
        assert_eq!(proxy_cfgs.len(), 1);
        let proxy_cfg = &proxy_cfgs[0];
        assert_eq!(proxy_cfg.listener, "0.0.0.0:8080");
        let host_cfg = proxy_cfg.servers.get("example.com").unwrap();
        assert_eq!(host_cfg.proxy_addr, "192.168.1.1:80");
        assert_eq!(host_cfg.proxy_tls, false);
        // Check load balancer config exists
        let lb_cfgs = cfg.load_balancer.unwrap();
        assert_eq!(lb_cfgs.len(), 1);
        let lb_cfg = &lb_cfgs[0];
        assert_eq!(lb_cfg.listener, "0.0.0.0:9090");
        assert_eq!(
            lb_cfg.upstreams,
            vec!["backend1".to_string(), "backend2".to_string()]
        );
        assert_eq!(lb_cfg.health_check, Some(true));
        assert_eq!(lb_cfg.health_check_frequency, Some(60));
        let lb_host = lb_cfg.servers.get("lb.example.com").unwrap();
        assert_eq!(lb_host.load_balancer_tls, false);
        assert!(rest.trim().is_empty());
    }
}
