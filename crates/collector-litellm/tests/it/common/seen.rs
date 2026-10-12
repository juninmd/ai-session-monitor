//! What the fake proxy saw of one request, and how a server stack would read its query.

/// One request as received.
#[derive(Clone, Debug)]
pub struct Seen {
    pub method: String,
    pub target: String,
    pub headers: Vec<(String, String)>,
}

impl Seen {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }

    pub fn path(&self) -> &str {
        self.target.split('?').next().unwrap_or_default()
    }

    pub fn query(&self) -> &str {
        self.target.split_once('?').map_or("", |(_, query)| query)
    }

    /// The value of one query parameter as a server stack would decode it.
    pub fn param(&self, name: &str) -> Option<String> {
        self.query()
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .find(|(key, _)| decode(key) == name)
            .map(|(_, value)| decode(value))
    }
}

/// Percent-decoding where a bare `+` is a space, as form decoding does: a client that leaves a raw
/// `+` in an id would be read back as a different id.
fn decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let escaped = (bytes[at] == b'%')
            .then(|| bytes.get(at + 1..at + 3))
            .flatten()
            .and_then(|hex| std::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match (bytes[at], escaped) {
            (_, Some(byte)) => {
                out.push(byte);
                at += 3;
            }
            (b'+', None) => {
                out.push(b' ');
                at += 1;
            }
            (byte, None) => {
                out.push(byte);
                at += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seen(target: &str) -> Seen {
        Seen { method: "GET".to_owned(), target: target.to_owned(), headers: Vec::new() }
    }

    #[test]
    fn the_decoder_reads_queries_like_a_server_stack() {
        let request = seen("/x?a=%C3%A9&b=1+2&c=%2B&d=%zz&e=%4&f");
        assert_eq!(request.param("a").as_deref(), Some("é"));
        assert_eq!(request.param("b").as_deref(), Some("1 2"), "a raw plus is a space");
        assert_eq!(request.param("c").as_deref(), Some("+"));
        assert_eq!(request.param("d").as_deref(), Some("%zz"), "bad escapes stay literal");
        assert_eq!(request.param("e").as_deref(), Some("%4"));
        assert_eq!(request.param("f"), None);
        assert_eq!(request.param("missing"), None);
        assert_eq!((request.path(), seen("/p").query()), ("/x", ""));
    }
}
