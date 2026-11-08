use expect_test::expect;

use crate::tests::check;

#[test]
fn makefile_comments_messages_and_defined_variables_are_prose() {
    check(
        "/src/Makefile",
        r#"# Builds the thing.
CC := gcc
OBJS = main.o util.o
export FOO ?= bar
override CFLAGS += -O2 -Wall

define my-rule
 echo hi
endef

$(info Hello wrold from $(CC))
$(error Missing thing)
ifeq ($(X),yes)
$(warning "Quoted text")
endif
"#,
        expect![[r#"
            makefile
            > # Builds the thing.
              ^^^^^^^^^^^^^^^^^^^ comment
            > CC := gcc
              ^^ declaration
            > OBJS = main.o util.o
              ^^^^ declaration
            > export FOO ?= bar
                     ^^^ declaration
            > override CFLAGS += -O2 -Wall
                       ^^^^^^ declaration
            >
            > define my-rule
                     ^^^^^^^ declaration
            >  echo hi
            > endef
            >
            > $(info Hello wrold from $(CC))
                     ^^^^^^^^^^^^^^^^^^^^^^ string
            > $(error Missing thing)
                      ^^^^^^^^^^^^^ string
            > ifeq ($(X),yes)
            > $(warning "Quoted text")
                        ^^^^^^^^^^^^^ string
            > endif
        "#]],
    );
}

#[test]
fn makefile_targets_and_recipes_are_not_prose() {
    check(
        "/src/GNUmakefile",
        "\
.PHONY: all clean teh
all: $(OBJS) | dir
\t$(CC) -o $@ $^
\t@echo \"building teh thing\"
%.o: %.c
\t$(CC) -c $<
foo := $(shell echo hi)
",
        expect![[r#"
            makefile
            > .PHONY: all clean teh
            > all: $(OBJS) | dir
            > 	$(CC) -o $@ $^
            > 	@echo "building teh thing"
            > %.o: %.c
            > 	$(CC) -c $<
            > foo := $(shell echo hi)
              ^^^ declaration
        "#]],
    );
}

#[test]
fn makefile_is_claimed_by_name_and_extension() {
    let hosts = crate::Hosts::builtin();
    let claimed: Vec<String> = [
        "Makefile",
        "makefile",
        "GNUmakefile",
        "Makefile.in.local",
        "scripts/Makefile.build",
        "rules.mk",
        "rules.mak",
        "Makefile.md",
    ]
    .iter()
    .map(|path| {
        format!(
            "{:<11}{path}",
            hosts.for_path(path).map_or("-", |host| host.id())
        )
    })
    .collect();
    expect![[r#"
        makefile   Makefile
        makefile   makefile
        makefile   GNUmakefile
        makefile   Makefile.in.local
        makefile   scripts/Makefile.build
        makefile   rules.mk
        makefile   rules.mak
        makefile   Makefile.md
    "#]]
    .assert_eq(&(claimed.join("\n") + "\n"));
}
