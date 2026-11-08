use expect_test::expect;

use crate::tests::check;

#[test]
fn sql_prose_is_comments_and_declared_names() {
    check(
        "/migrations/0001_users.sql",
        r#"-- Users of the app.
/* A block
   comment. */
CREATE TABLE IF NOT EXISTS app.users (
  user_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id uuid REFERENCES orgs (org_id),
  CONSTRAINT users_unique UNIQUE (user_id)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_name ON app.users (user_id);
CREATE VIEW active_users AS SELECT user_id FROM app.users u JOIN orgs o ON o.id = u.org_id;
CREATE TYPE mood AS ENUM ('happy', 'sad');
CREATE SCHEMA IF NOT EXISTS billing;
CREATE FUNCTION add_one(input_value int) RETURNS int AS $$ SELECT input_value + 1 $$ LANGUAGE sql;
ALTER TABLE app.users ADD COLUMN IF NOT EXISTS nick_name text;
INSERT INTO app.users (user_id, nick_name) VALUES (1, 'alice');
"#,
        expect![[r#"
            sql
            > -- Users of the app.
              ^^^^^^^^^^^^^^^^^^^^ comment
            > /* A block
              ^^^^^^^^^^ comment (+1 lines)
            >    comment. */
            > CREATE TABLE IF NOT EXISTS app.users (
                                             ^^^^^ declaration
            >   user_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
                ^^^^^^^ declaration
            >   org_id uuid REFERENCES orgs (org_id),
                ^^^^^^ declaration
            >   CONSTRAINT users_unique UNIQUE (user_id)
            > );
            > CREATE UNIQUE INDEX IF NOT EXISTS idx_users_name ON app.users (user_id);
                                                ^^^^^^^^^^^^^^ declaration
            > CREATE VIEW active_users AS SELECT user_id FROM app.users u JOIN orgs o ON o.id = u.org_id;
                          ^^^^^^^^^^^^ declaration
            > CREATE TYPE mood AS ENUM ('happy', 'sad');
                          ^^^^ declaration
                                        ^^^^^^^ value
                                                 ^^^^^ value
            > CREATE SCHEMA IF NOT EXISTS billing;
                                          ^^^^^^^ declaration
            > CREATE FUNCTION add_one(input_value int) RETURNS int AS $$ SELECT input_value + 1 $$ LANGUAGE sql;
                              ^^^^^^^ declaration
                                      ^^^^^^^^^^^ declaration
            > ALTER TABLE app.users ADD COLUMN IF NOT EXISTS nick_name text;
                                                             ^^^^^^^^^ declaration
            > INSERT INTO app.users (user_id, nick_name) VALUES (1, 'alice');
                                                                    ^^^^^^^ value
        "#]],
    );
}

#[test]
fn sql_strings_are_text_and_values() {
    check(
        "/migrations/0002_seed.sql",
        r#"CREATE TABLE t (
  status text NOT NULL DEFAULT 'pending',
  note text DEFAULT 'Nothing to see here' COMMENT 'Free text note',
  weight int DEFAULT 10
);
INSERT INTO t (status, note) VALUES ('active', E'Line one\nline two'), (N'Hello world', 'x@example.com');
SELECT * FROM t WHERE status = 'done' AND "Odd Name" = "other";
"#,
        expect![[r#"
            sql
            > CREATE TABLE t (
                           ^ declaration
            >   status text NOT NULL DEFAULT 'pending',
                ^^^^^^ declaration
                                             ^^^^^^^^^ value
            >   note text DEFAULT 'Nothing to see here' COMMENT 'Free text note',
                ^^^^ declaration
                                  ^^^^^^^^^^^^^^^^^^^^^ string
                                                                ^^^^^^^^^^^^^^^^ string
            >   weight int DEFAULT 10
                ^^^^^^ declaration
            > );
            > INSERT INTO t (status, note) VALUES ('active', E'Line one\nline two'), (N'Hello world', 'x@example.com');
                                                   ^^^^^^^^ value
                                                             ^^^^^^^^^^^^^^^^^^^^^ string
                                                                                      ^^^^^^^^^^^^^^ string
                                                                                                      ^^^^^^^^^^^^^^^ value
            > SELECT * FROM t WHERE status = 'done' AND "Odd Name" = "other";
                                             ^^^^^^ value
        "#]],
    );
}

#[test]
fn sql_mysql_backticks_and_engine_options() {
    check(
        "/migrations/0003_items.sql",
        r#"CREATE TABLE `order_items` (
  `item_id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
  `qty_ordered` INT DEFAULT 0,
  PRIMARY KEY (`item_id`),
  KEY `idx_qty` (`qty_ordered`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
"#,
        expect![[r#"
            sql
            > CREATE TABLE `order_items` (
                            ^^^^^^^^^^^^ declaration
            >   `item_id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
                 ^^^^^^^^ declaration
            >   `qty_ordered` INT DEFAULT 0,
                 ^^^^^^^^^^^^ declaration
            >   PRIMARY KEY (`item_id`),
            >   KEY `idx_qty` (`qty_ordered`)
            > ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#]],
    );
}

#[test]
fn sql_a_statement_that_does_not_parse_leaves_the_next_one_alone() {
    check(
        "/migrations/0004_broken.sql",
        r#"GARBAGE ((( nonsense here;
CREATE TABLE after_error (col_one int);
CREATE TABLE gt (id INTEGER PRIMARY KEY AUTOINCREMENT, nm TEXT);
CREATE PROCEDURE do_proc(IN a INT) BEGIN SELECT 1; END;
CREATE TABLE last_one (col_two int);
"#,
        expect![[r#"
            sql
            > GARBAGE ((( nonsense here;
            > CREATE TABLE after_error (col_one int);
                           ^^^^^^^^^^^ declaration
                                        ^^^^^^^ declaration
            > CREATE TABLE gt (id INTEGER PRIMARY KEY AUTOINCREMENT, nm TEXT);
                           ^^ declaration
                               ^^ declaration
                                                                     ^^ declaration
            > CREATE PROCEDURE do_proc(IN a INT) BEGIN SELECT 1; END;
            > CREATE TABLE last_one (col_two int);
                           ^^^^^^^^ declaration
                                     ^^^^^^^ declaration
        "#]],
    );
}

#[test]
fn sql_function_arguments_and_locals_are_declared_and_the_body_holds_comments() {
    check(
        "/migrations/0005_function.sql",
        r#"CREATE FUNCTION f(x int) RETURNS int AS $body$
DECLARE
  my_var int := 3;
BEGIN
  -- inside the body
  RETURN my_var;
END;
$body$ LANGUAGE plpgsql;
"#,
        expect![[r#"
            sql
            > CREATE FUNCTION f(x int) RETURNS int AS $body$
                              ^ declaration
                                ^ declaration
            > DECLARE
            >   my_var int := 3;
                ^^^^^^ declaration
            > BEGIN
            >   -- inside the body
                ^^^^^^^^^^^^^^^^^^ comment
            >   RETURN my_var;
            > END;
            > $body$ LANGUAGE plpgsql;
        "#]],
    );
}
