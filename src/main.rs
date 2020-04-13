struct PubIPResult {
    provider: &'static str,
    ip: String,
}

fn https_ifconfig_co() -> Result<PubIPResult, &'static str> {
    http_get("https://ifconfig.co/ip", "ifconfig.co (https)")
}

fn http_get(url: &str, provider: &'static str) -> Result<PubIPResult, &'static str> {
    let resp = ureq::get(url)
        .timeout_connect(1_000)
        .timeout_read(1_000)
        .timeout_write(1_000)
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
fn main() {
    if let Ok(pubip) = https_ifconfig_co() {
        println!("PUBLIC_IP_PROVIDER=\"{}\"", pubip.provider);
        println!("PUBLIC_IP={}", pubip.ip);
    } else {
        eprintln!("Failed to get ip.");
        std::process::exit(1);
    }
}
