===================
Fixture: ok_minimal
===================

.. The control. A well formed need must produce no diagnostics at all, so this
   fixture's golden file is empty apart from the exit code. It also proves the
   shared configuration is being inherited: without it, stkh_req would not be a
   declared type and this file would report an unknown directive instead.

.. stkh_req:: A well formed stakeholder requirement
   :id: STKH_WELL_FORMED

   A node shall receive exactly the contexts that are wired to it.
