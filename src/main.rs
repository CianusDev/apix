fn main() {
    let args: Vec<String> = std::env::args().collect();
    let method = &args[1];
    let url = &args[2];

    if method == "GET" {
        trpl::block_on(get(url));
    } else {
        println!("Unsupported method: {}", method);
    }
}

async fn get(url: &str) {
    println!("GET request to {}", url);
    let req = reqwest::get(url).await.expect("Failed to send GET request");
    let status_code = req.status().as_u16();
    let headers = req.headers().clone();
    let body = req
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse JSON response");

    println!(
        "status: {:#?}\nheaders: {:?}\n body: {:#?}",
        status_code, headers, body
    );
}
