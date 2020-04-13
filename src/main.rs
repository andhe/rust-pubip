fn main() {
    let resp = ureq::get("https://ifconfig.co/ip").call();

    // .ok() tells if response is 200-299.
    if resp.ok() {
        let ip = resp.into_string().unwrap();
        println!("Response: {}", ip.trim());
    } else {
        println!("Failed to get ip.");
    }
}
