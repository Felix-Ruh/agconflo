====================
Fixture: id_required
====================

.. A need with no id. ubc's own default for id_required is false, under which a
   need with no id is accepted and silently given a content-derived one - so if
   this fixture ever goes quiet, the project's requirement ids have stopped
   being stable and every link to them is at risk.

.. stkh_req:: A need with no id at all

   A node shall receive exactly the contexts that are wired to it.
