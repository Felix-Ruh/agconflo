=================
Fixture: id_regex
=================

.. An id that does not match the project's pattern. The expected message quotes
   the pattern itself, so this fixture also proves the inherited regex is ours
   and not ubc's default, which is unanchored at the end.

.. stkh_req:: An id that does not match the pattern
   :id: stkh_lowercase

   A node shall receive exactly the contexts that are wired to it.
