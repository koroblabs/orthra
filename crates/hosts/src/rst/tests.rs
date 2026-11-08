use expect_test::expect;

use crate::tests::check;

#[test]
fn a_paragraph_and_a_title_are_prose_and_a_literal_is_not() {
    check(
        "/docs/index.rst",
        r#"Seperates the Title
===================

A paragraph with ``recieve()``, :func:`os.occured` and `Link teh <http://x.org>`_ then |sub| and *emphasis* teh.

Run it::

    recieve --occured

>>> recieve()

* a list item teh
"#,
        expect![[r#"
            restructuredtext
            > Seperates the Title
              ^^^^^^^^^^^^^^^^^^^ prose
            > ===================
            >
            > A paragraph with ``recieve()``, :func:`os.occured` and `Link teh <http://x.org>`_ then |sub| and *emphasis* teh.
              ^^^^^^^^^^^^^^^^ prose
                                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ prose
                                                                                                           ^^^^^^^^^^^^^^^^^^^ prose
            >
            > Run it::
            >
            >     recieve --occured
            >
            > >>> recieve()
            >
            > * a list item teh
                ^^^^^^^^^^^^^^^ prose
        "#]],
    );
}

#[test]
fn a_directive_is_prose_only_where_it_holds_a_sentence() {
    check(
        "/docs/usage.rst",
        r#".. note:: Seperates this title

   The body teh of a note.

.. code-block:: python
   :caption: A caption teh

   x = recieve

.. image:: occured.png
   :alt: An alt teh
   :width: 20

.. function:: recieve(teh)

   Documents teh function.

.. _occured-target: http://occured.example.com

.. |sub| replace:: replaced teh

.. a comment teh

:Author: Teh Author
:param recieve: teh
"#,
        expect![[r#"
            restructuredtext
            > .. note:: Seperates this title
                        ^^^^^^^^^^^^^^^^^^^^ prose
            >
            >    The body teh of a note.
                 ^^^^^^^^^^^^^^^^^^^^^^^ prose
            >
            > .. code-block:: python
            >    :caption: A caption teh
                           ^^^^^^^^^^^^^ prose
            >
            >    x = recieve
            >
            > .. image:: occured.png
            >    :alt: An alt teh
                       ^^^^^^^^^^ prose
            >    :width: 20
            >
            > .. function:: recieve(teh)
            >
            >    Documents teh function.
            >
            > .. _occured-target: http://occured.example.com
            >
            > .. |sub| replace:: replaced teh
            >
            > .. a comment teh
              ^^^^^^^^^^^^^^^^ comment
            >
            > :Author: Teh Author
                       ^^^^^^^^^^ prose
            > :param recieve: teh
                              ^^^ prose
        "#]],
    );
}

#[test]
fn an_admonition_holding_code_is_given_up() {
    check(
        "/docs/note.rst",
        r#".. warning::

   Run it teh way::

      recieve --occured

.. tip::

   A plain tip teh.

.. admonition:: Seperates

   Look.

   .. code-block:: python

      recieve = 1

A closing paragraph teh.
"#,
        expect![[r#"
            restructuredtext
            > .. warning::
            >
            >    Run it teh way::
            >
            >       recieve --occured
            >
            > .. tip::
            >
            >    A plain tip teh.
                 ^^^^^^^^^^^^^^^^ prose
            >
            > .. admonition:: Seperates
                              ^^^^^^^^^ prose
            >
            >    Look.
            >
            >    .. code-block:: python
            >
            >       recieve = 1
            >
            > A closing paragraph teh.
              ^^^^^^^^^^^^^^^^^^^^^^^^ prose
        "#]],
    );
}

#[test]
fn a_table_is_not_prose() {
    check(
        "/docs/table.rst",
        r#"===== =====
Col a Col b
===== =====
v1    v2
===== =====

+-----+-----+
| a   | b   |
+-----+-----+

After teh table.
"#,
        expect![[r#"
            restructuredtext
            > ===== =====
            > Col a Col b
            > ===== =====
            > v1    v2
            > ===== =====
            >
            > +-----+-----+
            > | a   | b   |
            > +-----+-----+
            >
            > After teh table.
              ^^^^^^^^^^^^^^^^ prose
        "#]],
    );
}
