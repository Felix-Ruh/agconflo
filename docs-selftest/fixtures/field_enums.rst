====================
Fixture: field_enums
====================

.. One illegal value for every enumerated field, so this fixture's golden file
   reads as the registry of those value sets. It is one fixture rather than five
   because fields are global and independent: each bad value is reported on its
   own line, naming its own field.

   The messages truncate above three values - `"a", "b" or 4 other candidates` -
   so what is pinned is the first two values and the count. That is still enough
   to force a diff when a set changes, which is the point while ears_pattern's
   set remains provisional.

   Two needs rather than one, so each field sits on a type that would plausibly
   carry it.

.. stkh_req:: A parent, so the requirement below breaks nothing but its enums
   :id: STKH_ENUMS_OK
   :stakeholder: user
   :statement: Agconflo shall reject a value outside a field's enumeration.

.. feat_req:: Illegal values on a requirement
   :id: FEAT_BAD_ENUMS
   :derived_from: STKH_ENUMS_OK
   :ears_pattern: whenever
   :verification_method: vibes
   :stakeholder: shareholder
   :statement: Agconflo shall reject a value outside a field's enumeration.

.. test_case:: Illegal values on a test case
   :id: TEST_BAD_ENUMS
   :verifies: FEAT_BAD_ENUMS
   :test_kind: smoke
   :coverage: mostly

.. dec:: A status outside its value set
   :id: DEC_BAD_STATUS
   :dec_status: maybe
   :decided_on: 2026-08-20
   :statement: Agconflo shall reject a decision status outside its value set.

.. evd:: A kind outside its value set
   :id: EVD_BAD_KIND
   :evd_kind: hearsay
   :observed_on: 2026-08-20
   :observation: This evidence exists to hold the evd_kind value set in a golden file.
