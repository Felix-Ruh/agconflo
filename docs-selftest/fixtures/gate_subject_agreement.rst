===============================
Fixture: gate_subject_agreement
===============================

.. The fixture for the subject-agreement Cypher gate in scripts/gates/. Every
   need here is valid by every schema rule, so this file's golden records no
   diagnostics at all - which is the point: schema validation cannot see a
   component requirement whose subject is not its component, and only the gate
   can.

   The gate's own check reads this file by a naming convention rather than a
   golden: every need whose id contains _BAD_ must be reported, and no other
   need may be. The controls are what make that a real test. The query this
   gate was first measured with flagged a VALID unwanted requirement, because it
   looked for the title at the start of the statement - so each EARS pattern
   gets a correct requirement that must stay unreported, alongside a wrong one
   that must not.

.. stkh_req:: A node sees only what was wired to it
   :id: STKH_SUBJ
   :stakeholder: user
   :statement: Agconflo shall give a node exactly the contexts wired to it.

   Parent of everything below.

.. feat_req:: Binding a node's parameters
   :id: FEAT_SUBJ
   :derived_from: STKH_SUBJ
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall bind only the contexts wired to a node's parameters.

   Parent of every component requirement below.

.. comp:: Context store
   :id: COMP_SUBJ_STORE
   :crate: agconflo-core

   The component nearly every requirement below is allocated to.

.. comp:: Context store cache
   :id: COMP_SUBJ_CACHE
   :crate: agconflo-core

   Its title begins with the other component's title, so a check that only asks
   whether a statement starts with the title cannot tell the two apart.

.. comp_req:: Control, ubiquitous
   :id: CREQ_OK_UBIQUITOUS
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: ubiquitous
   :statement: Context store shall bind only the contexts wired to a parameter.

   Correct subject. Must not be reported.

.. comp_req:: Control, event
   :id: CREQ_OK_EVENT
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: event
   :statement: When an activation begins, Context store shall bind only wired contexts.

   Correct subject after a trigger. Must not be reported.

.. comp_req:: Control, state
   :id: CREQ_OK_STATE
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: state
   :statement: While a run is paused, Context store shall keep every bound context.

   Correct subject after a state. Must not be reported.

.. comp_req:: Control, optional
   :id: CREQ_OK_OPTIONAL
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: optional
   :statement: Where a global context is declared, Context store shall bind it to every node.

   Correct subject after a feature clause. Must not be reported.

.. comp_req:: Control, unwanted
   :id: CREQ_OK_UNWANTED
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: unwanted
   :statement: If a parameter has no wired context, then Context store shall reject the activation.

   Correct subject after "then". This is the case the first query got wrong.
   Must not be reported.

.. comp_req:: Control, complex
   :id: CREQ_OK_COMPLEX
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: complex
   :statement: While a run is paused, when a person supplies a context, Context store shall bind it.

   Correct subject after a state and a trigger. Must not be reported.

.. comp_req:: Control, the longer title
   :id: CREQ_OK_LONGER_TITLE
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_CACHE
   :ears_pattern: ubiquitous
   :statement: Context store cache shall keep every context it has bound.

   Correct subject for the component whose title extends another's. Must not be
   reported.

.. comp_req:: Offender, the parent restated
   :id: CREQ_BAD_RESTATES_PARENT
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: ubiquitous
   :statement: Agconflo shall bind only the contexts wired to a node's parameters.

   The defect this gate exists for: the parent's statement copied down a level
   with nothing allocated. Every schema rule accepts it. Must be reported.

.. comp_req:: Offender, the other component's title
   :id: CREQ_BAD_PREFIX
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: ubiquitous
   :statement: Context store cache shall keep every context it has bound.

   Allocated to the context store, and about the cache. The statement does begin
   with the allocated component's title, so a prefix test passes it. Must be
   reported.

.. comp_req:: Offender, event
   :id: CREQ_BAD_EVENT
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: event
   :statement: When an activation begins, Agconflo shall bind only wired contexts.

   Must be reported.

.. comp_req:: Offender, state
   :id: CREQ_BAD_STATE
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: state
   :statement: While a run is paused, Agconflo shall keep every bound context.

   Must be reported.

.. comp_req:: Offender, optional
   :id: CREQ_BAD_OPTIONAL
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: optional
   :statement: Where a global context is declared, Agconflo shall bind it to every node.

   Must be reported.

.. comp_req:: Offender, unwanted
   :id: CREQ_BAD_UNWANTED
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: unwanted
   :statement: If a parameter has no wired context, then Agconflo shall reject the activation.

   Must be reported.

.. comp_req:: Offender, complex
   :id: CREQ_BAD_COMPLEX
   :derived_from: FEAT_SUBJ
   :allocated_to: COMP_SUBJ_STORE
   :ears_pattern: complex
   :statement: While a run is paused, when a person supplies a context, Agconflo shall bind it.

   Must be reported.
