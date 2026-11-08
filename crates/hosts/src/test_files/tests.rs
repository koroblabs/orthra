use expect_test::expect;

use super::is_test_path;
use crate::Hosts;

#[test]
fn each_host_names_its_own_test_files() {
    let hosts = Hosts::builtin();
    let paths = [
        "/src/lib.rs",
        "/src/tests.rs",
        "/src/parse_test.rs",
        "/src/parse_tests.rs",
        "/tests/flow.rs",
        "/crates/engine/test/flow.rs",
        "/app/models.py",
        "/app/test_models.py",
        "/app/models_test.py",
        "/app/conftest.py",
        "/app/testing.py",
        "/cmd/main.go",
        "/cmd/main_test.go",
        "/cmd/testdata/input.go",
        "/web/app.ts",
        "/web/app.test.ts",
        "/web/app.spec.tsx",
        "/web/__tests__/app.ts",
        "/web/latest.ts",
        "C:\\web\\tests\\app.ts",
        "/README.md",
        "/tests/README.md",
    ];
    let rendered: String = paths
        .iter()
        .map(|path| {
            let host = hosts.for_path(path).expect("a host for every path here");
            let verdict = if is_test_path(host, path) {
                "test"
            } else {
                "-"
            };
            format!("{verdict:<4} {path}\n")
        })
        .collect();
    expect![[r#"
        -    /src/lib.rs
        test /src/tests.rs
        test /src/parse_test.rs
        test /src/parse_tests.rs
        test /tests/flow.rs
        test /crates/engine/test/flow.rs
        -    /app/models.py
        test /app/test_models.py
        test /app/models_test.py
        test /app/conftest.py
        -    /app/testing.py
        -    /cmd/main.go
        test /cmd/main_test.go
        test /cmd/testdata/input.go
        -    /web/app.ts
        test /web/app.test.ts
        test /web/app.spec.tsx
        test /web/__tests__/app.ts
        -    /web/latest.ts
        test C:\web\tests\app.ts
        -    /README.md
        test /tests/README.md
    "#]]
    .assert_eq(&rendered);
}

/// A file a tool writes is claimed by no host, by its name: a lock file, a minified bundle, a
/// parser generator's output, protobuf's code.
#[test]
fn a_generated_file_is_claimed_by_no_host() {
    let hosts = Hosts::builtin();
    let paths = [
        "web/package-lock.json",
        "web/package.json",
        "dist/axios.min.js",
        "src/axios.js",
        "public/css/site.min.css",
        "public/css/site.css",
        "src/lex.yy.c",
        "src/parser.tab.c",
        "src/parser.c",
        "api/user.pb.go",
        "api/user.go",
        "proto/user_pb2.py",
        "Sources/User.pb.swift",
        "lib/user_pb.rb",
    ];
    let claimed: Vec<String> = paths
        .iter()
        .map(|path| {
            format!(
                "{:<11}{path}",
                hosts.for_path(path).map_or("-", |host| host.id())
            )
        })
        .collect();
    expect![[r#"
        -          web/package-lock.json
        json       web/package.json
        -          dist/axios.min.js
        javascript src/axios.js
        -          public/css/site.min.css
        css        public/css/site.css
        -          src/lex.yy.c
        -          src/parser.tab.c
        c          src/parser.c
        -          api/user.pb.go
        go         api/user.go
        -          proto/user_pb2.py
        -          Sources/User.pb.swift
        -          lib/user_pb.rb
    "#]]
    .assert_eq(&(claimed.join("\n") + "\n"));
}

/// A project's text files with no extension are plain text by their names, a pattern included;
/// one with an extension keeps its host.
#[test]
fn a_projects_text_files_are_claimed_by_name() {
    let hosts = Hosts::builtin();
    let paths = [
        "LICENSE",
        "LICENSE-MIT",
        "crates/core/LICENSE-APACHE",
        "LICENSE.md",
        "AUTHORS",
        "NOTICE",
        "CHANGELOG",
        "CHANGELOG.md",
        "Makefile",
        "LICENSES",
    ];
    let claimed: Vec<String> = paths
        .iter()
        .map(|path| {
            format!(
                "{:<11}{path}",
                hosts.for_path(path).map_or("-", |host| host.id())
            )
        })
        .collect();
    expect![[r#"
        plain-text LICENSE
        plain-text LICENSE-MIT
        plain-text crates/core/LICENSE-APACHE
        markdown   LICENSE.md
        plain-text AUTHORS
        plain-text NOTICE
        plain-text CHANGELOG
        markdown   CHANGELOG.md
        makefile   Makefile
        -          LICENSES
    "#]]
    .assert_eq(&(claimed.join("\n") + "\n"));
}
