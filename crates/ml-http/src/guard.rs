//! The loopback guard: a document's text goes to this machine only, unless the user wrote
//! `allowRemote` (`docs/04-context.md` rule 7, `CLAUDE.md` §5).
//!
//! The URL is read by the parser the request is sent with (`ureq::http::Uri`), and the request is
//! sent to the URL this returns, so the guard and the connection cannot read one URL two ways. A
//! host is this machine when it is an address in 127.0.0.0/8, `::1`, or the name `localhost`. No
//! other name is looked up to decide: what a name resolves to is somebody else's to change, between
//! the check and the request and after both.

use std::net::{Ipv4Addr, Ipv6Addr};

use orthra_context::ScoreError;
use ureq::http::Uri;

#[cfg(test)]
mod tests;

/// `url`, if text may be sent to it: http or https, no credentials in it, and a host on this
/// machine unless `allow_remote`.
pub(crate) fn checked(url: &str, allow_remote: bool) -> Result<Uri, ScoreError> {
    let refused = |why: &str| ScoreError(format!("orthra sends no text to `{url}`: {why}"));
    let uri: Uri = url
        .parse()
        .map_err(|err| refused(&format!("it is not a URL ({err})")))?;
    if !matches!(uri.scheme_str(), Some("http" | "https")) {
        return Err(refused("it is not http or https"));
    }
    let Some(authority) = uri.authority() else {
        return Err(refused("it names no host"));
    };
    // `http://127.0.0.1@example.com/` is example.com. Credentials are refused outright, rather than
    // read past: a URL that can be misread by a person is not one to send text to. The URL is not
    // repeated, so that a password in it is not shown or logged.
    if authority.as_str().contains('@') {
        return Err(ScoreError(
            "orthra sends no text to an endpoint with credentials in its URL".to_owned(),
        ));
    }
    if allow_remote || is_loopback(authority.host()) {
        return Ok(uri);
    }
    Err(refused(&format!(
        "`{}` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set",
        authority.host()
    )))
}

/// Whether `host`, as a URL writes it, is this machine.
fn is_loopback(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if let Some(v6) = host.strip_prefix('[').and_then(|v6| v6.strip_suffix(']')) {
        return v6.parse::<Ipv6Addr>().is_ok_and(|ip| ip.is_loopback());
    }
    // Strictly dotted-quad: `127.1` and `2130706433` are names here, and refused, though a resolver
    // might read them as 127.0.0.1.
    host.parse::<Ipv4Addr>().is_ok_and(|ip| ip.is_loopback())
}
