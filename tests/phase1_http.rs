use std::process::Command;

fn run_apix(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--")
        .args(args)
        .output()
        .expect("Failed to execute apix")
}

#[test]
fn get_request_returns_success() {
    let output = run_apix(&["GET", "https://jsonplaceholder.typicode.com/posts/1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("200"));
    assert!(stdout.contains("GET request to"));
}

#[test]
fn get_request_returns_json_body() {
    let output = run_apix(&["GET", "https://jsonplaceholder.typicode.com/posts/1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("userId"));
    assert!(stdout.contains("title"));
}

#[test]
fn invalid_method_returns_error() {
    let output = run_apix(&["INVALID", "https://example.com"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("InvalidMethod"));
}

#[test]
fn no_args_shows_usage() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .output()
        .expect("Failed to execute apix");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("Usage: apix <METHOD> <URL>"));
}

#[test]
fn invalid_url_returns_error() {
    let output = run_apix(&["GET", "not-a-url"]);

    assert!(!output.status.success());
}
