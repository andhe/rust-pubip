fn main() {
    let resp = ureq::get("https://ifconfig.co/ip")
        .timeout_connect(1_000)
        .timeout_read(1_000)
        .timeout_write(1_000)
        .call();

    // .ok() tells if response is 200-299.
    if resp.ok() {
        let ip = resp.into_string().unwrap();
        println!("Response: {}", ip.trim());
    } else {
        println!("Failed to get ip.");
    }
}
