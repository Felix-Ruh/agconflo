==========================
Fixture: rule_link_targets
==========================

.. Links pointing at needs of the wrong type. Every need here is otherwise
   conformant - right prefix, every mandatory field - so the only defect is what
   each link resolves to.

   The interesting case is CREQ_WRONG_PARENT, which derives from a stakeholder
   requirement rather than a feature requirement. It is well formed, it is a real
   link, and it skips a level of the V - which is the mistake this rule exists to
   catch, and the one a reader would most easily miss.

   A network rule fires when the link is absent as well as when it resolves to the
   wrong type, so these fixtures cover both halves of each rule.

.. stkh_req:: A stakeholder requirement, used as a parent and as a wrong target
   :id: STKH_TARGETS
   :stakeholder: user
   :statement: Agconflo shall reject a link that resolves to the wrong type.

   Body.

.. comp:: A component, used as a wrong target
   :id: COMP_TARGETS
   :crate: agconflo-core

   Body.

.. feat_req:: Derived from a component rather than a stakeholder requirement
   :id: FEAT_WRONG_PARENT
   :derived_from: COMP_TARGETS
   :ears_pattern: ubiquitous
   :verification_method: review
   :statement: Agconflo shall reject a link that resolves to the wrong type.

   Body.

.. feat_arch:: Realises a component and uses a requirement, both inverted
   :id: ARCH_WRONG_TARGETS
   :realises: COMP_TARGETS
   :uses: FEAT_WRONG_PARENT
   :statement: Agconflo shall reject an architecture decision wired backwards.

   Body.

.. comp_req:: Derived from a stakeholder requirement, skipping the feature level
   :id: CREQ_WRONG_PARENT
   :derived_from: STKH_TARGETS
   :allocated_to: COMP_TARGETS
   :ears_pattern: ubiquitous
   :statement: A component shall reject a requirement that skips a level.

   Body.

.. comp_req:: Allocated to a requirement rather than a component
   :id: CREQ_WRONG_ALLOC
   :derived_from: FEAT_WRONG_PARENT
   :allocated_to: FEAT_WRONG_PARENT
   :ears_pattern: ubiquitous
   :statement: A component shall reject an allocation to something that is not one.

   Body.

.. test_case:: Verifies a component, which is not a requirement
   :id: TEST_WRONG_TARGET
   :verifies: COMP_TARGETS
   :test_kind: positive
   :coverage: full

   Body.
