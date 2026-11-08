use expect_test::expect;

use crate::tests::check;

#[test]
fn proto_prose_is_comments_and_declared_names() {
    check(
        "/api/v1/greeter.proto",
        r#"// Greeter service.
syntax = "proto3";

package greeter.v1;

import "google/api/annotations.proto";

option go_package = "example.com/greeter/v1;greeterv1";

/* A greeting request. */
message HelloRequest {
  // Who is greeted.
  string user_name = 1;
  map<string, string> label_map = 2;
  oneof shape_kind {
    string plain_text = 3;
  }
}

enum Status {
  STATUS_UNSPECIFIED = 0;
  STATUS_OK = 1;
}

service Greeter {
  rpc SayHello(HelloRequest) returns (HelloReply) {
    option (google.api.http) = {
      get: "/v1/hello/{name}"
    };
  }
}
"#,
        expect![[r#"
            proto
            > // Greeter service.
              ^^^^^^^^^^^^^^^^^^^ comment
            > syntax = "proto3";
            >
            > package greeter.v1;
                      ^^^^^^^ declaration
                              ^^ declaration
            >
            > import "google/api/annotations.proto";
            >
            > option go_package = "example.com/greeter/v1;greeterv1";
            >
            > /* A greeting request. */
              ^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            > message HelloRequest {
                      ^^^^^^^^^^^^ declaration
            >   // Who is greeted.
                ^^^^^^^^^^^^^^^^^^ comment
            >   string user_name = 1;
                       ^^^^^^^^^ declaration
            >   map<string, string> label_map = 2;
                                    ^^^^^^^^^ declaration
            >   oneof shape_kind {
                      ^^^^^^^^^^ declaration
            >     string plain_text = 3;
                         ^^^^^^^^^^ declaration
            >   }
            > }
            >
            > enum Status {
                   ^^^^^^ declaration
            >   STATUS_UNSPECIFIED = 0;
                ^^^^^^^^^^^^^^^^^^ declaration
            >   STATUS_OK = 1;
                ^^^^^^^^^ declaration
            > }
            >
            > service Greeter {
                      ^^^^^^^ declaration
            >   rpc SayHello(HelloRequest) returns (HelloReply) {
                    ^^^^^^^^ declaration
            >     option (google.api.http) = {
            >       get: "/v1/hello/{name}"
                         ^^^^^^^^^^^^^^^^^^ value
            >     };
            >   }
            > }
        "#]],
    );
}
