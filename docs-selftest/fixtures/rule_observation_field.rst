===============================
Fixture: rule_observation_field
===============================

.. Defects in an observation's VALUE rather than in what it says - the evidence
   counterpart of rule_statement_field: one below the lower bound, one above the
   upper, one wrapped over two source lines, and one carrying a character that is
   not plain ASCII.

   The last two are why evidence needs its own copies of two rules that already
   existed for statement. Both of those select on the FIELD, and an evidence need
   carries no statement at all, so without the twins an observation would be the
   one normative value in this project still able to hide a non-breaking space.
   That was measured before the twins were written: an observation carrying one
   produced no diagnostic whatsoever.

   The wrapped case reports ONCE here where the statement equivalent reports
   twice. An observation is held to no anchored grammar, so there is no second
   rule for the newline to break - which is also why the one-line rule for this
   field rests on the honest data constraint rather than on a baffling near-miss.

.. evd:: An observation below the lower bound
   :id: EVD_TOO_SHORT
   :evd_kind: measurement
   :observed_on: 2026-08-18
   :observation: TBD

.. evd:: An observation wrapped over two source lines
   :id: EVD_WRAPPED
   :evd_kind: measurement
   :observed_on: 2026-08-18
   :observation: The warm index took 41 ms and the cold index took 80 ms,
      measured on this machine.

.. evd:: An observation carrying a non-breaking space
   :id: EVD_NBSP
   :evd_kind: measurement
   :observed_on: 2026-08-18
   :observation: The warm index took 41 ms and the cold index took 80 ms.

.. evd:: An observation above the upper bound
   :id: EVD_TOO_LONG
   :evd_kind: measurement
   :observed_on: 2026-08-18
   :observation: ubc check was measured at one, five, six, ten and fifty thousand needs across two operating systems, three network conditions and both a cold and a warm licence cache, and every one of those runs is recorded here at a length no bound would allow.
