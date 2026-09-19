=============================
Fixture: report_unimplemented
=============================

.. The fixture for the unimplemented-requirements review report in
   scripts/reports/. Every need here is valid by every schema rule, so the
   golden records nothing - and no rule could do better, since a requirement is
   written before its code and nothing may refuse it for that.

   Every need whose id contains _BAD_ must appear in the report, and no other
   may. Three of the offenders are joined to something that a looser query
   would take for code: a test case verifying one, a test case naming another
   through implements, and an implementation naming the third through ubc's
   built-in links instead of implements. The controls are an implementation
   naming two requirements at once, a requirement with two implementations, and
   the stakeholder and feature requirements, which have no code either and must
   not appear: only a component requirement is answered by code.

.. stkh_req:: A node sees only what was wired to it
   :id: STKH_UNIMPL
   :stakeholder: user
   :statement: Agconflo shall give a node exactly the contexts wired to it.

   Has no code. Must not appear.

.. feat_req:: Binding a node's parameters
   :id: FEAT_UNIMPL
   :derived_from: STKH_UNIMPL
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall bind only the contexts wired to a node's parameters.

   Has no code. Must not appear.

.. comp:: Context store
   :id: COMP_UNIMPL
   :crate: agconflo-core

   Owns every component requirement below.

.. comp_req:: Implemented twice
   :id: CREQ_OK_IMPLEMENTED
   :derived_from: FEAT_UNIMPL
   :allocated_to: COMP_UNIMPL
   :ears_pattern: ubiquitous
   :statement: Context store shall bind only the contexts wired to a parameter.

   Two implementations name it. Must not appear.

.. comp_req:: Implemented beside another requirement
   :id: CREQ_OK_SHARED_FIRST
   :derived_from: FEAT_UNIMPL
   :allocated_to: COMP_UNIMPL
   :ears_pattern: ubiquitous
   :statement: Context store shall keep every bound context until its run ends.

   Named first by an implementation that names two. Must not appear.

.. comp_req:: Implemented after another requirement
   :id: CREQ_OK_SHARED_SECOND
   :derived_from: FEAT_UNIMPL
   :allocated_to: COMP_UNIMPL
   :ears_pattern: ubiquitous
   :statement: Context store shall release a context once no node holds it.

   Named second by the same implementation, so a query reading only the first
   target of a link would miss it. Must not appear.

.. comp_req:: No code at all
   :id: CREQ_BAD_NO_CODE
   :derived_from: FEAT_UNIMPL
   :allocated_to: COMP_UNIMPL
   :ears_pattern: ubiquitous
   :statement: Context store shall reject a parameter with no wired context.

   Nothing names it. Must appear.

.. comp_req:: Tested but not implemented
   :id: CREQ_BAD_ONLY_TESTED
   :derived_from: FEAT_UNIMPL
   :allocated_to: COMP_UNIMPL
   :ears_pattern: ubiquitous
   :statement: Context store shall give every bound context a fresh identifier.

   A test case verifies it, and a test is not code. Must appear.

.. comp_req:: Linked but not implemented
   :id: CREQ_BAD_ONLY_LINKED
   :derived_from: FEAT_UNIMPL
   :allocated_to: COMP_UNIMPL
   :ears_pattern: ubiquitous
   :statement: Context store shall record the node every context was bound to.

   An implementation names it, but through links rather than implements, so it
   says nothing about answering for it. Must appear.

.. comp_req:: Named by something that is not code
   :id: CREQ_BAD_NOT_CODE
   :derived_from: FEAT_UNIMPL
   :allocated_to: COMP_UNIMPL
   :ears_pattern: ubiquitous
   :statement: Context store shall refuse a context type with an empty name.

   A test case names it through implements. Any need may carry that link, and
   only an implementation's is checked, so this is valid - and a test case is
   still not code. Must appear.

.. test_case:: Every bound context gets a fresh identifier
   :id: TEST_UNIMPL
   :verifies: CREQ_BAD_ONLY_TESTED
   :test_kind: property
   :coverage: full

   The only link into the requirement it verifies.

.. test_case:: A test case that claims to implement
   :id: TEST_UNIMPL_CLAIMS_CODE
   :verifies: CREQ_OK_IMPLEMENTED
   :implements: CREQ_BAD_NOT_CODE
   :test_kind: positive
   :coverage: partial

   The only link into the requirement it names through implements.

.. impl:: Binding in the store
   :id: IMPL_UNIMPL_BIND
   :implements: CREQ_OK_IMPLEMENTED
   :code_url: https://example.invalid/crates/agconflo-core/src/store.rs#L1

   One of two implementations of the same requirement.

.. impl:: Binding and recording in the store
   :id: IMPL_UNIMPL_RECORD
   :implements: CREQ_OK_IMPLEMENTED
   :links: CREQ_BAD_ONLY_LINKED
   :code_url: https://example.invalid/crates/agconflo-core/src/store.rs#L2

   The second implementation of the same requirement, and the only link into
   the one it names through links.

.. impl:: Keeping and releasing in the store
   :id: IMPL_UNIMPL_LIFETIME
   :implements: CREQ_OK_SHARED_FIRST, CREQ_OK_SHARED_SECOND
   :code_url: https://example.invalid/crates/agconflo-core/src/store.rs#L3

   One implementation answering for two requirements.
