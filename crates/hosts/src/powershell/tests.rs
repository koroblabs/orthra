use expect_test::expect;

use crate::tests::check;

#[test]
fn powershell_prose_is_comments_strings_and_declared_names() {
    check(
        "/scripts/Get-Thing.ps1",
        r##"<#
.SYNOPSIS
  Gets the thing.
#>
#Requires -Version 5.1
# A line comment.
function Get-Thing {
  param(
    [Parameter(Mandatory)] [string]$Name,
    $Other = 'x'
  )
  $result = Get-Item -Path "C:\network\path"
  $message = "Hello world, $Name"
  $newline = "Hello`nworld, friend"
  $plain = "Hello world, friend"
  $raw = 'It is verbatim text'
  $here = @"
Here text $Name
"@
  Write-Output $result
}
class Widget : Base {
  [string]$Label
  [void] Render([int]$width) { }
}
enum Color { Red; Blue = 2 }
"##,
        expect![[r#"
            powershell
            > <#
              ^^ comment (+3 lines)
            > .SYNOPSIS
            >   Gets the thing.
            > #>
            > #Requires -Version 5.1
            > # A line comment.
              ^^^^^^^^^^^^^^^^^ comment
            > function Get-Thing {
                       ^^^^^^^^^ declaration
            >   param(
            >     [Parameter(Mandatory)] [string]$Name,
                                                  ^^^^ declaration
            >     $Other = 'x'
                   ^^^^^ declaration
                           ^^^ value
            >   )
            >   $result = Get-Item -Path "C:\network\path"
                 ^^^^^^ declaration
                                         ^^^^^^^^^^^^^^^^^ string
            >   $message = "Hello world, $Name"
                 ^^^^^^^ declaration
                           ^^^^^^^^^^^^^^^^^^^^ value
            >   $newline = "Hello`nworld, friend"
                 ^^^^^^^ declaration
                           ^^^^^^^^^^^^^^^^^^^^^^ value
            >   $plain = "Hello world, friend"
                 ^^^^^ declaration
                         ^^^^^^^^^^^^^^^^^^^^^ string
            >   $raw = 'It is verbatim text'
                 ^^^ declaration
                       ^^^^^^^^^^^^^^^^^^^^^ string
            >   $here = @"
                 ^^^^ declaration
                        ^^ value (+2 lines)
            > Here text $Name
            > "@
            >   Write-Output $result
            > }
            > class Widget : Base {
                    ^^^^^^ declaration
            >   [string]$Label
                         ^^^^^ declaration
            >   [void] Render([int]$width) { }
                       ^^^^^^ declaration
                                    ^^^^^ declaration
            > }
            > enum Color { Red; Blue = 2 }
                   ^^^^^ declaration
                           ^^^ declaration
                                ^^^^ declaration
        "#]],
    );
}
