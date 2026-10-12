use super::*;

fn flags(list: &[&str]) -> ServerFlags {
    let args: Vec<String> = list.iter().map(|arg| (*arg).to_owned()).collect();
    ServerFlags::parse(&args)
}

fn listening(ports: &[u16]) -> BTreeSet<u16> {
    ports.iter().copied().collect()
}

fn shape(targets: &[Target]) -> Vec<(Scheme, u16, &str)> {
    targets.iter().map(|target| (target.scheme, target.port, target.token.expose())).collect()
}

#[test]
fn https_is_tried_on_the_declared_port_then_on_every_listening_port() {
    let server =
        flags(&["--csrf_token", "tok", "--https_server_port", "8443", "--lsp_port", "50200"]);
    let targets = plan_targets(&server, &listening(&[50123, 8443, 50200, 50150]));
    // The LSP port speaks JSON-RPC over TCP, not HTTP: never probed.
    assert_eq!(
        shape(&targets),
        [
            (Scheme::Https, 8443, "tok"),
            (Scheme::Https, 50123, "tok"),
            (Scheme::Https, 50150, "tok"),
        ]
    );
}

#[test]
fn a_random_port_server_is_found_through_the_port_table_alone() {
    let server = flags(&["--csrf_token", "tok", "--https_server_port", "0"]);
    let targets = plan_targets(&server, &listening(&[50200, 50123]));
    assert_eq!(shape(&targets), [(Scheme::Https, 50123, "tok"), (Scheme::Https, 50200, "tok")]);
}

#[test]
fn a_busy_process_is_probed_on_a_few_ports_only() {
    let many: Vec<u16> = (40_000..40_050).collect();
    let targets = plan_targets(&flags(&["--csrf_token", "tok"]), &listening(&many));
    assert_eq!(targets.len(), MAX_HTTPS_PORTS);
    assert_eq!(targets[0].port, 40_000);
    assert!(targets.iter().all(|target| target.scheme == Scheme::Https));
}

#[test]
fn the_extension_server_is_a_last_resort_over_plain_http_with_its_own_token() {
    let server = flags(&[
        "--csrf_token",
        "main",
        "--extension_server_csrf_token",
        "ext",
        "--extension_server_port",
        "50999",
    ]);
    let targets = plan_targets(&server, &listening(&[50123]));
    assert_eq!(shape(&targets), [(Scheme::Https, 50123, "main"), (Scheme::Http, 50999, "ext")]);
    // Without a token of its own it presents the main one.
    let shared = flags(&["--csrf_token", "main", "--extension_server_port", "50999"]);
    assert_eq!(shape(&plan_targets(&shared, &listening(&[]))), [(Scheme::Http, 50999, "main")]);
    // Its token alone is enough for the fallback, and for nothing else.
    let only_ext =
        flags(&["--extension_server_csrf_token", "ext", "--extension_server_port", "50999"]);
    assert_eq!(
        shape(&plan_targets(&only_ext, &listening(&[50123]))),
        [(Scheme::Http, 50999, "ext")]
    );
}

#[test]
fn no_usable_token_or_no_address_means_no_targets() {
    assert!(plan_targets(&flags(&[]), &listening(&[50123])).is_empty());
    assert!(plan_targets(&flags(&["--csrf_token", "tok"]), &listening(&[])).is_empty());
    // A token that cannot travel as a header is no token.
    let unicode = flags(&["--csrf_token=t\u{f3}k", "--extension_server_port=5"]);
    assert!(plan_targets(&unicode, &listening(&[50123])).is_empty());
    let spaced = flags(&["--csrf_token", "to k", "--extension_server_port", "5"]);
    assert!(plan_targets(&spaced, &listening(&[50123])).is_empty());
    // A bad extension token falls back to the main one.
    let mixed = flags(&[
        "--csrf_token",
        "main",
        "--extension_server_csrf_token=\u{e7}",
        "--extension_server_port=5",
    ]);
    assert_eq!(shape(&plan_targets(&mixed, &listening(&[]))), [(Scheme::Http, 5, "main")]);
}

#[test]
fn a_target_never_prints_its_token() {
    let targets = plan_targets(&flags(&["--csrf_token", "visible-secret-3"]), &listening(&[1234]));
    assert_eq!(targets.len(), 1);
    assert!(!format!("{:?} {:#?}", targets[0], targets).contains("visible-secret"));
}

fn mapped(error: DiscoveryError) -> HarnessError {
    error.into()
}

#[test]
fn discovery_failures_map_to_the_most_specific_error() {
    let not_running = mapped(DiscoveryError::NotRunning);
    assert_eq!((not_running.code, not_running.retryable), (ErrorCode::NotRunning, true));
    assert!(not_running.params.is_empty());

    let starting = mapped(DiscoveryError::NoPort(None));
    assert_eq!((starting.code, starting.retryable), (ErrorCode::NotRunning, true));
    assert_eq!(starting.params.get("reason").map(String::as_str), Some("no_port"));

    let cause = HarnessError::new(ErrorCode::Timeout, true).with_param("host", "x");
    assert_eq!(mapped(DiscoveryError::NoPort(Some(cause.clone()))), cause);
    assert_eq!(mapped(DiscoveryError::Failed(cause.clone())), cause);

    let tokenless = mapped(DiscoveryError::NoToken);
    assert_eq!((tokenless.code, tokenless.retryable), (ErrorCode::Unauthorized, false));
    assert_eq!(tokenless.params.get("reason").map(String::as_str), Some("no_token"));
}
