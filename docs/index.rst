=====================
Agconflo requirements
=====================

This is the root of Agconflo's requirements project. Requirements are grouped by
subject in the directories beside this file rather than kept in one place. A
document is indexed and validated wherever it sits, so the grouping is for
readers rather than for the toolchain.

The metamodel they are written against lives in ``ubproject.toml`` and
``schemas.json``: the need types, the links that join them, and the rules
covering mandatory fields per level, link targets, allocation cardinality, EARS
grammar and requirement smells. Exact counts are deliberately not repeated here -
prose that restates a number goes stale the first time the number changes, and
nothing checks it. The invariant is worth stating instead: every one of those
rules is guarded by a fixture in ``docs-selftest/`` that fails without it,
because a wrongly shaped rule is silently ignored rather than rejected.
