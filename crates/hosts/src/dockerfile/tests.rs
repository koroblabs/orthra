use expect_test::expect;

use crate::tests::check;

#[test]
fn dockerfile_comments_label_prose_and_declared_names_are_prose() {
    check(
        "/Dockerfile",
        r#"# syntax=docker/dockerfile:1
# escape=\
# A comment here
ARG VERSION=1.0
FROM golang:${VERSION} AS builder
LABEL org.opencontainers.image.description="Does thing wrold" maintainer="me"
LABEL description="Another one"
ENV FOO=bar BAZ="qux quux"
WORKDIR /app
"#,
        expect![[r#"
            dockerfile
            > # syntax=docker/dockerfile:1
            > # escape=\
            > # A comment here
              ^^^^^^^^^^^^^^^^ comment
            > ARG VERSION=1.0
                  ^^^^^^^ declaration
                          ^^^ value
            > FROM golang:${VERSION} AS builder
                                        ^^^^^^^ declaration
            > LABEL org.opencontainers.image.description="Does thing wrold" maintainer="me"
                                                         ^^^^^^^^^^^^^^^^^^ string
            > LABEL description="Another one"
                                ^^^^^^^^^^^^^ string
            > ENV FOO=bar BAZ="qux quux"
                  ^^^ declaration
                      ^^^ value
                          ^^^ declaration
                              ^^^^^^^^^^ value
            > WORKDIR /app
        "#]],
    );
}

#[test]
fn dockerfile_shell_instructions_are_not_prose() {
    check(
        "/images/web.Dockerfile",
        r#"FROM scratch
RUN go build -o /out \
    && echo "teh thing"
COPY --from=builder /out /out
EXPOSE 8080
CMD ["/out", "--teh"]
ENTRYPOINT /out teh
HEALTHCHECK --interval=5s CMD curl -f http://localhost/
"#,
        expect![[r#"
            dockerfile
            > FROM scratch
            > RUN go build -o /out \
            >     && echo "teh thing"
            > COPY --from=builder /out /out
            > EXPOSE 8080
            > CMD ["/out", "--teh"]
            > ENTRYPOINT /out teh
            > HEALTHCHECK --interval=5s CMD curl -f http://localhost/
        "#]],
    );
}

#[test]
fn dockerfile_is_claimed_by_name_pattern_and_extension() {
    let hosts = crate::Hosts::builtin();
    let claimed: Vec<String> = [
        "Dockerfile",
        "Containerfile",
        "Dockerfile.dev",
        "web.Dockerfile",
        "app.dockerfile",
        "docker/Dockerfile.alpine",
        "Dockerfile.md",
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
        dockerfile Dockerfile
        dockerfile Containerfile
        dockerfile Dockerfile.dev
        dockerfile web.Dockerfile
        dockerfile app.dockerfile
        dockerfile docker/Dockerfile.alpine
        dockerfile Dockerfile.md
    "#]]
    .assert_eq(&(claimed.join("\n") + "\n"));
}
