/*
 * SPDX-FileCopyrightText: 2020 Andreas Henriksson <andreas@fatal.se>
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

// traits must be in scope, so import them.
use dnssector::rr_iterator::RdataIterable;
use dnssector::rr_iterator::DNSIterable;
use std::net::ToSocketAddrs;

struct PubIPResult {
    provider: &'static str,
    ip: String,
}

// supports ipv6
fn https_ifconfig_co() -> Result<PubIPResult, &'static str> {
    http_get("https://ifconfig.co/ip", "ifconfig.co (https)")
}

// ipv4-only?
fn https_ifconfig_me() -> Result<PubIPResult, &'static str> {
    http_get("https://ifconfig.me/ip", "ifconfig.me (https)")
}

fn https_icanhazip() -> Result<PubIPResult, &'static str> {
    http_get("https://icanhazip.com/", "icanhazip (https)")
}

fn https_ipv4_icanhazip() -> Result<PubIPResult, &'static str> {
    http_get("https://ipv4.icanhazip.com/", "icanhazip (ipv4, https)")
}

fn http_get(url: &str, provider: &'static str) -> Result<PubIPResult, &'static str> {
    let resp = ureq::get(url)
        .timeout_connect(10_000)
        .timeout_read(10_000)
        .timeout_write(10_000)
        .call();

    // .ok() tells if response is 200-299.
    if resp.ok() {
        let ip = resp.into_string().unwrap();

        Ok(PubIPResult {
            provider: provider,
            ip: String::from(ip.trim()),
        })
    } else {
        Err("Request to ifconfig.co failed.")
    }
}

/* More potential providers:

    # HTTP

    http://ipecho.net/plain
    http://indent.me
    http://bot.whatismyipaddress.com
    https://diagnostic.opendns.com/myip
    http://checkip.amazonaws.com
    http://whatismyip.akamai.com
    https://myip.biturl.top/

    # DNS

    dig +short myip.opendns.com @resolver1.opendns.com
    dig +short ANY whoami.akamai.net @ns1-1.akamaitech.net
    dig +short ANY o-o.myaddr.l.google.com @ns1.google.com

*/


fn dns_google() -> Result<PubIPResult, std::io::Error> {
    // NOTE: problem 1: dnssector::constants::Type::from_string
    //                  can't translate "ANY" to Type::ANY.
    //                  Workaround: change query_type argument
    //                  to directly take dnssector::constants::Type
    //                  and pass in dnssector::constants::Type::ANY
    //                  to avoid needing Type::from_string.
    //       problem 2: the reply is a TXT record which dnssector
    //                  has no decode method for (similar to rr_ip).
    //                  Workaround: none known.
    dns_lookup("ns1.google.com:53", "o-o.myaddr.l.google.com", "ANY", "google (DNS)")
}

fn dns_akamai() -> Result<PubIPResult, std::io::Error> {
    dns_lookup("ns1-1.akamaitech.net:53",  "whoami.akamai.net", "A", "akamai (DNS)")
}

fn dns_lookup(upstream_server_name: &str, lookup_name: &str, query_type: &str, provider: &'static str) -> Result<PubIPResult, std::io::Error> {
    let upstream_server_addr_vec: Vec<_> = upstream_server_name.to_socket_addrs().expect("Failed to parse upstream server name").collect();
    let upstream_server_addr: std::net::SocketAddr = upstream_server_addr_vec[0];

    let local_addr = match upstream_server_addr {
        std::net::SocketAddr::V4(_) => std::net::SocketAddr::new(std::net::IpAddr::from([0; 4]), 0),
        std::net::SocketAddr::V6(_) => std::net::SocketAddr::new(std::net::IpAddr::from([0; 16]), 0),
    };

    let parsed_query = dnssector::gen::query(
        lookup_name.as_bytes(),
        dnssector::constants::Type::from_string(query_type).unwrap(),
        dnssector::constants::Class::from_string("IN").unwrap(),
        );

    let mut parsed_response = {
        let query = parsed_query.unwrap().into_packet();
        let socket = std::net::UdpSocket::bind(local_addr).unwrap();
        let _ = socket.set_read_timeout(Some(std::time::Duration::new(5, 0)));
        socket.connect(upstream_server_addr)?;
        socket.send(&query)?;
        let mut response = vec![0; dnssector::constants::DNS_MAX_COMPRESSED_SIZE];
        let response_len = socket
            .recv(&mut response)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::WouldBlock, "Timeout"))?;
        response.truncate(response_len);
        dnssector::DNSSector::new(response)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?
    };

    let mut it = parsed_response.into_iter_answer();
    while let Some(item) = it {
        if let Ok(std::net::IpAddr::V4(addr)) = item.rr_ip() {
            return Ok(PubIPResult {
                provider: provider,
                ip: addr.to_string(),
            });
        } else if let Ok(std::net::IpAddr::V6(addr)) = item.rr_ip() {
            return Ok(PubIPResult {
                provider: provider,
                ip: addr.to_string(),
            });
        }
        it = item.next();
    }

    Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Failed to find public ip in dns lookup",
            ))
}


fn print_as_env(pubip: &PubIPResult) {
    println!("PUBLIC_IP_PROVIDER=\"{}\"", pubip.provider);
    println!("PUBLIC_IP={}", pubip.ip);
}

fn main() {
    if let Ok(pubip) = dns_akamai() {
        print_as_env(&pubip);
    } else if let Ok(pubip) = https_ipv4_icanhazip() {
        print_as_env(&pubip);
    } else if let Ok(pubip) = https_icanhazip() {
        print_as_env(&pubip);
    } else if let Ok(pubip) = https_ifconfig_me() {
        print_as_env(&pubip);
    } else if let Ok(pubip) = https_ifconfig_co() {
        print_as_env(&pubip);
    } else {
        eprintln!("Failed to get ip.");
        std::process::exit(1);
    }
}
