===================
Fixture: dead_link
===================

.. A link to an id that does not exist. This is the one fixture exercising link
   resolution rather than a single need's own fields, and it will matter more
   from the moment the real link types arrive: every link in this project points
   up the V, so a dangling one means a level was skipped or renamed.

.. stkh_req:: A link to an id that does not exist
   :id: STKH_DANGLING
   :links: STKH_NOT_PRESENT

   A node shall receive exactly the contexts that are wired to it.
