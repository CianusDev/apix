mod app;
mod config;
mod errors;
mod http;
mod models;

use app::App;
use models::Method;
use models::Request;

#[tokio::main]
async fn main() -> errors::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: apix <METHOD> <URL>");
        eprintln!("Example: apix GET https://api.example.com/users");
        std::process::exit(1);
    }

    let method = Method::from_str(&args[1])?;
    let url = args[2].clone();

    let app = App::new()?;
    app.initialize()?;

    let request = Request::new(method, url.clone());

    println!("{} request to {}", method, url);
    let response = app.http_client.execute(&request).await?;
    println!("{}", response.format());

    Ok(())
}
