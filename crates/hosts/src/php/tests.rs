use expect_test::expect;

use crate::tests::check;

#[test]
fn php_prose_is_comments_strings_and_declared_names() {
    check(
        "/src/Greeter.php",
        r#"<?php

namespace App\Greeting;

use Psr\Log\LoggerInterface as Logger;

require_once __DIR__ . '/../vendor/autoload.php';

/**
 * Says hello to whoever asks.
 *
 * @param string $name Who to greet.
 */
final class Greeter
{
    private const PREFIX = 'Hello, ';

    private int $count = 0;

    public function __construct(private Logger $logger) {}

    // Builds the greeting.
    # A hash comment.
    /* A block comment. */
    public function greet(string $name, string ...$extra): string
    {
        $message = "Dear {$name}\nwelcome";
        $quoted = 'single quoted';
        foreach ($extra as $index => $item) {
            $this->logger->info(<<<TEXT
            Heredoc text for $item
            TEXT);
        }
        $raw = <<<'RAW'
        Nowdoc text
        RAW;
        $listing = `ls -la`;
        return self::PREFIX . $message . $quoted . $raw;
    }
}
"#,
        expect![[r#"
            php
            > <?php
            >
            > namespace App\Greeting;
                        ^^^ declaration
                            ^^^^^^^^ declaration
            >
            > use Psr\Log\LoggerInterface as Logger;
                                             ^^^^^^ declaration
            >
            > require_once __DIR__ . '/../vendor/autoload.php';
            >
            > /**
            >  * Says hello to whoever asks.
                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >  *
            >  * @param string $name Who to greet.
                                     ^^^^^^^^^^^^^ doccomment
            >  */
            > final class Greeter
                          ^^^^^^^ declaration
            > {
            >     private const PREFIX = 'Hello, ';
                                ^^^^^^ declaration
                                          ^^^^^^ string
            >
            >     private int $count = 0;
                               ^^^^^ declaration
            >
            >     public function __construct(private Logger $logger) {}
                                  ^^^^^^^^^^^ declaration
                                                              ^^^^^^ declaration
            >
            >     // Builds the greeting.
                  ^^^^^^^^^^^^^^^^^^^^^^^ comment
            >     # A hash comment.
                  ^^^^^^^^^^^^^^^^^ comment
            >     /* A block comment. */
                  ^^^^^^^^^^^^^^^^^^^^^^ comment
            >     public function greet(string $name, string ...$extra): string
                                  ^^^^^ declaration
                                                ^^^^ declaration
                                                                 ^^^^^ declaration
            >     {
            >         $message = "Dear {$name}\nwelcome";
                       ^^^^^^^ declaration
                                  ^^^^ string
                                                ^^^^^^^ string
            >         $quoted = 'single quoted';
                       ^^^^^^ declaration
                                 ^^^^^^^^^^^^^ string
            >         foreach ($extra as $index => $item) {
                                          ^^^^^ declaration
                                                    ^^^^ declaration
            >             $this->logger->info(<<<TEXT
            >             Heredoc text for $item
                          ^^^^^^^^^^^^^^^^ string
            >             TEXT);
            >         }
            >         $raw = <<<'RAW'
                       ^^^ declaration
            >         Nowdoc text
                      ^^^^^^^^^^^ string
            >         RAW;
            >         $listing = `ls -la`;
                       ^^^^^^^ declaration
            >         return self::PREFIX . $message . $quoted . $raw;
            >     }
            > }
        "#]],
    );
}

#[test]
fn php_templates_attributes_and_loops() {
    check(
        "/templates/list.php",
        r#"<!DOCTYPE html>
<title>Unchekced markup</title>
<?php foreach ($rows as $key => &$row): ?>
  <li><?= $row['label'] ?></li>
<?php endforeach; ?>
<?php
#[Route('/users', name: 'user_list')]
function listing(array $items): array
{
    static $calls = 0;
    try {
        return array_map(fn (int $value) => $value * 2, $items);
    } catch (\RuntimeException $error) {
        return [];
    }
}
"#,
        expect![[r#"
            php
            > <!DOCTYPE html>
            > <title>Unchekced markup</title>
            > <?php foreach ($rows as $key => &$row): ?>
                                       ^^^ declaration
                                                ^^^ declaration
            >   <li><?= $row['label'] ?></li>
                              ^^^^^ string
            > <?php endforeach; ?>
            > <?php
            > #[Route('/users', name: 'user_list')]
                       ^^^^^^ string
                                       ^^^^^^^^^ string
            > function listing(array $items): array
                       ^^^^^^^ declaration
                                      ^^^^^ declaration
            > {
            >     static $calls = 0;
                          ^^^^^ declaration
            >     try {
            >         return array_map(fn (int $value) => $value * 2, $items);
                                                ^^^^^ declaration
            >     } catch (\RuntimeException $error) {
                                              ^^^^^ declaration
            >         return [];
            >     }
            > }
        "#]],
    );
}

#[test]
fn php_docblock_tags_and_their_types_are_not_prose() {
    check(
        "/src/Shapes.php",
        r#"<?php
/**
 * Reads the stat of a file.
 *
 * @param non-empty-string $filename The file to stat, if it exists.
 * @param array{dev: int, rdev: int} &$into Where the stat goes.
 * @param callable(int): void ...$then Called with the size.
 * @return array{
 *     'nlink': int<0, max>,
 *     'yday': int,
 * }|false The stat, or false when there is none.
 * @phpstan-return 'cli'|'phpdbg' Which SAPI.
 * @psalm-ignore-falsable-return
 * @throws \RuntimeException When the file is gone.
 * @author Jordi Boggiano <j.boggiano@seld.be>
 * @since 2.1 It was a function before.
 * @deprecated Use the stat command instead.
 * @ORM\Column(type="string", length=255)
 */
function stat_of(string $filename, array &$into, callable ...$then): array|false {}
"#,
        expect![[r#"
            php
            > <?php
            > /**
            >  * Reads the stat of a file.
                 ^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >  *
            >  * @param non-empty-string $filename The file to stat, if it exists.
                                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >  * @param array{dev: int, rdev: int} &$into Where the stat goes.
                                                          ^^^^^^^^^^^^^^^^^^^^ doccomment
            >  * @param callable(int): void ...$then Called with the size.
                                                     ^^^^^^^^^^^^^^^^^^^^^ doccomment
            >  * @return array{
            >  *     'nlink': int<0, max>,
            >  *     'yday': int,
            >  * }|false The stat, or false when there is none.
                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >  * @phpstan-return 'cli'|'phpdbg' Which SAPI.
                                                ^^^^^^^^^^^ doccomment
            >  * @psalm-ignore-falsable-return
            >  * @throws \RuntimeException When the file is gone.
                                           ^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >  * @author Jordi Boggiano <j.boggiano@seld.be>
            >  * @since 2.1 It was a function before.
            >  * @deprecated Use the stat command instead.
                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ doccomment
            >  * @ORM\Column(type="string", length=255)
            >  */
            > function stat_of(string $filename, array &$into, callable ...$then): array|false {}
                       ^^^^^^^ declaration
                                       ^^^^^^^^ declaration
                                                         ^^^^ declaration
                                                                            ^^^^ declaration
        "#]],
    );
}

#[test]
fn a_string_is_not_split_at_an_escaped_apostrophe() {
    check(
        "/src/Notice.php",
        "<?php\necho 'It didn\\'t work', \"Line\\nbreak\";\n",
        expect![[r#"
            php
            > <?php
            > echo 'It didn\'t work', "Line\nbreak";
                    ^^^^^^^^^^^^^^^ string
                                       ^^^^ string
                                             ^^^^^ string
        "#]],
    );
}
