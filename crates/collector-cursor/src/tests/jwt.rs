//! The JWT decoder and the cookie built from it.

use usage_model::HarnessView;

use super::{COOKIE, NOW, token};
use crate::jwt::{Unusable, cookie, decode};

/// base64url without padding: the test-side encoder, so claims can be varied freely.
fn encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n =
            chunk.iter().enumerate().fold(0u32, |n, (i, b)| n | (u32::from(*b) << (16 - 8 * i)));
        for i in 0..=chunk.len() {
            out.push(char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize]));
        }
    }
    out
}

fn jwt(claims: &str) -> String {
    format!("{}.{}.sig-fabricated", encode(br#"{"alg":"none"}"#), encode(claims.as_bytes()))
}

fn built(claims: &str) -> Result<String, Unusable> {
    cookie(&jwt(claims), NOW).map(|secret| secret.expose().to_owned())
}

#[test]
fn the_test_encoder_agrees_with_the_decoder_and_with_known_vectors() {
    assert_eq!(encode(b"Man"), "TWFu");
    assert_eq!(encode(b"Ma"), "TWE");
    assert_eq!(encode(b"M"), "TQ");
    for sample in [&b""[..], b"a", b"ab", b"abc", b"abcd", &[0xff, 0xfe, 0xfd, 0x00, 0x3e, 0x3f]] {
        assert_eq!(decode(&encode(sample)).as_deref(), Some(sample));
    }
}

#[test]
fn the_decoder_takes_padding_and_refuses_anything_outside_the_alphabet() {
    assert_eq!(decode("TWE=").as_deref(), Some(&b"Ma"[..]));
    assert_eq!(decode("TQ==").as_deref(), Some(&b"M"[..]));
    for bad in ["TW+u", "TW/u", "TW u", "TWFu\n", "T", "TWFuT", "TW\u{e9}u"] {
        assert_eq!(decode(bad), None, "{bad:?}");
    }
    assert_eq!(decode(&"A".repeat(10_000)), None);
}

#[test]
fn the_cookie_is_the_user_id_after_the_last_pipe_and_the_whole_token() {
    let valid = token("valid");
    let secret = cookie(&valid, NOW).unwrap();
    assert_eq!(secret.expose(), format!("{COOKIE}{valid}"));
    assert_eq!(format!("{secret:?}"), "Secret(***)", "the cookie is a Secret");
}

#[test]
fn user_ids_come_from_sub_in_every_plain_form() {
    for (sub, id) in [
        ("auth0|user_01ABC", "user_01ABC"),
        ("google-oauth2|user_01ABC", "user_01ABC"),
        ("a|b|user_01ABC", "user_01ABC"),
        ("user_01ABC", "user_01ABC"),
    ] {
        let claims = format!(r#"{{"sub":"{sub}","exp":4102444800}}"#);
        assert!(
            built(&claims).unwrap().starts_with(&format!("WorkosCursorSessionToken={id}%3A%3A")),
            "{sub}"
        );
    }
}

#[test]
fn a_sub_that_is_not_a_plain_id_makes_the_token_unusable() {
    for sub in ["auth0|", "", "auth0|user 01", "auth0|user;01", "auth0|us\\u00e9r", "auth0|a%3Ab"] {
        let claims = format!(r#"{{"sub":"{sub}","exp":4102444800}}"#);
        assert_eq!(built(&claims), Err(Unusable::Malformed), "{sub:?}");
    }
    let long = format!(r#"{{"sub":"auth0|{}","exp":4102444800}}"#, "a".repeat(200));
    assert_eq!(built(&long), Err(Unusable::Malformed));
    for claims in
        [r#"{"exp":4102444800}"#, r#"{"sub":7}"#, r#"{"sub":null}"#, "[]", "null", "not json"]
    {
        assert_eq!(built(claims), Err(Unusable::Malformed), "{claims}");
    }
}

#[test]
fn a_token_expiring_within_sixty_seconds_is_not_used() {
    let at = |exp: i64| built(&format!(r#"{{"sub":"auth0|u1","exp":{exp}}}"#));
    assert_eq!(at(NOW + 59), Err(Unusable::Expired));
    assert_eq!(at(NOW), Err(Unusable::Expired));
    assert_eq!(at(1_000_000_000), Err(Unusable::Expired));
    assert!(at(NOW + 60).is_ok(), "exactly 60 s away is still usable");
    assert!(at(NOW + 3_600).is_ok());
    assert_eq!(built(r#"{"sub":"auth0|u1","exp":-1}"#), Err(Unusable::Expired));
    // No `exp` at all: the server decides.
    assert!(built(r#"{"sub":"auth0|u1"}"#).is_ok());
    assert!(built(r#"{"sub":"auth0|u1","exp":"soon"}"#).is_ok());
}

#[test]
fn the_fixture_tokens_behave_as_named() {
    assert!(cookie(&token("valid"), NOW).is_ok());
    assert_eq!(cookie(&token("expired"), NOW).unwrap_err(), Unusable::Expired);
    assert_eq!(cookie(&token("expires_in_30s"), NOW).unwrap_err(), Unusable::Expired);
    assert!(cookie(&token("no_exp"), NOW).is_ok());
    assert_eq!(cookie(&token("no_sub"), NOW).unwrap_err(), Unusable::Malformed);
}

#[test]
fn text_that_is_not_plainly_a_jwt_never_becomes_a_header_value() {
    let valid = token("valid");
    let hostile = [
        String::new(),
        "   ".to_owned(),
        "abc".to_owned(),
        "a.b".to_owned(),
        "a.b.c.d".to_owned(),
        "a..c".to_owned(),
        format!("{valid}\r\nX-Injected: 1"),
        format!("{valid}; Path=/"),
        format!("{valid} extra"),
        format!("{valid}%3A"),
        format!("{valid}\0"),
        format!("a.{}.c", "A".repeat(9_000)),
    ];
    for text in hostile {
        assert_eq!(cookie(&text, NOW).unwrap_err(), Unusable::Malformed, "{text:?}");
    }
    // Surrounding whitespace is not part of the token.
    assert!(cookie(&format!("  {valid}\n"), NOW).is_ok());
}

#[test]
fn a_cookie_can_be_discarded_without_touching_a_view() {
    // Compile-time/behavioural guard: nothing in a fresh view mentions credentials.
    let view = HarnessView::needs_setup(usage_model::HarnessId::Cursor);
    assert!(!serde_json::to_string(&view).unwrap().contains("Workos"));
}
