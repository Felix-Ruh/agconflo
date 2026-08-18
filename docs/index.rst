=====================
Agconflo requirements
=====================

This is the root of Agconflo's requirements project. Requirements are grouped by
subject, and every group is listed in the table of contents below.

That listing is enforced rather than decorative. A document is indexed and
validated wherever it sits, but once any toctree exists in a project, ubc reports
every document that no toctree reaches - so a requirements file that nobody added
below fails the build instead of sitting unread.

The metamodel they are written against lives in ``ubproject.toml`` and
``schemas.json``: the need types, the links that join them, and the rules
covering mandatory fields per level, link targets, allocation cardinality, EARS
grammar and requirement smells. Exact counts are deliberately not repeated here -
prose that restates a number goes stale the first time the number changes, and
nothing checks it. The invariant is worth stating instead: every one of those
rules is guarded by a fixture in ``docs-selftest/`` that fails without it,
because a wrongly shaped rule is silently ignored rather than rejected.

.. toctree::
   :maxdepth: 2

   stakeholder/context
   stakeholder/authoring
   stakeholder/execution
   stakeholder/process
   decisions/context
