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

/// A handle in a comment's conventions is a name: `TODO(qzxv)`, and a code review's `CR qzxva:`
/// and `CR-someday qzxvb:`. Xcode's header names its author after `Created by`, and a copyright
/// sign, or `Copyright` with its capital anywhere in a line, is followed by the holders' names. A
/// word after
/// `TODO:` or in a sentence that starts with `Created` is still prose.
#[test]
fn a_handle_and_a_created_by_line_are_names() {
    check(
        "/src/main.c",
        r#"//  Created by Qzxvc Wrtyp on 9/21/09.
//  (c) Qzxvh Wrtypb
// Written by: Qzxvd Wrtypa
// TODO(qzxv): split this.
// CR qzxva: rename it.
// CR-someday qzxvb: and this.
// Original image Copyright Qzxvi Wrtypc
// A copyright qzxvj is prose.
// TODO: qzxve is a finding, and so is (qzxvf).
// Created qzxvg here.
int main(void) { return 0; }
"#,
        expect![[r#"
            > //  Created by Qzxvc Wrtyp on 9/21/09.
            > //  (c) Qzxvh Wrtypb
            > // Written by: Qzxvd Wrtypa
            > // TODO(qzxv): split this.
            > // CR qzxva: rename it.
            > // CR-someday qzxvb: and this.
            > // Original image Copyright Qzxvi Wrtypc
            > // A copyright qzxvj is prose.
                             ^^^^^ unknown-word
            > // TODO: qzxve is a finding, and so is (qzxvf).
                       ^^^^^ unknown-word
                                                      ^^^^^ unknown-word
            > // Created qzxvg here.
                         ^^^^^ unknown-word
            > int main(void) { return 0; }
        "#]],
    );
}
