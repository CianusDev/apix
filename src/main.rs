mod app;
mod config;
mod errors;
mod http;
mod models;

use app::App;
use errors::ApixError;
use models::Method;
use models::Request;

#[tokio::main]
async fn main() -> errors::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: apix <METHOD> <URL> [-H \"key: value\"]... [-d '{{\"json\"}}']");
        eprintln!("Example: apix GET https://api.example.com/users");
        eprintln!("Example: apix POST https://api.example.com/users -H \"Authorization: Bearer token\" -d '{{\"name\": \"John\"}}'");
        std::process::exit(1);
    }

    let method = Method::from_str(&args[1])?;
    let url = args[2].clone();
    let (headers, body) = parse_options(&args[3..])?;

    let app = App::new()?;
    app.initialize()?;

    let mut request = Request::new(method, url.clone());
    request.headers = headers;
    request.body = body;

    // Ajouter Content-Type: application/json si body present et pas de Content-Type
    if request.body.is_some() {
        let has_content_type = request
            .headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
        if !has_content_type {
            request
                .headers
                .push(("Content-Type".to_string(), "application/json".to_string()));
        }
    }

    println!("{} request to {}", method, url);
    let response = app.http_client.execute(&request).await?;
    println!("{}", response.format());

    Ok(())
}

fn parse_options(args: &[String]) -> errors::Result<(Vec<(String, String)>, Option<String>)> {
    let mut headers = Vec::new();
    let mut body = None;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-H" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| ApixError::MissingArgument("-H requires a value".to_string()))?;
                let (key, val) = parse_header(value)?;
                headers.push((key, val));
                i += 2;
            }
            "-d" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| ApixError::MissingArgument("-d requires a value".to_string()))?;
                body = Some(value.clone());
                i += 2;
            }
            other => {
                return Err(ApixError::MissingArgument(format!(
                    "Unknown option: {}",
                    other
                )));
            }
        }
    }

    Ok((headers, body))
}

fn parse_header(header: &str) -> errors::Result<(String, String)> {
    let pos = header
        .find(':')
        .ok_or_else(|| ApixError::MissingArgument(format!("Invalid header format: {}", header)))?;
    let key = header[..pos].trim().to_string();
    let value = header[pos + 1..].trim().to_string();
    Ok((key, value))
}
