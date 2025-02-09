use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
struct Config {
    super_config: String,
    proxy: ProxyConfig,
    load_balancer: LoadBalancerConfig,
}

#[derive(Debug, Deserialize)]
struct ProxyConfig {
    default_configs: String,
    mores: String,
    #[serde(flatten)]
    domains: HashMap<String, DomainConfig>,
}

#[derive(Debug, Deserialize)]
struct LoadBalancerConfig {
    default_configs: String,
    mores: String,
    #[serde(flatten)]
    domains: HashMap<String, DomainConfig>,
}

#[derive(Debug, Deserialize)]
struct DomainConfig {
    some_config: String,
    more: String,
    #[serde(flatten)]
    routes: HashMap<String, RouteConfig>,
}

#[derive(Debug, Deserialize)]
struct RouteConfig {
    some_config: String,
    more: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the configuration file
    let config_str = fs::read_to_string("config.txt")?;

    // Parse the configuration using Serde
    let config: Config = toml::from_str(&config_str)?;

    // Print the parsed configuration
    println!("{:#?}", config);

    Ok(())
}
