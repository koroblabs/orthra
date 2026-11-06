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
