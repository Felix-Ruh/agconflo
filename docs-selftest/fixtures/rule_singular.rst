=======================
Fixture: rule_singular
=======================

.. The three ways a statement stops being singular, one need each. They are three
   patterns of one rule rather than three rules, and each is pinned separately
   because a golden file records the pointer into the rule - allOf/0, allOf/1,
   allOf/2 - so dropping any one of them reddens this fixture.

   The third pattern is a period followed by a SPACE rather than any period at
   all. That distinction is load-bearing: it lets the grammar rules accept a
   quantified statement like "within 0.5 seconds", which an outright ban on
   internal periods would have rejected - a false positive on exactly the
   requirements worth writing.

.. stkh_req:: Two obligations in one sentence
   :id: STKH_TWO_OBLIGATIONS
   :stakeholder: user
   :statement: Agconflo shall record provenance and shall render it on request.

.. stkh_req:: Two clauses joined by a semicolon
   :id: STKH_SEMICOLON
   :stakeholder: user
   :statement: Agconflo shall record provenance; the viewer renders it.

.. stkh_req:: Two sentences in one statement
   :id: STKH_TWO_SENTENCES
   :stakeholder: user
   :statement: Agconflo shall record provenance. The viewer renders it.
