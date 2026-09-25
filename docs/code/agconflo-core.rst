=====================
Code in agconflo-core
=====================

Where each component requirement allocated to ``agconflo-core`` is met in its
source. Nothing below is written here: every implementation comes from a
one-line marker in ``crates/agconflo-core/src``, placed on the item that does
the work::

  // @<title>,<IMPL id>,impl,[<component requirement ids>],[<decision or note ids>]
  // @<title>,<TRACE id>,trace,[],[<decision or note ids>]

The identifier is ``IMPL_`` followed by the module and a name for what the code
does there, the way a test case's identifier follows its test's path, so the
module says which component the code belongs to. One requirement may be met in
several places, and each place carries its own marker: that is why the markers
are one line each rather than references to a need written here, which keep
only the first place (``DEC_IMPL_FROM_MARKERS``).

The second list, which may be left out, names the decisions and notes the code
follows (``DEC_CODE_FOLLOWS_BY_MARKER``). A ``trace`` marker is for code that
follows one and meets no component requirement (``DEC_TRACE_MARKERS``). A note
is written in this document, as a ``code_note``, when an explanation belongs to
one piece of this crate's code alone (``DEC_NOTES_BESIDE_THE_CRATE``).

Each implementation's and trace's ``code_url`` is a link into the repository
at the commit being checked, recorded when the project is indexed rather than
written by anyone. A marker naming a requirement that does not exist is a dead link, and
fails the check like any other. A requirement that no marker names is not an
error, since code comes after its requirement; the review report
``scripts/reports/unimplemented.cypher`` lists them.

.. code_note:: Deep contexts are walked without recursion
   :id: NOTE_CONTEXT_NO_RECURSION

   Nesting has no fixed depth, so everything that goes through a composition's
   parts does so with a stack of its own rather than the call stack. Measured
   at 100,000 levels, a recursive render and the derived drop each overflowed
   even a 32 MiB stack, and a recursive lineage walk an 8 MiB one - an abort,
   not a failure anyone can assert on. The drop releases a part only when it
   was the last holder, which is when ``Arc::into_inner`` hands it over; a part
   held anywhere else is left whole.

   ``Debug`` is written out for the same reason: a derived one would recurse
   into every part, an abort on a deep context and a whole tree printed on a
   shallow one, so parts are shown by identifier.

.. code_note:: A context carries no metadata yet
   :id: NOTE_CONTEXT_NO_METADATA

   There is deliberately no metadata on a context. ``DEC_METADATA_TRANSFORMED``
   has no requirement to answer to until transform nodes exist, and since
   content is reached only through methods, metadata can be added later without
   breaking any caller. Its absence is a decision, not an omission to fill in.

.. code_note:: What an identifier and its source leave out
   :id: NOTE_ID_TRAITS

   A context identifier has no ordering. Nothing promises that a later context
   carries a larger identifier, and an ``Ord`` would invite code to assume it.
   Copying one is harmless: a copy names the same context, and a context is
   created from a source rather than from an identifier, so a copy can never
   label a second one.

   The source's ``Default`` is written out rather than derived. A derived one
   sets the next identifier to none, which is a source that has already run
   out - measured, and exactly the derive clippy's ``new_without_default``
   invites.

.. code_note:: How the compile-fail test cases compile their snippets
   :id: NOTE_COMPILE_FAIL_HARNESS

   Not ``compile_fail`` doctests: the test runner does not run doctests
   (``EVD_NEXTEST_NO_DOCTESTS``), so such a case would never produce a result.
   Not ``trybuild`` either, which compares the compiler's whole output against a
   stored copy - wording, layout, line numbers - while the toolchain follows
   stable, so a new compiler could fail those cases with no code having
   changed; it also rebuilt every dev-dependency for its scratch project,
   measured at 9 s from cold. What a case needs asserted is narrower: the error
   code, and the phrase naming why the code was refused.

   Each snippet is the body of ``main`` in a project of its own under
   ``<target>/compile-fail/<case>/``, depending on the crate by path and checked
   with ``cargo check``. The projects share one target directory, so the crate
   is compiled for them once and cargo's lock serialises concurrent cases, and
   it is compiled there without ``cfg(test)``: a snippet sees exactly what a
   dependent crate would. A refusal is evidence only beside a neighbouring
   snippet that is seen to compile.

.. src-trace::
   :project: agconflo-core
