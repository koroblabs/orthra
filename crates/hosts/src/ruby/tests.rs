use expect_test::expect;

use crate::tests::check;

#[test]
fn ruby_prose_is_comments_strings_and_declared_names() {
    check(
        "/lib/greeter.rb",
        r##"#!/usr/bin/env ruby
# frozen_string_literal: true

require "greeter/seperate"
require_relative "teh_helper"

=begin
Seperates the block comments.
=end

# Greets teh world.
#   Greeter.new("x").greet # an example, Seperates
# :nodoc:
module Greeting
  GREETING_LANGUAGE = "en"

  class Greeter < Base
    attr_reader :recieve, :other
    attr_accessor :occured

    # Says hello.
    # @param name [String] who recieve it
    # @param opts [Hash] the Seperates options
    # @return [Array<String>] teh lines
    # @raise [ArgumentError] when empty
    # @author Somebody Unknownnn
    def greet(name, greeting = "Hello", *rest, key: "default", **opts, &block)
      text = "Hello\nworld #{name} occured"
      first, second = rest
      count ||= 1
      puts text, :recieve, %w[recieve occured], %i[recieve]
      @recieve = text
    end

    def self.build(thing) = new(thing)
    def name=(value); end
    def ==(other); end
  end
end

items.each { |item, index| puts item }
add = ->(left, right) { left + right }
puts <<~TEXT
  Seperates teh heredoc.
TEXT
"##,
        expect![[r#"
            ruby
            > #!/usr/bin/env ruby
            > # frozen_string_literal: true
            >
            > require "greeter/seperate"
            > require_relative "teh_helper"
            >
            > =begin
              ^^^^^^ comment (+2 lines)
            > Seperates the block comments.
            > =end
            >
            > # Greets teh world.
              ^^^^^^^^^^^^^^^^^^^ comment
            > #   Greeter.new("x").greet # an example, Seperates
            > # :nodoc:
            > module Greeting
                     ^^^^^^^^ declaration
            >   GREETING_LANGUAGE = "en"
                ^^^^^^^^^^^^^^^^^ declaration
                                     ^^ value
            >
            >   class Greeter < Base
                      ^^^^^^^ declaration
            >     attr_reader :recieve, :other
                               ^^^^^^^ declaration
                                         ^^^^^ declaration
            >     attr_accessor :occured
                                 ^^^^^^^ declaration
            >
            >     # Says hello.
                  ^^^^^^^^^^^^^ comment
            >     # @param name [String] who recieve it
                                ^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >     # @param opts [Hash] the Seperates options
                                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >     # @return [Array<String>] teh lines
                                            ^^^^^^^^^ doccomment
            >     # @raise [ArgumentError] when empty
                                           ^^^^^^^^^^ doccomment
            >     # @author Somebody Unknownnn
            >     def greet(name, greeting = "Hello", *rest, key: "default", **opts, &block)
                      ^^^^^ declaration
                            ^^^^ declaration
                                  ^^^^^^^^ declaration
                                              ^^^^^ value
                                                       ^^^^ declaration
                                                             ^^^ declaration
                                                                   ^^^^^^^ value
                                                                               ^^^^ declaration
                                                                                      ^^^^^ declaration
            >       text = "Hello\nworld #{name} occured"
                    ^^^^ declaration
                            ^^^^^ value
                                   ^^^^^ value
                                                 ^^^^^^^ value
            >       first, second = rest
                    ^^^^^ declaration
                           ^^^^^^ declaration
            >       count ||= 1
                    ^^^^^ declaration
            >       puts text, :recieve, %w[recieve occured], %i[recieve]
            >       @recieve = text
            >     end
            >
            >     def self.build(thing) = new(thing)
                           ^^^^^ declaration
                                 ^^^^^ declaration
            >     def name=(value); end
                      ^^^^ declaration
                            ^^^^^ declaration
            >     def ==(other); end
                         ^^^^^ declaration
            >   end
            > end
            >
            > items.each { |item, index| puts item }
                            ^^^^ declaration
                                  ^^^^^ declaration
            > add = ->(left, right) { left + right }
              ^^^ declaration
                       ^^^^ declaration
                             ^^^^^ declaration
            > puts <<~TEXT
            >   Seperates teh heredoc.
            > TEXT
        "#]],
    );
}

#[test]
fn ruby_one_token_strings_are_values() {
    check(
        "/lib/headers.rb",
        r#"env["HTTP_HOST"] = "recieve"
headers = { "Content-Type" => "text/html", name: "occured" }
puts "Hello, world", 'single quoted text'
"#,
        expect![[r#"
            ruby
            > env["HTTP_HOST"] = "recieve"
                   ^^^^^^^^^ value
                                  ^^^^^^^ value
            > headers = { "Content-Type" => "text/html", name: "occured" }
              ^^^^^^^ declaration
                           ^^^^^^^^^^^^ value
                                             ^^^^^^^^^ value
                                                                ^^^^^^^ value
            > puts "Hello, world", 'single quoted text'
                    ^^^^^^^^^^^^ string
                                    ^^^^^^^^^^^^^^^^^^ string
        "#]],
    );
}
