use expect_test::expect;

use crate::tests::check;

#[test]
fn julia_prose_is_comments_docstrings_strings_and_declared_names() {
    check(
        "/src/greet.jl",
        r##"# A line comment.
#= A block
comment =#

"""
    greet(person)

Builds the greeting.
"""
function greet(person::String, count::Int = 1; flag = true) where {T}
    msg = "Hello\nworld, $person"
    pattern = r"not text"
    return msg
end

"Docs for the point."
struct Point{T} <: Shape
    x::T
    y
end

const LIMIT = 5
short(a, b) = a + b
Base.show(io::IO, p::Point) = print(io, "kept")
"##,
        expect![[r#"
            julia
            > # A line comment.
              ^^^^^^^^^^^^^^^^^ comment
            > #= A block
              ^^^^^^^^^^ comment (+1 lines)
            > comment =#
            >
            > """
            >     greet(person)
                  ^^^^^^^^^^^^^ doccomment
            >
            > Builds the greeting.
              ^^^^^^^^^^^^^^^^^^^^ doccomment
            > """
            > function greet(person::String, count::Int = 1; flag = true) where {T}
                       ^^^^^ declaration
            >     msg = "Hello\nworld, $person"
                  ^^^ declaration
                         ^^^^^ value
                                ^^^^^^ value
            >     pattern = r"not text"
                  ^^^^^^^ declaration
            >     return msg
            > end
            >
            > "Docs for the point."
               ^^^^^^^^^^^^^^^^^^^ doccomment
            > struct Point{T} <: Shape
            >     x::T
                  ^ declaration
            >     y
                  ^ declaration
            > end
            >
            > const LIMIT = 5
                    ^^^^^ declaration
            > short(a, b) = a + b
              ^^^^^ declaration
            > Base.show(io::IO, p::Point) = print(io, "kept")
                                                       ^^^^ value
        "#]],
    );
}

#[test]
fn julia_math_is_not_prose() {
    check(
        "/src/math.jl",
        r##"
"The density is ``\\mathbf{x}`` here." # kept
pdf(d, x) = exp(d.κ * αsum)
αsum = 1
"##,
        expect![[r#"
            julia
            >
            > "The density is ``\\mathbf{x}`` here." # kept
               ^^^^^^^^^^^^^^^^^ doccomment
                                                     ^^^^^^ comment
            > pdf(d, x) = exp(d.κ * αsum)
              ^^^ declaration
            > αsum = 1
        "#]],
    );
}

/// Names in any script, with Julia's `!`, and the forms that hold no plain name: a qualified
/// function, an operator, a tuple.
#[test]
fn julia_declares_plain_names_in_any_script() {
    crate::tests::check(
        "/src/names.jl",
        "function résumé!(θ, κ::Int, λ = 1)\n    θ\nend\nBase.show(io) = io\n+(a, b) = a\nα = 1\nfor (i, j) in pairs\nend\nfor β in 1:3\nend\nstruct Точка{T}\n    ширина::T\nend\nmodule Модуль end\n",
        expect_test::expect![[r#"
            julia
            > function résumé!(θ, κ::Int, λ = 1)
                       ^^^^^^^ declaration
            >     θ
            > end
            > Base.show(io) = io
            > +(a, b) = a
            > α = 1
            > for (i, j) in pairs
            > end
            > for β in 1:3
            > end
            > struct Точка{T}
                     ^^^^^ declaration
            >     ширина::T
                  ^^^^^^ declaration
            > end
            > module Модуль end
                     ^^^^^^ declaration
        "#]],
    );
}
