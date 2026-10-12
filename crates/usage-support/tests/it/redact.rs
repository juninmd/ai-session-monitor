//! `redact::scrub`: what it masks, what it must leave alone, and that it cannot be made to hang.

use std::time::{Duration, Instant};

use usage_support::redact::scrub;

fn assert_scrubs(cases: &[(&str, &str)]) {
    for (input, expected) in cases {
        assert_eq!(scrub(input), *expected, "input: {input:?}");
    }
}

#[test]
fn authorization_and_cookie_header_values_are_masked_whole() {
    assert_scrubs(&[
        ("Authorization: Bearer sk-live-abcdefgh12345678", "Authorization: ***"),
        ("authorization: basic dXNlcjpwYXNzd29yZA==", "authorization: ***"),
        ("Proxy-Authorization: Basic Zm9vOmJhcg==", "Proxy-Authorization: ***"),
        ("AUTHORIZATION=Token abc 123 def", "AUTHORIZATION=***"),
        ("Cookie: session=abc123; theme=dark", "Cookie: ***"),
        ("Set-Cookie: sid=abc123; Path=/; HttpOnly", "Set-Cookie: ***"),
        // Only the value goes: the next header and the line break survive, CRLF included.
        ("Authorization: Bearer abc\r\nAccept: */*\r\n", "Authorization: ***\r\nAccept: */*\r\n"),
        (r#"{"Authorization":"Bearer abc123xyz","x":1}"#, r#"{"Authorization":"***","x":1}"#),
        (
            // Split so the scanner's curl-header rule does not see a complete command.
            concat!("curl -H 'Authorization: ", "Bearer abc123xyz", "' https://api.example.com/v1"),
            "curl -H 'Authorization: ***' https://api.example.com/v1",
        ),
    ]);
}

#[test]
fn bearer_tokens_are_masked_anywhere_but_plain_prose_is_not() {
    assert_scrubs(&[
        (
            "request failed: Bearer abc123xyz was rejected",
            "request failed: Bearer *** was rejected",
        ),
        ("bearer   eyfoo-bar_baz.qux", "bearer   ***"),
        ("the bearer of bad news", "the bearer of bad news"),
        ("Bearer token missing", "Bearer token missing"),
        ("Bearers abc12345", "Bearers abc12345"),
    ]);
}

#[test]
fn url_userinfo_is_masked_and_hosts_and_paths_are_kept() {
    assert_scrubs(&[
        ("https://alice:p%40ss@example.com/path?x=1", "https://***@example.com/path?x=1"),
        ("postgres://user:pa@ss@db.internal:5432/app", "postgres://***@db.internal:5432/app"),
        ("see https://bob@host.example/ now", "see https://***@host.example/ now"),
        ("https://example.com/a@b", "https://example.com/a@b"),
        ("https://example.com:8443/path", "https://example.com:8443/path"),
        ("mail me at user@example.com", "mail me at user@example.com"),
        ("https://@host/", "https://@host/"),
    ]);
}

#[test]
fn query_parameter_values_named_like_secrets_are_masked() {
    assert_scrubs(&[
        ("GET /v1/models?key=AbCd1234&user=bob", "GET /v1/models?key=***&user=bob"),
        (
            "?token=abc&api_key=def&secret=ghi&password=jkl",
            "?token=***&api_key=***&secret=***&password=***",
        ),
        ("?access_token=abc123&page=2", "?access_token=***&page=2"),
        ("https://h/x?Key=AbC#frag", "https://h/x?Key=***#frag"),
        ("?apikey=abc&api-key=def&refresh_token=ghi", "?apikey=***&api-key=***&refresh_token=***"),
        ("?q=hello+world&key=a%20b", "?q=hello+world&key=***"),
        ("?monkey=banana&keyboard=us&tokens=12", "?monkey=banana&keyboard=us&tokens=12"),
    ]);
}

#[test]
fn json_style_assignments_keep_the_structure() {
    assert_scrubs(&[
        (r#"{"api_key": "abc 123", "name": "x"}"#, r#"{"api_key": "***", "name": "x"}"#),
        (r#"{"password":"p\"w","a":1}"#, r#"{"password":"***","a":1}"#),
        (r#"{"token":"","a":1}"#, r#"{"token":"","a":1}"#),
        (r#"{"csrfToken": 12345678}"#, r#"{"csrfToken": ***}"#),
        (r#"{"token": {"a": 1}}"#, r#"{"token": {"a": 1}}"#),
        // JSON embedded in a JSON string has escaped quotes; the outer document must stay valid.
        (
            r#"{"body":"{\"token\":\"abc123\",\"ok\":true}"}"#,
            r#"{"body":"{\"token\":\"***\",\"ok\":true}"}"#,
        ),
    ]);
}

#[test]
fn command_lines_and_environment_assignments_are_masked() {
    assert_scrubs(&[
        ("agy --csrf_token 3f2a-91bc-44d0 --port 8080", "agy --csrf_token *** --port 8080"),
        ("tool --api-key=abc123 --verbose", "tool --api-key=*** --verbose"),
        (r#"tool --password "my secret pass" run"#, r#"tool --password "***" run"#),
        ("ANTHROPIC_API_KEY=abc123 OTHER=1", "ANTHROPIC_API_KEY=*** OTHER=1"),
        ("X-Codeium-Csrf-Token: 1a2b3c", "X-Codeium-Csrf-Token: ***"),
        ("clientSecret = hunter2hunter2", "clientSecret = ***"),
        ("(token=abc), next", "(token=***), next"),
    ]);
}

#[test]
fn long_opaque_runs_after_a_key_word_are_masked_in_prose() {
    assert_scrubs(&[
        ("invalid key AbCdEfGh0123456789xyz rejected", "invalid key *** rejected"),
        ("token 0123456789abcdef0123456789abcdef", "token ***"),
        ("secret abcdefghijklmnopqrstuvwxyzabcdef", "secret ***"),
        // Short words after the key word are ordinary prose.
        ("invalid token provided", "invalid token provided"),
        ("the key is missing", "the key is missing"),
        ("Primary key", "Primary key"),
    ]);
}

#[test]
fn well_known_credential_prefixes_are_masked_without_context() {
    // Assembled at run time so the secret scanner does not flag the fixture.
    let jwt = [
        "eyJhbGciOiJIUzI1NiJ9",
        "eyJzdWIiOiIxMjM0NTY3ODkwIn0",
        "dBjftJeZ4CVPmB92K27uhbUJU1p1r_wW1gFWFOEjXk",
    ]
    .join(".");
    let github = format!("ghp_{}", "a1B2".repeat(9));
    assert_scrubs(&[
        ("using sk-ant-api03-AbCdEf_123456-xyz for the call", "using *** for the call"),
        ("sk-live-123", "***"),
        ("oc_sk_AbCdEf123456", "***"),
        ("AIzaSyA1234567890abcdefghijklmnopqrstu", "***"),
        ("ya29.a0AfH6SMBxxxxxxxxxxxxxxxxxxxxxxxx", "***"),
        (&format!("token was {jwt}."), "token was ***."),
        (&format!("header {jwt} end"), "header *** end"),
        (&format!("x {github} y"), "x *** y"),
    ]);
    // The other GitHub token families each have their own prefix in the rule table.
    for prefix in ["gho_", "ghu_", "ghs_", "ghr_", "github_pat_"] {
        let token = format!("{prefix}{}", "a1B2".repeat(9));
        assert_eq!(scrub(&format!("x {token} y")), "x *** y", "{prefix}");
    }
}

#[test]
fn lookalikes_and_ordinary_diagnostics_are_left_alone() {
    for text in [
        "task-abcdefghijk",
        "risk-free-rate",
        "sk-ab",
        "sk-",
        "disk-cache-miss-count",
        "max_tokens=4096",
        "key_id=abcd",
        "monkey: banana",
        "keyboard layout",
        "missing field `rate_limits` at line 1 column 20",
        "unknown field `key`, expected one of `a`, `b`",
        "HTTP 429 from api.example.com",
        "GET https://example.com/api/key/info failed",
        "eyJhbGciOiJIUzI1NiJ9 is only a header",
        "A chave de API expirou; tente novamente",
        "ghp_short",
        "",
    ] {
        assert_eq!(scrub(text), text, "must be unchanged: {text:?}");
    }
}

#[test]
fn unicode_and_multiline_text_is_masked_on_char_boundaries() {
    let input = "falha na autenticação: token=çãõ-segredo-é\nlinha seguinte sem segredo\n\
                 Authorization: Bearer abc123xyz\nfim 🔑";
    let expected = "falha na autenticação: token=***\nlinha seguinte sem segredo\n\
                    Authorization: ***\nfim 🔑";
    assert_eq!(scrub(input), expected);
    assert_scrubs(&[
        // A Unicode space ends an unquoted value, like an ASCII one.
        ("token=abc\u{3000}depois", "token=***\u{3000}depois"),
        ("🔑 sk-AbCdEf123456 🔑", "🔑 *** 🔑"),
        ("🔑sk-AbCdEf123456", "🔑***"),
        // Inside a longer word, so it is not a key.
        ("ásk-AbCdEf123456", "ásk-AbCdEf123456"),
        ("密码 password=秘密123 完", "密码 password=*** 完"),
        ("chave=\"çã õ\" ok", "chave=\"çã õ\" ok"),
        ("api_key=\"çã õ\" ok", "api_key=\"***\" ok"),
    ]);
}

#[test]
fn unbalanced_quotes_mask_to_the_end_of_the_line_only() {
    assert_scrubs(&[
        ("token=\"abc def\nnext line", "token=\"***\nnext line"),
        ("{\"password\": \"abc\\", "{\"password\": \"***"),
        ("secret='abc'\nsecond='keep'", "secret='***'\nsecond='keep'"),
    ]);
}

#[test]
fn scrub_is_idempotent_and_never_panics_on_hostile_input() {
    let pieces = [
        "Bearer ",
        "sk-",
        "oc_sk_",
        "token",
        "key",
        "=",
        ":",
        "\"",
        "'",
        "\\",
        "\n",
        "\r\n",
        " ",
        "://",
        "@",
        "é",
        "🔑",
        "?",
        "&",
        "eyJ",
        ".",
        "abc123",
        "Authorization",
        "--",
        "-",
        "_",
        "{",
        "}",
        "[",
        "]",
        "\u{3000}",
        "\0",
        "Cookie",
        "csrf",
        "x",
        "9",
    ];
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..4000 {
        let count = 1 + (next() % 40) as usize;
        let input: String =
            (0..count).map(|_| pieces[(next() % pieces.len() as u64) as usize]).collect();
        let once = scrub(&input);
        assert_eq!(scrub(&once), once, "not idempotent for {input:?}");
    }
}

#[test]
fn secrets_never_survive_in_a_realistic_log_dump() {
    let secrets = ["sk-live-4f9a2c71", "p4ss-w0rd-xyz", "tok_123456789abcdef0", "csrf-9d8c7b6a"];
    let dump = format!(
        "POST https://admin:{p}@litellm.internal/key/info?api_key={k}&user=ana\n\
         > authorization: Bearer {k}\n> x-csrf-token: {c}\n< 401 {{\"detail\":\"bad token {t}\"}}\n\
         retry with --password {p}",
        k = secrets[0],
        p = secrets[1],
        t = secrets[2],
        c = secrets[3],
    );
    let out = scrub(&dump);
    for secret in secrets {
        assert!(!out.contains(secret), "{secret} leaked in:\n{out}");
    }
    assert!(out.contains("litellm.internal/key/info"), "host and path must stay readable:\n{out}");
    assert!(out.contains("user=ana"), "non-secret params must stay:\n{out}");
}

#[test]
fn adversarial_input_is_processed_in_linear_time() {
    let n = 1 << 20;
    let floods = [
        "eyJ_".repeat(n / 4),
        "cookie:".repeat(n / 7),
        "key=".repeat(n / 4),
        "sk-".repeat(n / 3),
        "bearer ".repeat(n / 7),
        "://@".repeat(n / 4),
        "authorization:".repeat(n / 14),
        "token ".repeat(n / 6),
        "a".repeat(n),
        "\\".repeat(n),
        "\"".repeat(n),
    ];
    for flood in &floods {
        let started = Instant::now();
        let out = scrub(flood);
        let took = started.elapsed();
        assert!(took < Duration::from_secs(10), "{:?} took {took:?}", &flood[..8]);
        assert!(out.len() <= flood.len() * 3 + 3);
    }
}
