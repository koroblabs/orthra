use expect_test::expect;

use super::checked;

/// Each URL, and whether text may be sent to it with `allowRemote` off.
fn verdicts(urls: &[&str], allow_remote: bool) -> String {
    urls.iter()
        .map(|url| match checked(url, allow_remote) {
            Ok(uri) => format!("sent: {uri}\n"),
            Err(err) => format!("{err}\n"),
        })
        .collect()
}

#[test]
fn text_goes_only_to_this_machine() {
    expect![[r#"
        sent: http://127.0.0.1:11434/v1/chat/completions
        sent: http://127.1.2.3:8080/v1/chat/completions
        sent: http://[::1]:11434/v1/chat/completions
        sent: http://localhost:1234/v1/chat/completions
        sent: http://LocalHost:1234/v1/chat/completions
        sent: https://localhost/v1/chat/completions
    "#]]
    .assert_eq(&verdicts(
        &[
            "http://127.0.0.1:11434/v1/chat/completions",
            "http://127.1.2.3:8080/v1/chat/completions",
            "http://[::1]:11434/v1/chat/completions",
            "http://localhost:1234/v1/chat/completions",
            "http://LocalHost:1234/v1/chat/completions",
            "https://localhost/v1/chat/completions",
        ],
        false,
    ));
}

#[test]
fn another_machine_or_a_name_that_only_looks_local_is_refused() {
    expect![[r#"
        orthra sends no text to `http://0.0.0.0:11434/v1`: `0.0.0.0` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set
        orthra sends no text to `http://192.168.1.5:11434/v1`: `192.168.1.5` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set
        orthra sends no text to `https://example.com/v1`: `example.com` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set
        orthra sends no text to `http://localhost.evil.com/v1`: `localhost.evil.com` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set
        orthra sends no text to `http://127.0.0.1.nip.io/v1`: `127.0.0.1.nip.io` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set
        orthra sends no text to `http://localhost./v1`: `localhost.` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set
        orthra sends no text to `http://[::]:11434/v1`: `[::]` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set
        orthra sends no text to `http://[::ffff:127.0.0.1]/v1`: `[::ffff:127.0.0.1]` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set
        orthra sends no text to `http://127.1/v1`: `127.1` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set
        orthra sends no text to `http://2130706433/v1`: `2130706433` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set
        orthra sends no text to `http://example.com#@127.0.0.1/v1`: `example.com` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set
    "#]]
    .assert_eq(&verdicts(
        &[
            "http://0.0.0.0:11434/v1",
            "http://192.168.1.5:11434/v1",
            "https://example.com/v1",
            "http://localhost.evil.com/v1",
            "http://127.0.0.1.nip.io/v1",
            "http://localhost./v1",
            "http://[::]:11434/v1",
            "http://[::ffff:127.0.0.1]/v1",
            "http://127.1/v1",
            "http://2130706433/v1",
            "http://example.com#@127.0.0.1/v1",
        ],
        false,
    ));
}

#[test]
fn credentials_another_scheme_or_no_host_are_refused_even_when_remote_is_allowed() {
    let urls = [
        "http://user:secret@127.0.0.1:11434/v1",
        "http://127.0.0.1@example.com/v1",
        "http://127.0.0.1:11434@example.com/v1",
        "ftp://127.0.0.1/v1",
        "127.0.0.1:11434/v1",
        "/v1/chat/completions",
        "http://",
    ];
    let local = verdicts(&urls, false);
    assert_eq!(local, verdicts(&urls, true));
    expect![[r#"
        orthra sends no text to an endpoint with credentials in its URL
        orthra sends no text to an endpoint with credentials in its URL
        orthra sends no text to an endpoint with credentials in its URL
        orthra sends no text to `ftp://127.0.0.1/v1`: it is not http or https
        orthra sends no text to `127.0.0.1:11434/v1`: it is not a URL (invalid format)
        orthra sends no text to `/v1/chat/completions`: it is not http or https
        orthra sends no text to `http://`: it is not a URL (empty string)
    "#]]
    .assert_eq(&local);
}

#[test]
fn allowed_remote_another_machine_is_sent_to() {
    expect![[r#"
        sent: https://example.com/v1/chat/completions
        sent: http://192.168.1.5:11434/v1/chat/completions
    "#]]
    .assert_eq(&verdicts(
        &[
            "https://example.com/v1/chat/completions",
            "http://192.168.1.5:11434/v1/chat/completions",
        ],
        true,
    ));
}
