use expect_test::{Expect, expect};
use orthra_testkit::annotate::Annotation;

use crate::Engine;

fn check(path: &str, text: &str, expect: Expect) {
    let mut engine = Engine::new(&["en-US"]).unwrap();
    let findings = engine.check_file(path, text);
    let annotations: Vec<Annotation> = findings
        .iter()
        .map(|finding| Annotation::new(finding.range, finding.code.0))
        .collect();
    expect.assert_eq(&Annotation::render(text, &annotations));
}

#[test]
fn a_comment_line_labeled_with_people_is_names() {
    check(
        "/themes/dusk.toml",
        r#"# Author: Matous Dzivjak <matousdzivjak@gmail.com>
# Original author : rebelot
# Maintainers: Qzxv Wrtyp, Jens Getreu
#   Authors: listed below, Qzxva
# Thanks to Qzxvb for the palette.
# Note: qzxvc is still a finding.
name = "Author: Qzxvd"
"#,
        expect![[r#"
            > # Author: Matous Dzivjak <matousdzivjak@gmail.com>
            > # Original author : rebelot
            > # Maintainers: Qzxv Wrtyp, Jens Getreu
            > #   Authors: listed below, Qzxva
            > # Thanks to Qzxvb for the palette.
                          ^^^^^ unknown-word
            > # Note: qzxvc is still a finding.
                      ^^^^^ unknown-word
            > name = "Author: Qzxvd"
                              ^^^^^ unknown-word
        "#]],
    );
}

/// A copyright line names its holders without a colon, and a line of a few words and an email in
/// angle brackets is a name whatever the language. A sentence that mentions an email is still
/// prose, and so is a line about copyright that does not start with the word.
#[test]
fn a_copyright_line_and_a_name_with_an_email_are_names() {
    check(
        "/src/server.c",
        r#"/*
 * Copyright (c) 2009-2012, Qzxva Sanfilippo <qzxva at gmail dot com>
 * Copyright 2014 Qzxvb Noordhuis
 *   qzxvc <qzxvc@example.com>
 *   Qzxvd Qzxve <qzxvd@example.com>
 * Mail qzxvf at <bugs@example.com> when the build breaks.
 * The copyright qzxvg is in LICENSE.
 */
int main(void) { return 0; }
"#,
        expect![[r#"
            > /*
            >  * Copyright (c) 2009-2012, Qzxva Sanfilippo <qzxva at gmail dot com>
            >  * Copyright 2014 Qzxvb Noordhuis
            >  *   qzxvc <qzxvc@example.com>
            >  *   Qzxvd Qzxve <qzxvd@example.com>
            >  * Mail qzxvf at <bugs@example.com> when the build breaks.
                      ^^^^^ unknown-word
            >  * The copyright qzxvg is in LICENSE.
                               ^^^^^ unknown-word
            >  */
            > int main(void) { return 0; }
        "#]],
    );
}

/// A license file is prose, and its copyright line names people as a comment's does.
#[test]
fn a_license_files_copyright_line_is_names() {
    check(
        "/LICENSE",
        "The MIT License (MIT)\n\nCopyright (c) 2016 Qzxva Brunsfeld\n\nPermission is hereby granted, qzxvb.\n",
        expect![[r#"
            > The MIT License (MIT)
            >
            > Copyright (c) 2016 Qzxva Brunsfeld
            >
            > Permission is hereby granted, qzxvb.
                                            ^^^^^ unknown-word
        "#]],
    );
}
