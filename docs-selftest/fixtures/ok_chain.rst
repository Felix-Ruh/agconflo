=================
Fixture: ok_chain
=================

.. One valid need of every declared type, wired with every declared link, every
   enumerated field at a legal value. It must produce nothing at all, which makes
   it the control for the whole metamodel: deleting a type reports an unknown
   directive here, deleting a link type reports an unknown option, and narrowing
   an enum reports a rejected value. The needs share this one file because a
   fixture is checked as a one-file project, so a link can only resolve within it.

.. stkh_req:: A node sees only what was wired to it
   :id: STKH_CHAIN
   :stakeholder: user
   :statement: Agconflo shall give a node exactly the contexts wired to it.

   Rationale belongs in the body, and the obligation in the statement above.

.. feat_req:: Binding a node's parameters
   :id: FEAT_CHAIN
   :derived_from: STKH_CHAIN
   :ears_pattern: event
   :verification_method: test
   :statement: When a node activation begins, Agconflo shall bind only the contexts wired to its parameters.

   Rationale belongs in the body, and the obligation in the statement above.

.. feat_arch:: Binding belongs to the context store
   :id: ARCH_CHAIN
   :realises: FEAT_CHAIN
   :uses: COMP_CHAIN
   :statement: Agconflo shall bind a node's parameters inside the context store.

   Rationale belongs in the body, and the obligation in the statement above.

.. comp:: Context store
   :id: COMP_CHAIN
   :crate: agconflo-core

   The component that owns context identity and lineage.

.. comp_req:: Rejecting an unwired parameter
   :id: CREQ_CHAIN
   :derived_from: FEAT_CHAIN
   :allocated_to: COMP_CHAIN
   :ears_pattern: unwanted
   :statement: If a parameter has no wired context, then Context store shall reject the activation.

   The grammatical subject is the component named in allocated_to, which is what
   makes a restatement of the parent detectable rather than merely suspected.

.. test_case:: An unwired parameter is rejected
   :id: TEST_CHAIN
   :verifies: CREQ_CHAIN
   :test_kind: error_path
   :coverage: full

   Asserts which error occurs and what the run does afterwards, not merely that
   something failed.
