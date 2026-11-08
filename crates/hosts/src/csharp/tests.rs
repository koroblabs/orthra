use expect_test::expect;

use crate::tests::check;

#[test]
fn csharp_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/Greeter.cs",
        r#"using System;
using static System.Math;
namespace My.Namespace;

/// <summary>
/// Greets whoever <paramref name="person"/> is. See <see cref="Other"/>.
/// </summary>
/// <param name="person">The person to greet.</param>
[Obsolete("old message")]
public class Greeter<TItem> : IGreeter where TItem : class
{
    private int count = 0;
    public string Name { get; set; }
    public event EventHandler Changed;
    // line comment
    /* block */
    public Greeter(string name) { Name = name; }
    public string Greet(string person, int times = 1)
    {
        var message = "Hello\nworld";
        string verb = @"verb\batim ""x""";
        string raw = """raw text""";
        string interp = $"Hi {person}, you are {times:N2} old";
        foreach (var item in new[] { 1 }) { int inner = item; }
        void Local(int a) {}
        try {} catch (Exception ex) {}
        var (p, q) = (1, 2);
        return Console.WriteLine(message);
    }
    ~Greeter() {}
}
public enum Color { Red, Green = 2 }
public interface IGreeter { void Hello(); }
public record Point(int X, int Y);
public delegate void Handler(int arg);
"#,
        expect![[r#"
            csharp
            > using System;
            > using static System.Math;
            > namespace My.Namespace;
            >
            > /// <summary>
                  ^^^^^^^^^ doccomment
            > /// Greets whoever <paramref name="person"/> is. See <see cref="Other"/>.
                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > /// </summary>
                  ^^^^^^^^^^ doccomment
            > /// <param name="person">The person to greet.</param>
                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            > [Obsolete("old message")]
            > public class Greeter<TItem> : IGreeter where TItem : class
                           ^^^^^^^ declaration
                                   ^^^^^ declaration
            > {
            >     private int count = 0;
                              ^^^^^ declaration
            >     public string Name { get; set; }
                                ^^^^ declaration
            >     public event EventHandler Changed;
                                            ^^^^^^^ declaration
            >     // line comment
                  ^^^^^^^^^^^^^^^ comment
            >     /* block */
                  ^^^^^^^^^^^ comment
            >     public Greeter(string name) { Name = name; }
                                        ^^^^ declaration
            >     public string Greet(string person, int times = 1)
                                ^^^^^ declaration
                                             ^^^^^^ declaration
                                                         ^^^^^ declaration
            >     {
            >         var message = "Hello\nworld";
                          ^^^^^^^ declaration
                                     ^^^^^ value
                                            ^^^^^ value
            >         string verb = @"verb\batim ""x""";
                             ^^^^ declaration
                                    ^^^^^^^^^^^^^^^^^^^ string
            >         string raw = """raw text""";
                             ^^^ declaration
                                      ^^^^^^^^ string
            >         string interp = $"Hi {person}, you are {times:N2} old";
                             ^^^^^^ declaration
                                        ^^ value
                                                   ^^^^^^^^^ string
                                                                        ^^^ value
            >         foreach (var item in new[] { 1 }) { int inner = item; }
                                   ^^^^ declaration
                                                              ^^^^^ declaration
            >         void Local(int a) {}
                           ^^^^^ declaration
                                     ^ declaration
            >         try {} catch (Exception ex) {}
                                              ^^ declaration
            >         var (p, q) = (1, 2);
                           ^ declaration
                              ^ declaration
            >         return Console.WriteLine(message);
            >     }
            >     ~Greeter() {}
            > }
            > public enum Color { Red, Green = 2 }
                          ^^^^^ declaration
                                  ^^^ declaration
                                       ^^^^^ declaration
            > public interface IGreeter { void Hello(); }
                               ^^^^^^^^ declaration
                                               ^^^^^ declaration
            > public record Point(int X, int Y);
                            ^^^^^ declaration
                                      ^ declaration
                                             ^ declaration
            > public delegate void Handler(int arg);
                                   ^^^^^^^ declaration
                                               ^^^ declaration
        "#]],
    );
}

#[test]
fn csharp_attributes_are_skipped_and_one_token_strings_are_values() {
    check(
        "/src/Plain.cs",
        r#"[assembly: InternalsVisibleTo("Acme.Widgets.Schema")]
[JsonProperty("user_name")]
public string Key = "yyyy'-'MM'-'dd";
public string Greeting = "Hello world";
"#,
        expect![[r#"
            csharp
            > [assembly: InternalsVisibleTo("Acme.Widgets.Schema")]
            > [JsonProperty("user_name")]
            > public string Key = "yyyy'-'MM'-'dd";
                                   ^^^^^^^^^^^^^^ value
            > public string Greeting = "Hello world";
                            ^^^^^^^^ declaration
                                        ^^^^^^^^^^^ string
        "#]],
    );
}
