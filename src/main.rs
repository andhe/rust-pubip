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

    # DNS

    dig +short myip.opendns.com @resolver1.opendns.com
    dig +short ANY whoami.akamai.net @ns1-1.akamaitech.net
    dig +short ANY o-o.myaddr.l.google.com @ns1.google.com

*/

fn print_as_env(pubip: &PubIPResult) {
    println!("PUBLIC_IP_PROVIDER=\"{}\"", pubip.provider);
    println!("PUBLIC_IP={}", pubip.ip);
}

fn main() {
    if let Ok(pubip) = https_ipv4_icanhazip() {
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
