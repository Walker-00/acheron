use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{char, multispace0, multispace1},
    combinator::{map, opt},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, tuple},
};
use std::collections::HashMap;

// Data structures for parsed DSL
#[derive(Debug)]
struct Proxy {
    listener: Option<String>,
    tls_certificate: Option<String>,
    tls_certificate_key: Option<String>,
    domains: Vec<Domain>,
}

#[derive(Debug)]
struct Domain {
    name: String,
    proxy_addr: Option<String>,
    proxy_tls: Option<bool>,
    proxy_headers: Option<Vec<(String, String)>>,
    proxy_uds: Option<bool>,
    routes: Vec<Route>,
}

#[derive(Debug)]
struct Route {
    path: String,
    proxy_addr: Option<String>,
    proxy_tls: Option<bool>,
    proxy_headers: Option<Vec<(String, String)>>,
    proxy_uds: Option<bool>,
}

// Parser functions
fn parse_key_value(input: &str) -> IResult<&str, (String, String)> {
    map(
        tuple((
            take_while(|c: char| c.is_alphanumeric() || c == '_'),
            preceded(multispace0, char('=')),
            preceded(multispace0, take_while(|c: char| c != '\n')),
        )),
        |(key, _, value)| (key.trim().to_string(), value.trim().to_string()),
    )(input)
}

fn parse_section(input: &str) -> IResult<&str, String> {
    delimited(char('['), take_while(|c: char| c != ']'), char(']'))(input)
}

fn parse_global_section(input: &str) -> IResult<&str, Proxy> {
    let (input, _) = tag("[proxy]")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, fields) = many0(parse_key_value)(input)?;

    let mut listener = None;
    let mut tls_certificate = None;
    let mut tls_certificate_key = None;

    for (key, value) in fields {
        match key.as_str() {
            "listener" => listener = Some(value),
            "tls_certificate" => tls_certificate = Some(value),
            "tls_certificate_key" => tls_certificate_key = Some(value),
            _ => {}
        }
    }

    Ok((input, Proxy {
        listener,
        tls_certificate,
        tls_certificate_key,
        domains: vec![],
    }))
}

fn parse_domain_section(input: &str) -> IResult<&str, Domain> {
    let (input, _) = tag("[[\"domain\"]]")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, fields) = many0(parse_key_value)(input)?;

    let mut name = String::new();
    let mut proxy_addr = None;
    let mut proxy_tls = None;
    let mut proxy_headers = None;
    let mut proxy_uds = None;

    for (key, value) in fields {
        match key.as_str() {
            "name" => name = value,
            "proxy_addr" => proxy_addr = Some(value),
            "proxy_tls" => proxy_tls = Some(value == "true"),
            "proxy_headers" => {
                let headers: Vec<(String, String)> = value
                    .split(',')
                    .map(|h| {
                        let mut parts = h.split(':');
                        (
                            parts.next().unwrap_or("").trim().to_string(),
                            parts.next().unwrap_or("").trim().to_string(),
                        )
                    })
                    .collect();
                proxy_headers = Some(headers);
            }
            "proxy_uds" => proxy_uds = Some(value == "true"),
            _ => {}
        }
    }

    Ok((input, Domain {
        name,
        proxy_addr,
        proxy_tls,
        proxy_headers,
        proxy_uds,
        routes: vec![],
    }))
}

fn parse_route_section(input: &str) -> IResult<&str, Route> {
    let (input, _) = tag("[[[\"/path\"]]]")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, fields) = many0(parse_key_value)(input)?;

    let mut path = String::new();
    let mut proxy_addr = None;
    let mut proxy_tls = None;
    let mut proxy_headers = None;
    let mut proxy_uds = None;

    for (key, value) in fields {
        match key.as_str() {
            "path" => path = value,
            "proxy_addr" => proxy_addr = Some(value),
            "proxy_tls" => proxy_tls = Some(value == "true"),
            "proxy_headers" => {
                let headers: Vec<(String, String)> = value
                    .split(',')
                    .map(|h| {
                        let mut parts = h.split(':');
                        (
                            parts.next().unwrap_or("").trim().to_string(),
                            parts.next().unwrap_or("").trim().to_string(),
                        )
                    })
                    .collect();
                proxy_headers = Some(headers);
            }
            "proxy_uds" => proxy_uds = Some(value == "true"),
            _ => {}
        }
    }

    Ok((input, Route {
        path,
        proxy_addr,
        proxy_tls,
        proxy_headers,
        proxy_uds,
    }))
}

fn parse_dsl(input: &str) -> IResult<&str, Proxy> {
    let (input, mut proxy) = parse_global_section(input)?;

    let (input, domains) = many0(parse_domain_section)(input)?;
    proxy.domains = domains;

    Ok((input, proxy))
}

fn main() {
    let input = r#"
[proxy]
listener = "127.0.0.1:8080"
tls_certificate = "path/to/cert1"
tls_certificate_key = "path/to/key1"

[["domain"]]
name = "domain1"
proxy_addr = "/tmp/proxy.sock"
proxy_tls = true

[[["/path"]]]
path = "/nested"
proxy_addr = "/tmp/nested.sock"
proxy_tls = true
    "#;

    match parse_dsl(input) {
        Ok((_, proxy)) => println!("{:#?}", proxy),
        Err(e) => eprintln!("Error parsing DSL: {:?}", e),
    }
}
