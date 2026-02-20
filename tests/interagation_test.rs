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
fn incomplete_args_shows_usage() {
    // Un seul arg (methode sans URL) doit afficher l'usage
    let output = run_apix(&["GET"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("Usage:"));
}

#[test]
fn invalid_url_returns_error() {
    let output = run_apix(&["GET", "not-a-url"]);

    assert!(!output.status.success());
}

// --- POST ---

#[test]
fn post_request_with_body() {
    let output = run_apix(&[
        "POST",
        "https://jsonplaceholder.typicode.com/posts",
        "-d",
        r#"{"title": "test", "body": "hello", "userId": 1}"#,
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("201") || stdout.contains("200"));
    assert!(stdout.contains("POST request to"));
}

#[test]
fn post_request_with_headers_and_body() {
    let output = run_apix(&[
        "POST",
        "https://jsonplaceholder.typicode.com/posts",
        "-H",
        "Accept: application/json",
        "-d",
        r#"{"title": "test", "body": "hello", "userId": 1}"#,
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("test"));
}

// --- PUT ---

#[test]
fn put_request_with_body() {
    let output = run_apix(&[
        "PUT",
        "https://jsonplaceholder.typicode.com/posts/1",
        "-d",
        r#"{"id": 1, "title": "updated", "body": "new body", "userId": 1}"#,
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("200"));
    assert!(stdout.contains("PUT request to"));
}

// --- PATCH ---

#[test]
fn patch_request_with_body() {
    let output = run_apix(&[
        "PATCH",
        "https://jsonplaceholder.typicode.com/posts/1",
        "-d",
        r#"{"title": "patched"}"#,
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("200"));
    assert!(stdout.contains("PATCH request to"));
}

// --- DELETE ---

#[test]
fn delete_request() {
    let output = run_apix(&["DELETE", "https://jsonplaceholder.typicode.com/posts/1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("200"));
    assert!(stdout.contains("DELETE request to"));
}

// --- Options CLI ---

#[test]
fn multiple_headers() {
    let output = run_apix(&[
        "GET",
        "https://jsonplaceholder.typicode.com/posts/1",
        "-H",
        "Accept: application/json",
        "-H",
        "X-Custom: test",
    ]);

    assert!(output.status.success());
}

#[test]
fn unknown_option_returns_error() {
    let output = run_apix(&["GET", "https://example.com", "--unknown"]);

    assert!(!output.status.success());
}

#[test]
fn header_without_value_returns_error() {
    let output = run_apix(&["GET", "https://example.com", "-H"]);

    assert!(!output.status.success());
}

// --- Phase 3 : JSON & Headers ---

#[test]
fn json_response_is_pretty_printed() {
    let output = run_apix(&["GET", "https://jsonplaceholder.typicode.com/posts/1"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Le JSON pretty-printed contient des sauts de ligne et de l'indentation
    assert!(stdout.contains("\"userId\""));
    assert!(stdout.contains("\"id\""));
    // Verifie que c'est bien indente (pretty print)
    assert!(stdout.contains("  "));
}

#[test]
fn non_json_response_does_not_crash() {
    // Requete vers une page HTML (pas du JSON)
    let output = run_apix(&["GET", "https://example.com"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("200"));
}

#[test]
fn custom_content_type_header_is_preserved() {
    let output = run_apix(&[
        "POST",
        "https://jsonplaceholder.typicode.com/posts",
        "-H",
        "Content-Type: text/plain",
        "-d",
        "hello world",
    ]);

    // Doit reussir sans ecraser le Content-Type custom
    assert!(output.status.success());
}
