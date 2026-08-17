=====================
Fixture: duplicate_id
=====================

.. Two needs claiming the same id. Both needs live in this one file because a
   fixture is checked as a one-file project - see scripts/docs-selftest.sh. The
   diagnostic carries two source locations rather than one, which is also what
   makes this fixture worth having: it is the only shape here that does.

.. stkh_req:: First use of the id
   :id: STKH_DUPLICATE
   :stakeholder: user
   :statement: Agconflo shall give a node exactly the contexts wired to it.

.. stkh_req:: Second use of the same id
   :id: STKH_DUPLICATE
   :stakeholder: user
   :statement: Agconflo shall give a node exactly the contexts wired to it.
