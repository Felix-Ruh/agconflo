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

   Most network rules here fire when the link is absent as well as when it
   resolves to the wrong type, so these needs cover both halves at once. The two
   decision links are the exception: they are typed with `items` and no
   `contains`, because a decision resting on reasoning rather than on a
   measurement is the normal case, so there is no absence half to cover.

.. stkh_req:: A stakeholder requirement, used as a parent and as a wrong target
   :id: STKH_TARGETS
   :stakeholder: user
   :statement: Agconflo shall reject a link that resolves to the wrong type.

.. comp:: A component, used as a wrong target
   :id: COMP_TARGETS
   :crate: agconflo-core

.. feat_req:: Derived from a component rather than a stakeholder requirement
   :id: FEAT_WRONG_PARENT
   :derived_from: COMP_TARGETS
   :ears_pattern: ubiquitous
   :verification_method: review
   :statement: Agconflo shall reject a link that resolves to the wrong type.

.. feat_arch:: Realises a component and uses a requirement, both inverted
   :id: ARCH_WRONG_TARGETS
   :realises: COMP_TARGETS
   :uses: FEAT_WRONG_PARENT
   :statement: Agconflo shall reject an architecture decision wired backwards.

.. comp_req:: Derived from a stakeholder requirement, skipping the feature level
   :id: CREQ_WRONG_PARENT
   :derived_from: STKH_TARGETS
   :allocated_to: COMP_TARGETS
   :ears_pattern: ubiquitous
   :statement: A component shall reject a requirement that skips a level.

.. comp_req:: Allocated to a requirement rather than a component
   :id: CREQ_WRONG_ALLOC
   :derived_from: FEAT_WRONG_PARENT
   :allocated_to: FEAT_WRONG_PARENT
   :ears_pattern: ubiquitous
   :statement: A component shall reject an allocation to something that is not one.

.. test_case:: Verifies a component, which is not a requirement
   :id: TEST_WRONG_TARGET
   :verifies: COMP_TARGETS
   :test_kind: positive
   :coverage: full

.. evd:: Evidence, used below as a wrong target for supersedes
   :id: EVD_TARGETS
   :evd_kind: measurement
   :observed_on: 2026-08-20
   :observation: This evidence exists only to be pointed at by the wrong link type.

.. dec:: A decision, used below as a wrong target for supported_by
   :id: DEC_TARGETS
   :dec_status: accepted
   :decided_on: 2026-08-20
   :statement: Agconflo shall reject a decision link that resolves to the wrong type.

.. dec:: Supported by a decision and superseding evidence, both inverted
   :id: DEC_WRONG_TARGETS
   :dec_status: accepted
   :decided_on: 2026-08-20
   :supported_by: DEC_TARGETS
   :supersedes: EVD_TARGETS
   :statement: Agconflo shall reject a decision wired to the wrong kind of need.
