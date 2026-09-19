========================
Fixture: report_untested
========================

.. The fixture for the untested-requirements review report in scripts/reports/.
   Every need here is valid by every schema rule, so the golden records nothing:
   no rule can ask whether a requirement is checked, because a requirement is
   written before the case that checks it and nothing may refuse it for that.

   Every need whose id contains _BAD_ must appear in the report, and no other
   may. Two of the offenders are joined to something a looser query would take
   for a test: an implementation, and a test case naming the requirement through
   ubc's built-in links instead of verifies. The controls are a requirement
   verified by one case, two requirements verified by a single case that names
   both, and the feature requirement above them, which no case verifies either
   and which must not appear: only a component requirement is in this queue.

.. stkh_req:: A node sees only what was wired to it
   :id: STKH_UNTESTED
   :stakeholder: user
   :statement: Agconflo shall give a node exactly the contexts wired to it.

   Not a component requirement. Must not appear.

.. feat_req:: Binding a node's parameters
   :id: FEAT_UNTESTED
   :derived_from: STKH_UNTESTED
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall bind only the contexts wired to a node's parameters.

   No case verifies this one either. Must not appear.

.. comp:: Context store
   :id: COMP_UNTESTED
   :crate: agconflo-core

   Owns every component requirement below.

.. comp_req:: Verified by one case
   :id: CREQ_OK_VERIFIED
   :derived_from: FEAT_UNTESTED
   :allocated_to: COMP_UNTESTED
   :ears_pattern: ubiquitous
   :statement: Context store shall bind only the contexts wired to a parameter.

   Must not appear.

.. comp_req:: Verified beside another requirement
   :id: CREQ_OK_SHARED_FIRST
   :derived_from: FEAT_UNTESTED
   :allocated_to: COMP_UNTESTED
   :ears_pattern: ubiquitous
   :statement: Context store shall keep every bound context until its run ends.

   Named first by a case that verifies two. Must not appear.

.. comp_req:: Verified after another requirement
   :id: CREQ_OK_SHARED_SECOND
   :derived_from: FEAT_UNTESTED
   :allocated_to: COMP_UNTESTED
   :ears_pattern: ubiquitous
   :statement: Context store shall release a context once no node holds it.

   Named second by the same case, so a query reading only the first target of a
   link would miss it. Must not appear.

.. comp_req:: Checked by nothing at all
   :id: CREQ_BAD_NO_TEST
   :derived_from: FEAT_UNTESTED
   :allocated_to: COMP_UNTESTED
   :ears_pattern: ubiquitous
   :statement: Context store shall reject a parameter with no wired context.

   Must appear.

.. comp_req:: Implemented but not verified
   :id: CREQ_BAD_ONLY_IMPLEMENTED
   :derived_from: FEAT_UNTESTED
   :allocated_to: COMP_UNTESTED
   :ears_pattern: ubiquitous
   :statement: Context store shall give every bound context a fresh identifier.

   Code answers for it and nothing checks that code, which is the case this
   queue is most worth reading for. Must appear.

.. comp_req:: Named by a case through links rather than verifies
   :id: CREQ_BAD_ONLY_LINKED
   :derived_from: FEAT_UNTESTED
   :allocated_to: COMP_UNTESTED
   :ears_pattern: ubiquitous
   :statement: Context store shall record the node every context was bound to.

   A test case mentions it, without claiming to verify it. Must appear.

.. test_case:: Verifies one requirement
   :id: TEST_UNTESTED_ONE
   :verifies: CREQ_OK_VERIFIED
   :test_kind: positive
   :coverage: full

   The only link into the requirement it verifies.

.. test_case:: Verifies two requirements at once
   :id: TEST_UNTESTED_BOTH
   :verifies: CREQ_OK_SHARED_FIRST, CREQ_OK_SHARED_SECOND
   :test_kind: property
   :coverage: partial

   One case answering for two requirements.

.. test_case:: Mentions a requirement without verifying it
   :id: TEST_UNTESTED_MENTIONS
   :verifies: CREQ_OK_VERIFIED
   :links: CREQ_BAD_ONLY_LINKED
   :test_kind: positive
   :coverage: partial

   The only link into the requirement it names through links.

.. impl:: Implements the requirement nothing verifies
   :id: IMPL_UNTESTED
   :implements: CREQ_BAD_ONLY_IMPLEMENTED
   :code_url: https://example.invalid/crates/agconflo-core/src/store.rs#L1

   Code is not a test of itself.
