//! Error helpers shared by the parsers and the collector.

use usage_model::{ErrorCode, HarnessError};

/// The payload is not the shape this collector was written for.
pub(crate) fn endpoint_changed() -> HarnessError {
    HarnessError::new(ErrorCode::EndpointChanged, false)
}

/// How much an error says about what is wrong; the highest wins when several targets failed.
/// A server that answered and refused beats one that merely did not answer.
const fn rank(code: ErrorCode) -> u8 {
    match code {
        ErrorCode::Unauthorized | ErrorCode::NotSignedIn => 9,
        ErrorCode::RateLimited => 8,
        ErrorCode::ServerError => 7,
        ErrorCode::EndpointChanged => 6,
        ErrorCode::ResponseTooLarge => 5,
        ErrorCode::BadResponse => 4,
        ErrorCode::NotFound => 3,
        ErrorCode::Timeout => 2,
        ErrorCode::Network | ErrorCode::TlsError => 1,
        _ => 0,
    }
}

pub(crate) fn most_specific(first: HarnessError, second: HarnessError) -> HarnessError {
    if rank(second.code) > rank(first.code) { second } else { first }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(code: ErrorCode) -> HarnessError {
        HarnessError::new(code, false)
    }

    #[test]
    fn a_refusal_beats_silence_and_the_first_wins_a_tie() {
        let pick = |a, b| most_specific(error(a), error(b)).code;
        assert_eq!(pick(ErrorCode::Network, ErrorCode::Unauthorized), ErrorCode::Unauthorized);
        assert_eq!(pick(ErrorCode::Unauthorized, ErrorCode::Network), ErrorCode::Unauthorized);
        assert_eq!(pick(ErrorCode::TlsError, ErrorCode::Timeout), ErrorCode::Timeout);
        assert_eq!(pick(ErrorCode::Network, ErrorCode::TlsError), ErrorCode::Network);
        assert_eq!(
            pick(ErrorCode::BadResponse, ErrorCode::EndpointChanged),
            ErrorCode::EndpointChanged
        );
    }
}
