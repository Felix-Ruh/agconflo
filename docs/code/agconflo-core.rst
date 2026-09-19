=====================
Code in agconflo-core
=====================

Where each component requirement allocated to ``agconflo-core`` is met in its
source. Nothing below is written here: every implementation comes from a
one-line marker in ``crates/agconflo-core/src``, placed on the item that does
the work::

  // @<title>,<IMPL id>,impl,[<component requirement ids>]

The identifier is ``IMPL_`` followed by the module and a name for what the code
does there, the way a test case's identifier follows its test's path, so the
module says which component the code belongs to. One requirement may be met in
several places, and each place carries its own marker: that is why the markers
are one line each rather than references to a need written here, which keep
only the first place (``DEC_IMPL_FROM_MARKERS``).

Each implementation's ``code_url`` is a link into the repository at the commit
being checked, recorded when the project is indexed rather than written by
anyone. A marker naming a requirement that does not exist is a dead link, and
fails the check like any other. A requirement that no marker names is not an
error, since code comes after its requirement; the review report
``scripts/reports/unimplemented.cypher`` lists them.

.. src-trace::
   :project: agconflo-core
