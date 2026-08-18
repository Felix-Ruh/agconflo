================================
Fixture: rule_link_target_purity
================================

.. MIXED link arrays. Every list below holds one target of the right type and one
   of the wrong type, so each need SATISFIES the presence half of its rule and
   violates only the purity half - which is the case `contains` cannot see, since
   it asks merely that at least one target be right. Before the `items` keyword
   was added beside it, every need in this file passed.

   That is the difference from rule_link_targets, whose links are absent or wholly
   wrong and therefore trip both halves at once. Between them the two fixtures
   cover the whole of each network rule: this one is the purity registry, one
   diagnostic per rule and nothing else.

   `allocated_to` is deliberately absent. A component requirement may be allocated
   to at most one component, so a mixed allocation is already illegal by
   cardinality and its purity can never be observed on its own. Its `items` branch
   is guarded instead by the wholly-wrong single target in rule_link_targets.

.. stkh_req:: A valid parent
   :id: STKH_PURITY
   :stakeholder: user
   :statement: Agconflo shall reject a link array that mixes target types.

.. comp:: A component, used throughout as the wrong-type target
   :id: COMP_PURITY
   :crate: agconflo-core

.. feat_req:: Derived from a stakeholder requirement and a component
   :id: FEAT_PURITY
   :derived_from: STKH_PURITY, COMP_PURITY
   :ears_pattern: ubiquitous
   :verification_method: review
   :statement: Agconflo shall reject a link array that mixes target types.

.. feat_arch:: Realises and uses, each with one wrong target beside a right one
   :id: ARCH_PURITY
   :realises: FEAT_PURITY, COMP_PURITY
   :uses: COMP_PURITY, FEAT_PURITY
   :statement: Agconflo shall reject an architecture link array that is impure.

.. comp_req:: Derived from a feature requirement and a component
   :id: CREQ_PURITY
   :derived_from: FEAT_PURITY, COMP_PURITY
   :allocated_to: COMP_PURITY
   :ears_pattern: ubiquitous
   :statement: A component shall reject a parent list that mixes target types.

.. test_case:: Verifies a component requirement and a component
   :id: TEST_PURITY
   :verifies: CREQ_PURITY, COMP_PURITY
   :test_kind: positive
   :coverage: full
