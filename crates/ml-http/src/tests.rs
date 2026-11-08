//! The client against a server of the test's own, on this machine: no test needs a model.

use std::{
    io::{BufRead, BufReader, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread::{self, JoinHandle},
    time::Duration,
};

use expect_test::expect;
use orthra_context::{ApiKey, Connector, Endpoint, Prompts, Rewrite, Rewriter, ScoreError};
use serde_json::{Value, json};

use crate::Http;

/// What the server was asked: the request line, the content type, the authorization, and the body.
struct Asked {
    line: String,
    content_type: String,
    authorization: String,
    body: String,
}

/// A server on this machine that answers one request with `response`, written whole: its
/// endpoint, and what it was asked once it has answered.
fn serve(response: String) -> (String, JoinHandle<Asked>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port on this machine");
    let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("the client connects");
        let asked = read_request(&mut stream);
        // A client that stops reading at its limit closes the connection before this is written.
        let _ = stream.write_all(response.as_bytes());
        asked
    });
    (endpoint, server)
}

fn read_request(stream: &mut TcpStream) -> Asked {
    let mut reader = BufReader::new(stream);
    let mut head = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("a request");
        if line.trim_end().is_empty() {
            break;
        }
        head.push(line.trim_end().to_owned());
    }
    let header = |name: &str| {
        head.iter()
            .find_map(|line| {
                let (key, value) = line.split_once(':')?;
                key.eq_ignore_ascii_case(name)
                    .then(|| value.trim().to_owned())
            })
            .unwrap_or_default()
    };
    let length = header("content-length").parse().unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body).expect("the body");
    Asked {
        line: head.first().cloned().unwrap_or_default(),
        content_type: header("content-type"),
        authorization: header("authorization"),
        body: String::from_utf8(body).expect("UTF-8"),
    }
}

fn http(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// A chat completion whose one choice says `content`.
fn completion(content: &str) -> String {
    let body = json!({
        "id": "chatcmpl-1",
        "object": "chat.completion",
        "model": "qwen3:4b",
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": content },
            "finish_reason": "stop",
        }],
    });
    http("200 OK", &body.to_string())
}

fn endpoint(url: &str) -> Endpoint {
    Endpoint {
        url: url.to_owned(),
        model: "qwen3:4b".to_owned(),
        allow_remote: false,
        api_key: None,
        prompts: Prompts::default(),
    }
}

fn rewriter(url: &str) -> std::sync::Arc<dyn Rewriter> {
    Http::default()
        .connect(&endpoint(url))
        .expect("a loopback endpoint")
}

/// An error, with the test server's port taken out.
fn error(got: Result<String, ScoreError>, endpoint: &str) -> String {
    match got {
        Ok(rewrite) => format!("rewritten: {rewrite:?}"),
        Err(err) => err.0.replace(endpoint, "<endpoint>"),
    }
}

#[test]
fn a_sentence_is_posted_with_the_prompt_and_the_answer_is_its_rewrite() {
    let (url, server) = serve(completion("\"The cat sat on the mat.\""));
    let got = rewriter(&url).rewrite("Teh cat sat on teh mat.", "en-US", Rewrite::Correct);
    let asked = server.join().unwrap();
    let body: Value = serde_json::from_str(&asked.body).expect("a JSON body");
    expect![[r#"
        POST /v1/chat/completions HTTP/1.1
        content-type: application/json
        {
          "messages": [
            {
              "content": "You correct the spelling and grammar of text. The user's message is one sentence of a document, or part of one, written in en-US. Keep its meaning, keep its wording wherever it is correct, and keep its markup and line breaks exactly as they are: Markdown, backticks, links, code. Do not complete it, explain it or add to it. Reply with the corrected text only, without quotes.",
              "role": "system"
            },
            {
              "content": "Teh cat sat on teh mat.",
              "role": "user"
            }
          ],
          "model": "qwen3:4b",
          "stream": false,
          "temperature": 0
        }
        -> Ok("The cat sat on the mat.")"#]]
    .assert_eq(&format!(
        "{}\ncontent-type: {}\n{}\n-> {got:?}",
        asked.line,
        asked.content_type,
        serde_json::to_string_pretty(&body).unwrap()
    ));
}

#[test]
fn a_language_no_pack_speaks_is_not_named() {
    let (url, server) = serve(completion("Γεια σου."));
    let got = rewriter(&format!("{url}/")).rewrite("Γεια σου.", "und", Rewrite::Correct);
    let asked = server.join().unwrap();
    let body: Value = serde_json::from_str(&asked.body).unwrap();
    let prompt = body["messages"][0]["content"].as_str().unwrap();
    assert!(prompt.contains("or part of one. Keep"), "{prompt}");
    assert_eq!(asked.line, "POST /v1/chat/completions HTTP/1.1");
    assert_eq!(got, Ok("Γεια σου.".to_owned()));
}

#[test]
fn an_error_status_says_what_the_endpoint_said() {
    let (url, server) = serve(http(
        "404 Not Found",
        r#"{"error":{"message":"model \"qwen3:4b\" not found, try pulling it first","type":"api_error","param":null,"code":null}}"#,
    ));
    let got = rewriter(&url).rewrite("Teh cat.", "en-US", Rewrite::Correct);
    server.join().unwrap();
    let (old, old_server) = serve(http(
        "500 Internal Server Error",
        r#"{"error":"out of memory"}"#,
    ));
    let old_got = rewriter(&old).rewrite("Teh cat.", "en-US", Rewrite::Correct);
    old_server.join().unwrap();
    let (bare, bare_server) = serve(http("502 Bad Gateway", "<html>Bad gateway</html>"));
    let bare_got = rewriter(&bare).rewrite("Teh cat.", "en-US", Rewrite::Correct);
    bare_server.join().unwrap();
    expect![[r#"
        <endpoint>/chat/completions: answered 404 Not Found: model "qwen3:4b" not found, try pulling it first
        <endpoint>/chat/completions: answered 500 Internal Server Error: out of memory
        <endpoint>/chat/completions: answered 502 Bad Gateway"#]]
    .assert_eq(&[
        error(got, &url),
        error(old_got, &old),
        error(bare_got, &bare),
    ]
    .join("\n"));
}

#[test]
fn an_answer_that_is_not_a_rewrite_is_an_error() {
    let (url, server) = serve(completion(
        "Here is the corrected sentence:\n\nThe cat sat.",
    ));
    let got = rewriter(&url).rewrite("Teh cat sat.", "en-US", Rewrite::Correct);
    server.join().unwrap();
    expect![[r#"<endpoint>/chat/completions: the model answered more lines than the sentence has, which is not a rewrite of it"#]]
    .assert_eq(&error(got, &url));
}

#[test]
fn a_redirect_is_not_followed() {
    let elsewhere = TcpListener::bind("127.0.0.1:0").unwrap();
    let to = format!(
        "http://{}/v1/chat/completions",
        elsewhere.local_addr().unwrap()
    );
    let (url, server) = serve(format!(
        "HTTP/1.1 307 Temporary Redirect\r\nLocation: {to}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    ));
    let got = rewriter(&url).rewrite("Teh cat.", "en-US", Rewrite::Correct);
    server.join().unwrap();
    elsewhere.set_nonblocking(true).unwrap();
    assert!(
        matches!(elsewhere.accept(), Err(err) if err.kind() == ErrorKind::WouldBlock),
        "the redirect was followed"
    );
    expect!["<endpoint>/chat/completions: answered 307 Temporary Redirect"]
        .assert_eq(&error(got, &url));
}

#[test]
fn an_endpoint_slower_than_the_timeout_is_an_error() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v1", listener.local_addr().unwrap());
    let (done, wait) = mpsc::channel::<()>();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        read_request(&mut stream);
        // Holds the connection open, answering nothing, until the test is over.
        let _ = wait.recv();
    });
    let got = Http::timing_out_after(Duration::from_millis(200))
        .connect(&endpoint(&url))
        .unwrap()
        .rewrite("Teh cat.", "en-US", Rewrite::Correct);
    done.send(()).unwrap();
    server.join().unwrap();
    let got = error(got, &url);
    assert!(
        got.starts_with("<endpoint>/chat/completions: timeout"),
        "{got}"
    );
}

#[test]
fn an_answer_larger_than_the_limit_is_an_error() {
    let (url, server) = serve(completion(&"word ".repeat(60_000)));
    let got = rewriter(&url).rewrite("Teh cat.", "en-US", Rewrite::Correct);
    let _ = server.join();
    let got = error(got, &url);
    assert!(got.contains("larger than request limit"), "{got}");
}

#[test]
fn an_endpoint_nobody_listens_on_is_an_error() {
    let url = {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        format!("http://{}/v1", listener.local_addr().unwrap())
    };
    let got = rewriter(&url).rewrite("Teh cat.", "en-US", Rewrite::Correct);
    assert!(got.is_err(), "{got:?}");
}

#[test]
fn an_endpoint_on_another_machine_is_refused_before_anything_is_sent() {
    let refused = |url: &str, allow_remote: bool| {
        Http::default()
            .connect(&Endpoint {
                url: url.to_owned(),
                model: "qwen3:4b".to_owned(),
                allow_remote,
                api_key: None,
                prompts: Prompts::default(),
            })
            .map(|_| "connected".to_owned())
            .unwrap_or_else(|err| err.0)
    };
    let model = Http::default()
        .connect(&Endpoint {
            model: " ".to_owned(),
            ..endpoint("http://127.0.0.1:11434/v1")
        })
        .map(|_| "connected".to_owned())
        .unwrap_or_else(|err| err.0);
    expect![[r#"
        orthra sends no text to `http://192.168.1.5:11434/v1/chat/completions`: `192.168.1.5` is not this machine (127.0.0.1, ::1 or localhost), and `allowRemote` is not set
        connected
        connected
        no model is named for http://127.0.0.1:11434/v1"#]]
    .assert_eq(
        &[
            refused("http://192.168.1.5:11434/v1", false),
            refused("http://192.168.1.5:11434/v1", true),
            refused("http://localhost:11434/v1/", false),
            model,
        ]
        .join("\n"),
    );
}

/// A rephrasing gets orthra's rephrasing prompt, a prompt the settings give replaces orthra's with
/// the language named where it says, and an API key goes as a bearer token.
#[test]
fn a_rephrasing_a_prompt_of_ones_own_and_an_api_key_are_sent() {
    let ask = |endpoint: Endpoint, how: Rewrite| {
        let (url, server) = serve(completion("Said again."));
        let got = Http::default()
            .connect(&Endpoint {
                url: url.clone(),
                ..endpoint
            })
            .unwrap()
            .rewrite("Said it.", "en-GB", how);
        let asked = server.join().unwrap();
        let body: Value = serde_json::from_str(&asked.body).unwrap();
        format!(
            "{} | authorization: {:?} | {got:?}",
            body["messages"][0]["content"].as_str().unwrap(),
            asked.authorization,
        )
    };
    let own = Endpoint {
        api_key: Some(ApiKey("sk-local-1".to_owned())),
        prompts: Prompts {
            correct: None,
            rephrase: Some("Say it more formally{language}.".to_owned()),
        },
        ..endpoint("")
    };
    expect![[r#"
        You rephrase text so that it reads naturally. The user's message is part of a document, written in en-GB. Keep its meaning and its language, and keep its markup and line breaks exactly as they are: Markdown, backticks, links, code. Correct its spelling and grammar too. Do not complete it, explain it or add to it. Reply with the rephrased text only, without quotes. | authorization: "" | Ok("Said again.")
        Say it more formally, written in en-GB. | authorization: "Bearer sk-local-1" | Ok("Said again.")
        You correct the spelling and grammar of text. The user's message is one sentence of a document, or part of one, written in en-GB. Keep its meaning, keep its wording wherever it is correct, and keep its markup and line breaks exactly as they are: Markdown, backticks, links, code. Do not complete it, explain it or add to it. Reply with the corrected text only, without quotes. | authorization: "Bearer sk-local-1" | Ok("Said again.")
        Endpoint { url: "", model: "qwen3:4b", allow_remote: false, api_key: Some(<api key>), prompts: Prompts { correct: None, rephrase: Some("Say it more formally{language}.") } }"#]].assert_eq(
        &[
            ask(endpoint(""), Rewrite::Rephrase),
            ask(own.clone(), Rewrite::Rephrase),
            ask(own.clone(), Rewrite::Correct),
            format!("{own:?}"),
        ]
        .join("\n"),
    );
}
