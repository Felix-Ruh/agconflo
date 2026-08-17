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

.. feat_req:: Illegal values on a requirement
   :id: FEAT_BAD_ENUMS
   :ears_pattern: whenever
   :verification_method: vibes
   :stakeholder: shareholder
   :statement: Agconflo shall reject a value outside a field's enumeration.

   Body.

.. test_case:: Illegal values on a test case
   :id: TEST_BAD_ENUMS
   :test_kind: smoke
   :coverage: mostly

   Body.
