use std::cell::RefCell;

use super::*;

const ALLOWED_HOSTS: [&str; 4] = ["opencode.ai", "github.com", "www.kimi.com", "z.ai"];

#[test]
fn every_link_maps_to_a_fixed_https_url_on_a_known_host() {
    for link in ExternalLink::ALL {
        let url = url_for(link);
        let rest = url.strip_prefix("https://").unwrap_or_else(|| panic!("{link:?}: {url}"));
        let host = rest.split(['/', '#', '?']).next().expect("host");
        assert!(ALLOWED_HOSTS.contains(&host), "{link:?}: {url}");
        assert!(url.chars().all(|c| c.is_ascii_graphic()), "{link:?}: {url}");
        assert!(!url.contains(['"', '\'', '<', '>', '`', '\\', '%', '@']), "{link:?}: {url}");
    }
}

#[test]
fn opening_a_link_launches_exactly_its_url() {
    for link in ExternalLink::ALL {
        let seen = RefCell::new(Vec::new());
        open(link, &|url| {
            seen.borrow_mut().push(url.to_owned());
            Ok(())
        })
        .expect("open");
        assert_eq!(*seen.borrow(), [url_for(link)]);
    }
}

#[test]
fn a_launcher_failure_becomes_a_spawn_failed_error_without_the_cause() {
    let err = open(ExternalLink::ProjectReadme, &|_| Err(io::Error::other("C:\\secret\\path")))
        .expect_err("must fail");
    assert_eq!(err.code, ErrorCode::SpawnFailed);
    assert!(!err.retryable);
    assert_eq!(err.params.get("target").map(String::as_str), Some("browser"));
    assert!(!format!("{err:?}").contains("secret"));
}
