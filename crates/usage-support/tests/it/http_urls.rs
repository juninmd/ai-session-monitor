//! The URL policy and the URL builders: pure functions, no network.

use usage_model::ErrorCode;
use usage_support::{append_path, append_query, check_url};

#[test]
fn check_url_accepts_https_and_keeps_prefix_query_and_port() {
    for url in [
        "https://api.example.com",
        "https://api.example.com/",
        "https://litellm.corp.example:4443/gateway/litellm",
        "HTTPS://API.EXAMPLE.COM/v1?x=1",
        "https://[2001:db8::1]:8443/",
        "https://münchen.example/",
        "https://127.0.0.1:8443/",
    ] {
        assert!(check_url(url, false).is_ok(), "{url}");
    }
}

#[test]
fn check_url_rejects_everything_outside_the_policy() {
    let long = format!("https://example.com/{}", "a".repeat(5000));
    for url in [
        "",
        " ",
        "example.com",
        "//example.com",
        "http://example.com/",
        "ftp://example.com/",
        "file:///c:/secrets",
        "javascript:alert(1)",
        "ws://example.com/",
        "https://user@example.com/",
        "https://user:pass@example.com/",
        "https://:pass@example.com/",
        "https://example.com/#frag",
        "https://example.com/#",
        "https://",
        "https://exa mple.com/",
        " https://example.com/",
        "https://example.com/ ",
        "https://example.com/\n",
        "https://exa\tmple.com/",
        "https://example.com/\u{7}",
        "https://example.com/\u{85}",
        "https://example.com/\0",
        long.as_str(),
    ] {
        let error = check_url(url, true).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidUrl, "{url:?}");
        assert!(!error.retryable);
        assert!(
            error.params.keys().all(|k| k == "host"),
            "only the host may be reported: {:?}",
            error.params
        );
    }
}

#[test]
fn check_url_allows_http_only_to_loopback_and_only_when_enabled() {
    for url in [
        "http://127.0.0.1:8080/",
        "http://127.255.255.254/",
        "http://localhost/",
        "http://LOCALHOST:3000/path",
        "http://[::1]:8080/",
        "http://[::ffff:127.0.0.1]/",
        "http://127.1/",
    ] {
        assert!(check_url(url, true).is_ok(), "{url}");
        assert_eq!(check_url(url, false).unwrap_err().code, ErrorCode::InvalidUrl, "{url}");
    }
    for url in [
        "http://example.com/",
        "http://localhost.example.com/",
        "http://127.0.0.1.example.com/",
        // Only the bare name is pinned to loopback; a `*.localhost` name may resolve anywhere.
        "http://api.localhost/",
        "http://api.localhost:8080/",
        "http://192.168.1.10/",
        "http://10.0.0.1/",
        "http://0.0.0.0/",
        "http://[::]/",
        "http://[2001:db8::1]/",
        "http://128.0.0.1/",
        "http://user@127.0.0.1/",
    ] {
        assert_eq!(check_url(url, true).unwrap_err().code, ErrorCode::InvalidUrl, "{url}");
    }
}

#[test]
fn a_refused_url_is_reported_by_host_only() {
    let error = check_url("http://example.com/secret/path?token=sk-live-abc", true).unwrap_err();
    assert_eq!(error.params.get("host").map(String::as_str), Some("example.com"));
    assert!(!format!("{error:?}").contains("sk-live"));

    let error = check_url("https://user:hunter2@example.com/x", true).unwrap_err();
    assert!(!format!("{error:?}").contains("hunter2"));

    // An unparsable URL has no trustworthy host, so nothing at all is reported.
    let error = check_url("https://exa mple.com/?token=sk-live-abc", true).unwrap_err();
    assert!(error.params.is_empty());
}

#[test]
fn append_query_percent_encodes_pairs_and_keeps_what_is_there() {
    let url = append_query(
        "https://h.example/prefix/info?keep=1",
        &[("user id", "ana & bob"), ("q", "日本=語"), ("plus", "a+b"), ("empty", "")],
    )
    .unwrap();
    assert_eq!(
        url,
        "https://h.example/prefix/info?keep=1&user%20id=ana%20%26%20bob\
         &q=%E6%97%A5%E6%9C%AC%3D%E8%AA%9E&plus=a%2Bb&empty="
    );
    assert_eq!(
        append_query("https://h.example/p", &[("a", "1")]).unwrap(),
        "https://h.example/p?a=1"
    );
    assert_eq!(append_query("https://h.example/p", &[]).unwrap(), "https://h.example/p");
    assert_eq!(
        append_query("https://h.example/p#frag", &[("a", "1")]).unwrap(),
        "https://h.example/p?a=1#frag"
    );
    assert_eq!(append_query("not a url", &[("a", "1")]).unwrap_err().code, ErrorCode::InvalidUrl);
}

#[test]
fn append_path_keeps_the_prefix_encodes_segments_and_refuses_traversal() {
    assert_eq!(
        append_path("https://h.example/litellm/", &["team", "a b/c", "é", "members", "me"])
            .unwrap(),
        "https://h.example/litellm/team/a%20b%2Fc/%C3%A9/members/me"
    );
    assert_eq!(
        append_path("https://h.example", &["key", "info"]).unwrap(),
        "https://h.example/key/info"
    );
    assert_eq!(
        append_path("https://h.example/p?x=1", &["a"]).unwrap(),
        "https://h.example/p/a?x=1"
    );
    for bad in ["", ".", ".."] {
        let error = append_path("https://h.example/p", &["ok", bad]).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidUrl, "{bad:?}");
    }
    // Dots inside a longer segment are ordinary, and an encoded dot-dot stays encoded.
    assert_eq!(
        append_path("https://h/p", &["a..b", "%2e%2e"]).unwrap(),
        "https://h/p/a..b/%252e%252e"
    );
}
