=============================
Fixture: rule_statement_field
=============================

.. Defects in the statement's value rather than its grammar: a wrapped statement,
   one below the lower length bound, one above the upper.

   The wrapped one reports TWICE - once for the newline and once for the grammar
   rule, which the newline also breaks. That is two correct diagnoses of one
   defect, and it is why the newline has a rule of its own: on its own, the
   grammar rule's message would say a statement does not match a pattern it
   plainly does match, with the newline invisible in the middle of it.

   The short and long statements are both well formed otherwise, so each reports
   only its bound.

.. stkh_req:: A statement wrapped over two source lines
   :id: STKH_WRAPPED
   :stakeholder: user
   :statement: Agconflo shall give a node exactly the contexts that were wired
      to it and no others.

.. stkh_req:: A statement below the lower bound
   :id: STKH_TOO_SHORT
   :stakeholder: user
   :statement: It shall work.

.. stkh_req:: A statement above the upper bound
   :id: STKH_TOO_LONG
   :stakeholder: user
   :statement: Agconflo shall record the provenance of every context, including the node that produced it, the activation it belonged to, the parameters it was bound to, the transform that combined it, and anything else a reader might later wish to know about it.
