=========================
Fixture: rule_ears_shape
=========================

.. One requirement per EARS pattern, each labelled with a pattern and written in
   a different one's grammar. Covers all six grammar rules.

   The defects are the mistakes an author actually makes rather than nonsense:
   the unwanted case drops the "then" that separates it from a state-driven
   requirement, and the complex case gives only one of its two clauses. The event,
   state and optional cases each state a perfectly good ubiquitous requirement
   under the wrong label - which is the failure the pattern field exists to catch,
   since without it every one of these would pass.

   All six share one parent, carry their mandatory fields, and sit inside the
   length bounds, so the only thing wrong with each is its grammar.

.. stkh_req:: A parent for all six
   :id: STKH_EARS
   :stakeholder: maintainer
   :statement: Agconflo shall hold a statement to the grammar of its declared pattern.

   Body.

.. feat_req:: Labelled ubiquitous, but states rather than obliges
   :id: FEAT_EARS_UBIQUITOUS
   :derived_from: STKH_EARS
   :ears_pattern: ubiquitous
   :verification_method: review
   :statement: Agconflo records the provenance of every context.

   Body.

.. feat_req:: Labelled event, but has no trigger clause
   :id: FEAT_EARS_EVENT
   :derived_from: STKH_EARS
   :ears_pattern: event
   :verification_method: test
   :statement: Agconflo shall bind contexts when an activation begins.

   Body.

.. feat_req:: Labelled state, but has no While clause
   :id: FEAT_EARS_STATE
   :derived_from: STKH_EARS
   :ears_pattern: state
   :verification_method: test
   :statement: Agconflo shall treat a node parked on a human gate as live.

   Body.

.. feat_req:: Labelled optional, but has no Where clause
   :id: FEAT_EARS_OPTIONAL
   :derived_from: STKH_EARS
   :ears_pattern: optional
   :verification_method: analysis
   :statement: Agconflo shall order fragments by their cached prefix.

   Body.

.. feat_req:: Labelled unwanted, but drops the then
   :id: FEAT_EARS_UNWANTED
   :derived_from: STKH_EARS
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a join receives contexts from two activations, Agconflo shall withhold it.

   Body.

.. feat_req:: Labelled complex, but gives only one clause
   :id: FEAT_EARS_COMPLEX
   :derived_from: STKH_EARS
   :ears_pattern: complex
   :verification_method: test
   :statement: While a loop is active, Agconflo shall end the iteration.

   Body.
