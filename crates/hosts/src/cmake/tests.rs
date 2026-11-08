use expect_test::expect;

use crate::tests::check;

#[test]
fn cmake_comments_messages_docs_and_declared_names_are_prose() {
    check(
        "/CMakeLists.txt",
        r##"# A comment
#[[ A block
comment ]]
cmake_minimum_required(VERSION 3.10)
project(foo LANGUAGES CXX DESCRIPTION "Builds the thing")
option(FOO_BUILD "Build teh thing" ON)
set(MY_VAR "value" CACHE STRING "Doc for var")
set(CMAKE_CXX_STANDARD 17)
message(STATUS "Hello wrold ${MY_VAR}\n")
MESSAGE(FATAL_ERROR "Oops")
function(my_func arg1 arg2)
  message("in func")
endfunction()
macro(my_macro)
endmacro()
add_custom_target(docs COMMENT "Generating docs")
"##,
        expect![[r#"
            cmake
            > # A comment
              ^^^^^^^^^^^ comment
            > #[[ A block
              ^^^^^^^^^^^ comment (+1 lines)
            > comment ]]
            > cmake_minimum_required(VERSION 3.10)
            > project(foo LANGUAGES CXX DESCRIPTION "Builds the thing")
                                                     ^^^^^^^^^^^^^^^^ string
            > option(FOO_BUILD "Build teh thing" ON)
                     ^^^^^^^^^ declaration
                                ^^^^^^^^^^^^^^^ string
            > set(MY_VAR "value" CACHE STRING "Doc for var")
                  ^^^^^^ declaration
                                               ^^^^^^^^^^^ string
            > set(CMAKE_CXX_STANDARD 17)
            > message(STATUS "Hello wrold ${MY_VAR}\n")
                              ^^^^^^^^^^^^^^^^^^^^^^^ string
            > MESSAGE(FATAL_ERROR "Oops")
                                   ^^^^ string
            > function(my_func arg1 arg2)
                       ^^^^^^^ declaration
                               ^^^^ declaration
                                    ^^^^ declaration
            >   message("in func")
                         ^^^^^^^ string
            > endfunction()
            > macro(my_macro)
                    ^^^^^^^^ declaration
            > endmacro()
            > add_custom_target(docs COMMENT "Generating docs")
                                              ^^^^^^^^^^^^^^^ string
        "#]],
    );
}

#[test]
fn cmake_other_arguments_are_not_prose() {
    check(
        "/cmake/Modules/FindFoo.cmake",
        r#"add_library(foo STATIC src/teh.cpp)
target_compile_options(foo PRIVATE "-Wteh")
if(FOO AND NOT BAR)
  set(X 1)
endif()
foreach(item IN LISTS ITEMS)
endforeach()
message(STATUS [[bracket arg]])
"#,
        expect![[r#"
            cmake
            > add_library(foo STATIC src/teh.cpp)
            > target_compile_options(foo PRIVATE "-Wteh")
            > if(FOO AND NOT BAR)
            >   set(X 1)
                    ^ declaration
            > endif()
            > foreach(item IN LISTS ITEMS)
            > endforeach()
            > message(STATUS [[bracket arg]])
        "#]],
    );
}

#[test]
fn cmakelists_goes_to_cmake_and_not_to_plain_text() {
    let hosts = crate::Hosts::builtin();
    let claimed: Vec<String> = [
        "CMakeLists.txt",
        "sub/CMakeLists.txt",
        "cmake/Foo.cmake",
        "notes.txt",
        "CMakeCache.txt",
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
        cmake      CMakeLists.txt
        cmake      sub/CMakeLists.txt
        cmake      cmake/Foo.cmake
        plain-text notes.txt
        plain-text CMakeCache.txt
    "#]]
    .assert_eq(&(claimed.join("\n") + "\n"));
}
