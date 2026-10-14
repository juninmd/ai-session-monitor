use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use usage_model::ErrorCode;

use super::*;
use crate::ClientOptions;

fn client() -> SafeClient {
    SafeClient::new(ClientOptions::default()).unwrap()
}

async fn server(replies: Vec<String>) -> (Url, tokio::task::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = Url::parse(&format!("http://{}/asset", listener.local_addr().unwrap())).unwrap();
    let task = tokio::spawn(async move {
        let mut requests = Vec::new();
        for reply in replies {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buffer = [0; 8192];
            let count = socket.read(&mut buffer).await.unwrap();
            requests.push(String::from_utf8_lossy(&buffer[..count]).to_string());
            socket.write_all(reply.as_bytes()).await.unwrap();
        }
        requests
    });
    (url, task)
}

#[test]
fn initial_urls_are_confined_to_the_public_repository() {
    for url in [
        "https://github.com/juninmd/ai-session-monitor/releases/latest/download/latest.json",
        "https://github.com/juninmd/ai-session-monitor/releases/download/v1.2.3/app.exe",
        "https://api.github.com/repos/juninmd/ai-session-monitor/releases/assets/123",
    ] {
        assert!(release_url(url).is_ok());
    }
    for url in [
        "http://github.com/juninmd/ai-session-monitor/releases/download/v1/a.exe",
        "https://github.com/other/repository/releases/download/v1/app.exe",
        "https://github.com/juninmd/ai-session-monitor/releases/download/v1/a.exe?token=secret",
        "https://user:secret@github.com/juninmd/ai-session-monitor/releases/download/v1/a.exe",
        "https://api.github.com/repos/juninmd/ai-session-monitor/releases/assets/123/evil",
        "https://github.com.evil.invalid/juninmd/ai-session-monitor/releases/download/v1/app.exe",
        "https://release-assets.githubusercontent.com/asset",
    ] {
        let error = release_url(url).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidUrl);
        assert!(!serde_json::to_string(&error).unwrap().contains("secret"));
    }
    assert!(redirect_allowed(
        &Url::parse("https://release-assets.githubusercontent.com/a?token=secret").unwrap()
    ));
    for value in [
        "https://evil.invalid/",
        "http://release-assets.githubusercontent.com/a",
        "https://release-assets.githubusercontent.com:444/a",
    ] {
        assert!(!redirect_allowed(&Url::parse(value).unwrap()));
    }
}

#[tokio::test]
async fn redirects_are_explicit_and_progress_is_cumulative() {
    let (url, task) = server(vec![
        "HTTP/1.1 302 Found\r\nLocation: /installer\r\nContent-Length: 0\r\n\r\n".into(),
        "HTTP/1.1 200 OK\r\nContent-Length: 9\r\n\r\ninstaller".into(),
    ])
    .await;
    let mut progress = Vec::new();
    let bytes = client()
        .release_download(
            url,
            10,
            |next| next.host_str() == Some("127.0.0.1"),
            |read, total| progress.push((read, total)),
        )
        .await
        .unwrap();
    assert_eq!(bytes, b"installer");
    assert_eq!(progress.first(), Some(&(0, Some(9))));
    assert_eq!(progress.last(), Some(&(9, Some(9))));
    let requests = task.await.unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|request| !request.to_ascii_lowercase().contains("authorization")));
}

#[tokio::test]
async fn disallowed_redirects_do_not_reveal_the_signed_url() {
    let (url, task) = server(vec![
        "HTTP/1.1 302 Found\r\nLocation: https://evil.invalid/?token=secret\r\nContent-Length: 0\r\n\r\n".into(),
    ]).await;
    let error = client().release_download(url, 10, redirect_allowed, |_, _| {}).await.unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidUrl);
    assert!(!serde_json::to_string(&error).unwrap().contains("secret"));
    task.await.unwrap();
}

#[tokio::test]
async fn both_declared_and_streamed_body_sizes_are_bounded() {
    for reply in [
        "HTTP/1.1 200 OK\r\nContent-Length: 9\r\n\r\ninstaller",
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n9\r\ninstaller\r\n0\r\n\r\n",
    ] {
        let (url, task) = server(vec![reply.into()]).await;
        let error = client().release_download(url, 8, |_| false, |_, _| {}).await.unwrap_err();
        assert_eq!(error.code, ErrorCode::ResponseTooLarge);
        task.await.unwrap();
    }
}

#[tokio::test]
async fn incomplete_and_partial_responses_cannot_be_installers() {
    for reply in [
        "HTTP/1.1 206 Partial Content\r\nContent-Length: 2\r\n\r\nxx",
        "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n",
        "HTTP/1.1 200 OK\r\nContent-Length: 9\r\n\r\nxx",
    ] {
        let (url, task) = server(vec![reply.into()]).await;
        assert!(client().release_download(url, 10, |_| false, |_, _| {}).await.is_err());
        task.await.unwrap();
    }
}

#[tokio::test]
async fn a_stalled_response_has_an_overall_deadline() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = Url::parse(&format!("http://{}/", listener.local_addr().unwrap())).unwrap();
    let mut client = client();
    client.timeout = Duration::from_millis(50);
    let result = client.release_download(url, 10, |_| false, |_, _| {}).await;
    assert_eq!(result.unwrap_err().code, ErrorCode::Timeout);
}
