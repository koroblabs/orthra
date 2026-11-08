use expect_test::expect;

use crate::tests::check;

#[test]
fn lua_prose_is_comments_docs_strings_and_declared_names() {
    check(
        "/lua/greet.lua",
        r#"local json = require("lazy.core.util")
-- A line comment.
--[==[ A block
comment ]==]

--- Builds the greeting.
---@param person string The person to greet.
---@return string greeting The greeting.
function M.greet(person, ...)
  local message, count = "Hello\nworld, " .. person, 0
  M.state = "kept"
  return vim.fn.shout(message)
end

local function helper(arg) end

function obj:method() end
"#,
        expect![[r#"
            lua
            > local json = require("lazy.core.util")
                    ^^^^ declaration
            > -- A line comment.
              ^^^^^^^^^^^^^^^^^^ comment
            > --[==[ A block
              ^^^^^^^^^^^^^^ comment (+1 lines)
            > comment ]==]
            >
            > --- Builds the greeting.
                  ^^^^^^^^^^^^^^^^^^^^ doccomment
            > ---@param person string The person to greet.
                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > ---@return string greeting The greeting.
                                ^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > function M.greet(person, ...)
                         ^^^^^ declaration
                               ^^^^^^ declaration
            >   local message, count = "Hello\nworld, " .. person, 0
                      ^^^^^^^ declaration
                               ^^^^^ declaration
                                        ^^^^^^^^^^^^^ string
            >   M.state = "kept"
                           ^^^^ value
            >   return vim.fn.shout(message)
            > end
            >
            > local function helper(arg) end
                             ^^^^^^ declaration
                                    ^^^ declaration
            >
            > function obj:method() end
                           ^^^^^^ declaration
        "#]],
    );
}
