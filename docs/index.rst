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
because a wrongly shaped rule can be silently ignored rather than rejected.

.. toctree::
   :maxdepth: 2

   stakeholder/context
   stakeholder/authoring
   stakeholder/execution
   stakeholder/process
   features/context
   features/wiring
   features/topology
   features/run
   features/behaviour
   features/models
   features/resume
   features/person
   features/yield
   components/context
   components/wiring
   components/topology
   components/run
   components/behaviour
   components/models
   components/resume
   components/person
   tests/context
   tests/wiring
   tests/topology
   tests/run
   tests/behaviour
   tests/models
   tests/resume
   tests/person
   code/agconflo-core
   code/agconflo-lua
   decisions/context
   decisions/workflow
   decisions/topology
   decisions/run
   decisions/behaviour
   decisions/models
   decisions/resume
   decisions/person
   decisions/yield
   decisions/toolchain
   decisions/requirements
   decisions/changes
   evidence/toolchain
   evidence/requirements
   evidence/changes
   evidence/topology
   evidence/run
   evidence/behaviour
   evidence/models
   evidence/resume
   evidence/person
   evidence/yield
