==========================
Fixture: unknown_directive
==========================

.. A directive that is not a declared need type. Markup-level lints reach the
   gate too, not only the needs-aware ones, and this is the cheapest proof of it.

   It used to use `feat_req`, and had to change the moment that became a real
   type - which its own comment predicted. The directive name here is deliberately
   one that will never be declared, so this fixture does not quietly turn into a
   valid need again the next time the metamodel grows.

.. never_a_need_type:: A directive that will never be declared
   :id: NEVER_A_NEED

   A node shall receive exactly the contexts that are wired to it.
