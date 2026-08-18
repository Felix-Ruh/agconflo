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

.. evd:: The unlicensed free tier is measured in files, not needs
   :id: EVD_CHAIN
   :evd_kind: measurement
   :observed_on: 2026-08-17
   :observation: ubc check exits 0 on five source files without a licence and exits 1 on six.

   Evidence carries its finding in observation rather than statement, so the
   requirement wording rules never reach it. A measurement is comparative and
   approximate by nature, which is what those rules exist to ban from a
   requirement - and what this field has to be free to say.

.. dec:: The documentation toolchain was Sphinx and Python
   :id: DEC_CHAIN_OLD
   :dec_status: superseded
   :decided_on: 2026-08-14
   :statement: Agconflo shall build its requirements project with Sphinx and Python.

   Kept rather than deleted, and marked superseded rather than edited into the
   decision that replaced it. A reversal is part of the record.

.. dec:: The documentation toolchain is ubc alone
   :id: DEC_CHAIN
   :dec_status: accepted
   :decided_on: 2026-08-16
   :supported_by: EVD_CHAIN
   :supersedes: DEC_CHAIN_OLD
   :statement: Agconflo shall build its requirements project with ubc and no Python.

   A decision keeps its obligation in statement, so it IS held to the wording
   rules - one hiding behind "the simpler option" is refused. This pair also
   wires both new link types, which is what makes their declarations guarded.
