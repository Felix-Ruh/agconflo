=====================
Agconflo requirements
=====================

This is the root of Agconflo's requirements project. It is currently a
**toolchain probe**: it holds exactly one requirement, which exists to prove
that authoring, indexing, validation and querying work end to end before any
real requirements are written against them.

The requirement below is nonetheless a real one rather than a placeholder - it
is the project's core thesis, stated in the form the rest will take. The other
requirement levels, the link types between them, and schema validation all
arrive with the metamodel; see the bootstrap plan.

Stakeholder requirements
========================

.. stkh_req:: A node sees only what was wired to it
   :id: STKH_EXPLICIT_CONTEXT

   A node shall receive exactly the contexts that are wired to it as inputs, and
   no others.
