=================
Fixture: id_regex
=================

.. An id that does not match the project's pattern. The expected message quotes
   the pattern itself, so this fixture also proves the inherited regex is ours
   and not ubc's default, which is unanchored at the end.

.. stkh_req:: An id that does not match the pattern
   :id: STKH_lowercase
   :stakeholder: user
   :statement: Agconflo shall give a node exactly the contexts wired to it.

   The prefix is deliberately correct: only the lowercase tail is at fault, so
   this fixture stays about id_regex and not about the per-type prefix rule.
