============================
Fixture: report_single_child
============================

.. The fixture for the single-child review report in scripts/reports/. Every
   need here is valid by every schema rule, so the golden records nothing.

   The report's own check uses the same convention as a gate's: every need
   whose id contains _BAD_ must appear in it, and no other need may. The
   difference is only what happens on docs/, where a report prints its rows for
   a person instead of failing on them.

   One offender copies its parent's statement outright, so its same_response
   must be true; the other refines its parent into something new, so it must be
   false. The runner checks which needs appear, not that column, so the column
   was checked by hand against this file.

.. stkh_req:: A parent with one child
   :id: STKH_ONE
   :stakeholder: user
   :statement: Agconflo shall give a node exactly the contexts wired to it.

   Has exactly one child, below.

.. feat_req:: The only child, copied from its parent
   :id: FEAT_BAD_COPIED_CHILD
   :derived_from: STKH_ONE
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall give a node exactly the contexts wired to it.

   The parent's statement, word for word. Must appear, with same_response true.

.. stkh_req:: A parent with two children
   :id: STKH_TWO
   :stakeholder: user
   :statement: Agconflo shall record which context each byte of a node's input came from.

   Has two children, so neither of them may appear.

.. feat_req:: The first of two children
   :id: FEAT_OK_SIBLING_A
   :derived_from: STKH_TWO
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall give every context an identifier unique within its run.

   Has a sibling. Must not appear.

.. feat_req:: The second of two children
   :id: FEAT_OK_SIBLING_B
   :derived_from: STKH_TWO
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall report every context a composed context was derived from.

   Has a sibling. Must not appear.

.. stkh_req:: A parent with no children
   :id: STKH_NONE
   :stakeholder: user
   :statement: Agconflo shall not alter a context after it has been created.

   Has no children. Nothing about it may appear.

.. comp:: Context store
   :id: COMP_ONE
   :crate: agconflo-core

   Owns the requirement below.

.. comp_req:: The only child one level further down
   :id: CREQ_BAD_ONLY_CHILD
   :derived_from: FEAT_OK_SIBLING_A
   :allocated_to: COMP_ONE
   :ears_pattern: ubiquitous
   :statement: Context store shall issue every identifier from one counter per run.

   The only child of a requirement that is itself one of two siblings, so the
   report works at every level rather than only at the top. It refines its
   parent into something new. Must appear, with same_response false.
