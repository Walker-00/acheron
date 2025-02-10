<div align="center">

<img src="https://github.com/user-attachments/assets/f77ae492-071e-460d-9c43-d28305d12da6" width="350" height="350" alt="Acheron" />

</div>

# Acheron Configuration Language

Acheron is a configuration language designed for Charon, a high-performance proxy server written in Rust. It provides an intuitive and human-readable format, making it easy to configure complex setups with minimal effort. Whether you're managing proxy configurations, load balancing, or routing rules, Acheron simplifies the process with clarity and precision.

## Key Features

- **Readability**: Acheron prioritizes human-friendly syntax, ensuring that configurations are easy to write, read, and maintain.
- **Flexibility**: Supports both proxy server configurations and load balancer setups.
- **Rust-based**: Built in Rust, ensuring safety, speed, and reliability.
- **Unified Structure**: Combines proxy and load balancer settings into a single cohesive format.
- **Open Source**: Licensed under the [WTFPL](http://www.wtfpl.net/), giving you total freedom to use and modify the project.

## Configuration Example

For more example look in [example](https://github.com/Walker-00/acheron/tree/rust/examples)

Here’s a sample Acheron configuration file to showcase its simplicity and power:

```acheron
prometheus_addr = "127.0.0.1:9090"

[proxy]
listener = "0.0.0.0:8080"
tls_certificate = "cert.pem"
tls_certificate_key = "key.pem"

[[domain=example.com]]
proxy_addr = "192.168.1.1"
proxy_tls = true
proxy_headers = [["Authorization": "Bearer token"]]

[[[route=/api]]]
proxy_addr = "192.168.1.2"
proxy_tls = false

proxy_uds = true

[[domain="example.net"]]
proxy_addr = "127.0.0.1:9799"

[proxy]
listener = "0.0.0.0:443"

[[domain="example.com"]]
proxy_addr = "192.168.1.1"
proxy_tls = true
proxy_headers = [["Authorization": "Bearer token"]]

[[[route="/bpi"]]]
proxy_addr = "192.168.1.2"
proxy_tls = false

proxy_uds = true

[[domain=example.net]]
proxy_addr = "192.168.1.3"

[load_balancer]
listener = "0.0.0.0:9091"
upstreams = ["192.168.1.4:8000", "192.168.1.5:8000"]
tls_certificate = "lb_cert.pem"
tls_certificate_key = "lb_key.pem"
health_check = true
health_check_frequency = 30
parallel_health_check = true

[[domain=load-balancer.com]]
load_balancer_tls = true
load_balancer_headers = [["X-Custom-Header: CustomValue"], ["Accept: */*"]]
```

## Format Highlights

- **Sections**: Denoted by headers like `[proxy]` and `[load_balancer]`.
- **Key-Value Pairs**: Use simple `key = value` syntax for configuration.
- **Nested Configurations**: Define domain-specific rules using double brackets `[["domain"]]`.
- **Headers**: Support customizable headers for both proxies and load balancers.

## Why Acheron?

- **Streamlined Configurations**: Simplifies the process of managing proxy and load balancer setups.
- **Human-Centric Design**: Focuses on reducing cognitive load with readable and logical syntax.
- **Powerful Yet Minimal**: Provides advanced capabilities without unnecessary complexity.

## Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/your-repo/acheron.git
   ```

2. Install dependencies:
   ```bash
   cargo build
   ```

3. Use Acheron to configure Charon:
   ```bash
   ./charon --config path/to/config.chr
   ```

## Community and Contributions

We welcome contributions of all kinds. Feel free to fork the repository, submit pull requests, or open issues to share ideas and suggestions. Let’s build a better way to manage proxies together!

## License

This project is licensed under the [WTFPL](http://www.wtfpl.net/). Do whatever you want with it. Seriously.

## Contact

For any questions or feedback, reach out at [rissk_it@proton.me](mailto:rissk_it@proton.me) or open an issue in the GitHub repository.

---

### Acknowledgments

- Built with ☕ using Rust.
- Inspired by the need for clear and effective configuration in modern infrastructure.

---

Enjoy using Acheron! 🎉
