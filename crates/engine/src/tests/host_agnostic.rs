//! `host-agnostic prose` (`docs/02-testing.md` level 4): one body of prose, written in every
//! host's comment syntax, must give the same findings as the same lines in a plain text file. It
//! catches a host that strips a decoration wrongly (the ` * ` of a block comment), or that reads
//! the same words under different rules (code between backticks).

use expect_test::expect;

use super::subject::packs;
use crate::Engine;

/// Lines of prose with something for every check: known typos, an unknown word, a contraction, a
/// URL, Cyrillic, a homoglyph and code between backticks.
const BODY: &[&str] = &[
    "Seperates the greeting from teh name, and occured twice.",
    "It's a qzxv word; https://example.com/qzxv is a link.",
    "Привіт stays, and сount is two scripts.",
    "Call `recieve_message()` first, then `qzxvRender`.",
];

/// How a host writes the body: before it, before each line, after it.
struct Wrapping {
    language: &'static str,
    name: &'static str,
    open: &'static str,
    line: &'static str,
    close: &'static str,
}

const WRAPPINGS: &[Wrapping] = &[
    Wrapping {
        language: "markdown",
        name: "paragraph",
        open: "",
        line: "",
        close: "",
    },
    Wrapping {
        language: "rust",
        name: "//",
        open: "",
        line: "// ",
        close: "fn f() {}\n",
    },
    Wrapping {
        language: "rust",
        name: "///",
        open: "",
        line: "/// ",
        close: "fn f() {}\n",
    },
    Wrapping {
        language: "rust",
        name: "//!",
        open: "",
        line: "//! ",
        close: "",
    },
    Wrapping {
        language: "rust",
        name: "/* */",
        open: "/*\n",
        line: " * ",
        close: " */\n",
    },
    Wrapping {
        language: "rust",
        name: "/** */",
        open: "/**\n",
        line: " * ",
        close: " */\nfn f() {}\n",
    },
    Wrapping {
        language: "python",
        name: "#",
        open: "",
        line: "# ",
        close: "",
    },
    Wrapping {
        language: "python",
        name: "docstring",
        open: "def f():\n    \"\"\"\n",
        line: "    ",
        close: "    \"\"\"\n",
    },
    Wrapping {
        language: "go",
        name: "//",
        open: "",
        line: "// ",
        close: "package main\n",
    },
    Wrapping {
        language: "go",
        name: "/* */",
        open: "/*\n",
        line: "",
        close: "*/\npackage main\n",
    },
    Wrapping {
        language: "sql",
        name: "--",
        open: "",
        line: "-- ",
        close: "",
    },
    Wrapping {
        language: "sql",
        name: "/* */",
        open: "/*\n",
        line: "",
        close: "*/\n",
    },
    Wrapping {
        language: "typescript",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "typescript",
        name: "/** */",
        open: "/**\n",
        line: " * ",
        close: " */\nfunction f() {}\n",
    },
    Wrapping {
        language: "typescriptreact",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "javascript",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "javascript",
        name: "/** */",
        open: "/**\n",
        line: " * ",
        close: " */\nfunction f() {}\n",
    },
    Wrapping {
        language: "javascriptreact",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "php",
        name: "//",
        open: "<?php\n",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "php",
        name: "#",
        open: "<?php\n",
        line: "# ",
        close: "",
    },
    Wrapping {
        language: "php",
        name: "/* */",
        open: "<?php\n/*\n",
        line: " * ",
        close: " */\n",
    },
    Wrapping {
        language: "php",
        name: "/** */",
        open: "<?php\n/**\n",
        line: " * ",
        close: " */\nfunction f() {}\n",
    },
    Wrapping {
        language: "css",
        name: "/* */",
        open: "/*\n",
        line: " * ",
        close: " */\n",
    },
    Wrapping {
        language: "html",
        name: "<!-- -->",
        open: "<!--\n",
        line: "  ",
        close: "-->\n",
    },
    Wrapping {
        language: "toml",
        name: "#",
        open: "",
        line: "# ",
        close: "key = 1\n",
    },
    Wrapping {
        language: "shellscript",
        name: "#",
        open: "",
        line: "# ",
        close: "key=1\n",
    },
    Wrapping {
        language: "yaml",
        name: "#",
        open: "",
        line: "# ",
        close: "key: 1\n",
    },
    Wrapping {
        language: "jsonc",
        name: "//",
        open: "{\n",
        line: "  // ",
        close: "}\n",
    },
    Wrapping {
        language: "git-commit",
        name: "message",
        open: "",
        line: "",
        close: "",
    },
    Wrapping {
        language: "lua",
        name: "--",
        open: "",
        line: "-- ",
        close: "",
    },
    Wrapping {
        language: "lua",
        name: "---",
        open: "",
        line: "--- ",
        close: "",
    },
    Wrapping {
        language: "lua",
        name: "--[[ ]]",
        open: "--[[\n",
        line: "",
        close: "]]\n",
    },
    Wrapping {
        language: "scss",
        name: "/* */",
        open: "/*\n",
        line: " * ",
        close: " */\n",
    },
    Wrapping {
        language: "scss",
        name: "//",
        open: "",
        line: "// ",
        close: ".a { color: red; }\n",
    },
    Wrapping {
        language: "less",
        name: "/* */",
        open: "/*\n",
        line: " * ",
        close: " */\n",
    },
    Wrapping {
        language: "less",
        name: "//",
        open: "",
        line: "// ",
        close: ".a { color: red; }\n",
    },
    Wrapping {
        language: "kotlin",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "kotlin",
        name: "/* */",
        open: "/*\n",
        line: "",
        close: "*/\n",
    },
    Wrapping {
        language: "kotlin",
        name: "/** */",
        open: "/**\n",
        line: " * ",
        close: " */\nfun f() {}\n",
    },
    Wrapping {
        language: "java",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "java",
        name: "/** */",
        open: "/**\n",
        line: " * ",
        close: " */\nclass C {}\n",
    },
    Wrapping {
        language: "swift",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "swift",
        name: "///",
        open: "",
        line: "/// ",
        close: "struct S {}\n",
    },
    Wrapping {
        language: "swift",
        name: "/* */",
        open: "/*\n",
        line: "",
        close: "*/\n",
    },
    Wrapping {
        language: "swift",
        name: "/** */",
        open: "/**\n",
        line: " * ",
        close: " */\nstruct S {}\n",
    },
    Wrapping {
        language: "c",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "c",
        name: "/* */",
        open: "/*\n",
        line: " * ",
        close: " */\n",
    },
    Wrapping {
        language: "c",
        name: "/** */",
        open: "/**\n",
        line: " * ",
        close: " */\nint f(void);\n",
    },
    Wrapping {
        language: "c",
        name: "///",
        open: "",
        line: "/// ",
        close: "int f(void);\n",
    },
    Wrapping {
        language: "cpp",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "cpp",
        name: "/* */",
        open: "/*\n",
        line: " * ",
        close: " */\n",
    },
    Wrapping {
        language: "cpp",
        name: "/** */",
        open: "/**\n",
        line: " * ",
        close: " */\nint f();\n",
    },
    Wrapping {
        language: "cpp",
        name: "///",
        open: "",
        line: "/// ",
        close: "int f();\n",
    },
    Wrapping {
        language: "ruby",
        name: "#",
        open: "",
        line: "# ",
        close: "key = 1\n",
    },
    Wrapping {
        language: "ruby",
        name: "=begin =end",
        open: "=begin\n",
        line: "",
        close: "=end\n",
    },
    Wrapping {
        language: "vue",
        name: "<!-- -->",
        open: "<!--\n",
        line: "  ",
        close: "-->\n",
    },
    Wrapping {
        language: "svelte",
        name: "<!-- -->",
        open: "<!--\n",
        line: "  ",
        close: "-->\n",
    },
    Wrapping {
        language: "scala",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "scala",
        name: "/* */",
        open: "/*\n",
        line: "",
        close: "*/\n",
    },
    Wrapping {
        language: "scala",
        name: "/** */",
        open: "/**\n",
        line: " * ",
        close: " */\nobject O\n",
    },
    Wrapping {
        language: "groovy",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "groovy",
        name: "/* */",
        open: "/*\n",
        line: "",
        close: "*/\n",
    },
    Wrapping {
        language: "groovy",
        name: "/** */",
        open: "/**\n",
        line: " * ",
        close: " */\nclass C {}\n",
    },
    Wrapping {
        language: "csharp",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "csharp",
        name: "/* */",
        open: "/*\n",
        line: " * ",
        close: " */\nclass C {}\n",
    },
    Wrapping {
        language: "csharp",
        name: "///",
        open: "",
        line: "/// ",
        close: "class C {}\n",
    },
    Wrapping {
        language: "fsharp",
        name: "//",
        open: "",
        line: "// ",
        close: "",
    },
    Wrapping {
        language: "fsharp",
        name: "(* *)",
        open: "(*\n",
        line: "",
        close: "*)\nlet x = 1\n",
    },
    Wrapping {
        language: "fsharp",
        name: "///",
        open: "",
        line: "/// ",
        close: "let x = 1\n",
    },
];

/// What `language` finds in `text`: each finding's words and check, in order.
fn found(engine: &mut Engine, language: &str, text: &str) -> String {
    engine
        .check(language, text)
        .iter()
        .map(|finding| format!("{} {}\n", &text[finding.range], finding.code.0))
        .collect()
}

#[test]
fn every_host_reads_the_same_prose_the_same_way() {
    let mut engine = Engine::with_packs(packs());
    let plain: String = BODY.iter().map(|line| format!("{line}\n")).collect();
    let expected = found(&mut engine, "plaintext", &plain);
    expect![[r#"
        Seperates known-typo
        teh known-typo
        occured known-typo
        qzxv unknown-word
        сount mixed-script
        recieve known-typo
    "#]]
    .assert_eq(&expected);

    let differing: Vec<String> = WRAPPINGS
        .iter()
        .filter_map(|wrapping| {
            let mut text = wrapping.open.to_owned();
            for line in BODY {
                text.push_str(wrapping.line);
                text.push_str(line);
                text.push('\n');
            }
            text.push_str(wrapping.close);
            let got = found(&mut engine, wrapping.language, &text);
            (got != expected).then(|| format!("{} {}:\n{got}", wrapping.language, wrapping.name))
        })
        .collect();
    assert!(differing.is_empty(), "\n{}", differing.join("\n"));
}
