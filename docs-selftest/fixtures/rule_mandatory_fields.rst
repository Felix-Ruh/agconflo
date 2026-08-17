==============================
Fixture: rule_mandatory_fields
==============================

.. One need of every type with its mandatory fields missing, covering all six
   mandatory-field rules. Ids and links are correct throughout, so the only
   defect in each need is the absent field - and `required` reports one
   diagnostic per missing FIELD, not one per need, which is why the golden file
   holds eleven lines for six needs.

   As in rule_id_prefix, the needs link to each other: a link rule checks its
   target's TYPE, which is right here even though the targets are themselves
   incomplete.

   Note what `required` means: the AUTHOR must have set the field. A default does
   not count as present, which is what makes these rules work at all.

.. stkh_req:: No statement, no stakeholder
   :id: STKH_NO_FIELDS

   Body.

.. feat_req:: No statement, pattern or verification method
   :id: FEAT_NO_FIELDS
   :derived_from: STKH_NO_FIELDS

   Body.

.. feat_arch:: No statement
   :id: ARCH_NO_FIELDS
   :realises: FEAT_NO_FIELDS
   :uses: COMP_NO_FIELDS

   Body.

.. comp:: No crate
   :id: COMP_NO_FIELDS

   Body.

.. comp_req:: No statement or pattern
   :id: CREQ_NO_FIELDS
   :derived_from: FEAT_NO_FIELDS
   :allocated_to: COMP_NO_FIELDS

   Body.

.. test_case:: No kind or coverage
   :id: TEST_NO_FIELDS
   :verifies: CREQ_NO_FIELDS

   Body.
