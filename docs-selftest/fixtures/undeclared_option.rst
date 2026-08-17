==========================
Fixture: undeclared_option
==========================

.. An option nobody declared. This is the only fixture that proves the field
   declarations are in force at all: without it, a requirement could carry a
   misspelled field name, satisfy every rule about the field it meant to set, and
   pass. ubc offers a closest match, which is why the typo here is a near miss
   rather than nonsense.

.. stkh_req:: A need carrying an option nobody declared
   :id: STKH_STRAY_OPTION
   :stakeholder: user
   :stakeholdr: user
   :statement: Agconflo shall give a node exactly the contexts wired to it.

   The declared field is set as well, so the only defect is the stray option -
   otherwise this fixture would also trip the mandatory-field rule and stop
   being about option declarations.
