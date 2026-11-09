//! A language the settings enable whose pack is installed, not built in, and not installed yet.
//! The server installs it in the background and checks the project again once it is in; when it
//! cannot, or may not, it says so, and checks the rest. Nothing written in a missing language is
//! checked, which a person has to be told, or it reads as nothing wrong.

use std::{fs, path::Path, sync::Arc};

use expect_test::expect;

use crate::tests::{Fixture, zed_client};

const NOTES: (&str, &str) = ("notes.md", "Teh помилкка, привіт.\n");

/// What the server said in messages, in order.
fn said(server: &Fixture) -> String {
    server
        .flow
        .params("window/showMessage")
        .iter()
        .map(|params| params["message"].as_str().unwrap_or("?").to_owned())
        .collect::<Vec<_>>()
        .join("\n")
}

async fn published_notes(server: &mut Fixture) -> String {
    server.open("notes.md").await;
    let items = server.published("notes.md").unwrap_or_default();
    server.render(&server.text("notes.md"), &items)
}

#[tokio::test]
async fn with_auto_install_off_the_server_says_how_and_checks_the_rest() {
    let mut server = Fixture::new(&[
        (
            "orthra.toml",
            "languages = [\"en-US\", \"uk-UA\"]\nauto-install = false\n",
        ),
        NOTES,
    ]);
    server.initialize(zed_client()).await;
    server.initialized().await;
    expect!["orthra: uk-UA is not installed, and auto-install is off, so nothing written in it is checked. Install it with `orthra pack install uk-UA`."].assert_eq(&said(&server));
    expect![[r#"
        > Teh помилкка, привіт.
          ^^^ known-typo 2: `Teh` is a misspelling of `The`
    "#]]
    .assert_eq(&published_notes(&mut server).await);
}

#[tokio::test]
async fn a_failed_install_is_said_once_and_the_rest_is_checked() {
    let mut server = Fixture::new(&[
        ("orthra.toml", "languages = [\"en-US\", \"uk-UA\"]\n"),
        NOTES,
    ]);
    server.initialize(zed_client()).await;
    server.initialized().await;
    server.flow.settle().await;
    expect!["orthra: uk-UA could not be installed: a test has no network. Nothing written in it is checked; install it with `orthra pack install uk-UA`."].assert_eq(&said(&server));
    expect![[r#"
        > Teh помилкка, привіт.
          ^^^ known-typo 2: `Teh` is a misspelling of `The`
    "#]]
    .assert_eq(&published_notes(&mut server).await);
}

#[tokio::test]
async fn an_installed_pack_is_checked_with_after_the_next_pass() {
    // Installs a dictionary of the test's own where the server reads uk-UA from, compiled as an
    // install compiles it.
    let installer = Arc::new(|tag: &str, packs: &Path| {
        let dir = orthra_install::definition(tag)
            .map_err(|err| err.0)?
            .dir(packs);
        fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
        let (aff, dic) = ("SET UTF-8\n", "1\nпривіт\n");
        let compiled = orthra_install::compile(aff, dic)
            .map_err(|err| err.0)?
            .forms;
        for (file, bytes) in [
            ("uk_UA.aff", aff.as_bytes()),
            ("uk_UA.dic", dic.as_bytes()),
            ("uk_50k.txt", "привіт 10\n".as_bytes()),
            ("uk_UA.fst", &compiled),
        ] {
            fs::write(dir.join(file), bytes).map_err(|err| err.to_string())?;
        }
        Ok(format!("{tag} for this test"))
    });
    let mut server = Fixture::installing(
        &[
            ("orthra.toml", "languages = [\"en-US\", \"uk-UA\"]\n"),
            NOTES,
        ],
        installer,
    );
    let mut passes = server.flow.server().passes();
    server.initialize(zed_client()).await;
    server.initialized().await;
    passes
        .wait_for(|done| *done >= 2)
        .await
        .expect("the pass after the install");
    server.flow.settle().await;
    assert!(server.packs().join("uk-UA/6.8.6.2/uk_UA.fst").is_file());
    expect!["orthra: installed uk-UA for this test; checking the project again"]
        .assert_eq(&said(&server));
    expect![[r#"
        > Teh помилкка, привіт.
          ^^^ known-typo 2: `Teh` is a misspelling of `The`
              ^^^^^^^^ unknown-word 2: `помилкка` is not a known word
    "#]]
    .assert_eq(&published_notes(&mut server).await);
}
