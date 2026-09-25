=======================
Code in agconflo-runner
=======================

Where each component requirement allocated to ``agconflo-runner`` is met in
its source. Nothing below is written here: every implementation comes from a
one-line marker in ``crates/agconflo-runner/src``, placed on the item that does
the work::

  // @<title>,<IMPL id>,impl,[<component requirement ids>],[<decision or note ids>]
  // @<title>,<TRACE id>,trace,[],[<decision or note ids>]

The identifier is ``IMPL_`` followed by the module and a name for what the code
does there, the way a test case's identifier follows its test's path, so the
module says which component the code belongs to. One requirement may be met in
several places, and each place carries its own marker (``DEC_IMPL_FROM_MARKERS``).

The second list, which may be left out, names the decisions and notes the code
follows (``DEC_CODE_FOLLOWS_BY_MARKER``). A ``trace`` marker is for code that
follows one and meets no component requirement (``DEC_TRACE_MARKERS``). A note
is written in this document, as a ``code_note``, when an explanation belongs to
one piece of this crate's code alone (``DEC_NOTES_BESIDE_THE_CRATE``).

Each implementation's and trace's ``code_url`` is a link into the repository
at the commit being checked, recorded when the project is indexed rather than
written by anyone. A requirement that no marker names is not an error, since
code comes after its requirement; the review report
``scripts/reports/unimplemented.cypher`` lists them.

.. src-trace::
   :project: agconflo-runner
