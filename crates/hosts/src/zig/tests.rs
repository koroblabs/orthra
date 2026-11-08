use expect_test::expect;

use crate::tests::check;

#[test]
fn zig_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/greet.zig",
        r##"//! Container docs.
const std = @import("std");
const Allocator = std.mem.Allocator;

/// A greeter.
pub const Greeter = struct {
    name: []const u8,
    count: u32 = 0,

    pub const max_count = 4;

    /// Says hello.
    pub fn greet(self: *Greeter, other: []const u8) !void {
        // A comment.
        //// not a doc
        const message = "Hello\nworld {s}";
        var index: usize = 0;
        const multi =
            \\multi line
            \\text here
        ;
        std.debug.print("recieve {s}\n", .{other});
        _ = message; _ = multi; _ = index;
    }
};

const Color = enum { red, green };
const Tag = union(enum) { a: i32, b };
const Err = error{ OutOfMemory, Bad };

test "greets teh world" {
    const x = 1;
    _ = x;
}

fn generic(comptime T: type, value: T) T { return value; }
extern "c" fn puts(s: [*:0]const u8) c_int;
"##,
        expect![[r#"
            zig
            > //! Container docs.
                  ^^^^^^^^^^^^^^^ doccomment
            > const std = @import("std");
                    ^^^ declaration
            > const Allocator = std.mem.Allocator;
                    ^^^^^^^^^ declaration
            >
            > /// A greeter.
                  ^^^^^^^^^^ doccomment
            > pub const Greeter = struct {
                        ^^^^^^^ declaration
            >     name: []const u8,
                  ^^^^ declaration
            >     count: u32 = 0,
                  ^^^^^ declaration
            >
            >     pub const max_count = 4;
                            ^^^^^^^^^ declaration
            >
            >     /// Says hello.
                      ^^^^^^^^^^^ doccomment
            >     pub fn greet(self: *Greeter, other: []const u8) !void {
                         ^^^^^ declaration
                               ^^^^ declaration
                                               ^^^^^ declaration
            >         // A comment.
                      ^^^^^^^^^^^^^ comment
            >         //// not a doc
                      ^^^^^^^^^^^^^^ comment
            >         const message = "Hello\nworld {s}";
                            ^^^^^^^ declaration
                                       ^^^^^ string
                                              ^^^^^^^^^ string
            >         var index: usize = 0;
                          ^^^^^ declaration
            >         const multi =
                            ^^^^^ declaration
            >             \\multi line
                          ^^^^^^^^^^^^ value (+1 lines)
            >             \\text here
            >         ;
            >         std.debug.print("recieve {s}\n", .{other});
                                       ^^^^^^^^^^^ string
            >         _ = message; _ = multi; _ = index;
            >     }
            > };
            >
            > const Color = enum { red, green };
                    ^^^^^ declaration
                                   ^^^ declaration
                                        ^^^^^ declaration
            > const Tag = union(enum) { a: i32, b };
                    ^^^ declaration
                                        ^ declaration
                                                ^ declaration
            > const Err = error{ OutOfMemory, Bad };
                    ^^^ declaration
                                 ^^^^^^^^^^^ declaration
                                              ^^^ declaration
            >
            > test "greets teh world" {
                    ^^^^^^^^^^^^^^^^ string
            >     const x = 1;
                        ^ declaration
            >     _ = x;
            > }
            >
            > fn generic(comptime T: type, value: T) T { return value; }
                 ^^^^^^^ declaration
                                  ^ declaration
                                           ^^^^^ declaration
            > extern "c" fn puts(s: [*:0]const u8) c_int;
                            ^^^^ declaration
                                 ^ declaration
        "#]],
    );
}

/// `build.zig.zon` is Zig Object Notation: the same grammar, and the package's name is declared
/// there.
#[test]
fn zon_manifest_is_read_by_the_zig_host() {
    check(
        "/build.zig.zon",
        r##".{
    // The package's manifest.
    .name = .greeter,
    .version = "0.1.0",
    .paths = .{ "build.zig", "src" },
}
"##,
        expect![[r#"
            zig
            > .{
            >     // The package's manifest.
                  ^^^^^^^^^^^^^^^^^^^^^^^^^^ comment
            >     .name = .greeter,
            >     .version = "0.1.0",
                              ^^^^^ string
            >     .paths = .{ "build.zig", "src" },
                               ^^^^^^^^^ string
                                            ^^^ string
            > }
        "#]],
    );
}
