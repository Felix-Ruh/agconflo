=====================
Agconflo requirements
=====================

This is the root of Agconflo's requirements project. It still holds exactly one
requirement: the metamodel is being built out ahead of the requirements written
against it, deliberately, so that the rules are in place and proven to fire
before there is any material for them to be wrong about.

The requirement below is a real one rather than a placeholder - it is the
project's core thesis, stated in the form the rest will take. The obligation
itself is the ``statement`` field; the body carries the reasoning behind it.

The metamodel it is written against is complete: six need types and five link
types in ``ubproject.toml``, and thirty-two rules in ``schemas.json`` covering
mandatory fields per level, link targets, allocation cardinality, EARS grammar
and requirement smells. Every one of those rules is guarded by a fixture in
``docs-selftest/`` that fails without it, because a wrongly shaped rule is
silently ignored rather than rejected.

Stakeholder requirements
========================

.. stkh_req:: A node sees only what was wired to it
   :id: STKH_EXPLICIT_CONTEXT
   :stakeholder: user
   :statement: Agconflo shall give a node exactly the contexts wired to it.

   Without this property the project is a worse LangGraph. The point of a
   context-centric engine is that "what was in this call's context window, and
   where did every byte come from?" has an exact answer, so any feature that
   makes a node's visible context implicit or unauditable needs a very good
   reason. Global contexts are the one sanctioned exception, and are recorded in
   the run log as genuine inputs precisely so that they do not become one.
