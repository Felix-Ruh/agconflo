===========================
Fixture: field_date_pattern
===========================

.. Dates that are not YYYY-MM-DD. The constraint lives in each field's own schema
   in ubproject.toml rather than in a rule, and a `pattern` there enforces exactly
   as an `enum` does: reported as needs.invalid_field_value at WARNING severity,
   which fails every gate here because they all pass --deny warning.

   BOTH date fields are covered, because they are two declarations and a fixture
   for one would not guard the other.

   Worth knowing while reading this file: the coverage meta-check in
   scripts/docs-selftest.sh cannot demand it. That check walks rule ids in
   schemas.json, and a field-schema constraint has no rule id - so fixtures for
   enums, bounds and patterns are discipline rather than something the harness
   can enforce. This one and field_enums are the whole of that discipline.

.. dec:: A date written the European way round
   :id: DEC_BAD_DATE
   :dec_status: accepted
   :decided_on: 17-08-2026
   :statement: Agconflo shall reject a date that is not written as YYYY-MM-DD.

.. evd:: A date with an unpadded month and day
   :id: EVD_BAD_DATE
   :evd_kind: measurement
   :observed_on: 2026-8-7
   :observation: An unpadded month sorts wrongly as a string, which is why it is refused.
