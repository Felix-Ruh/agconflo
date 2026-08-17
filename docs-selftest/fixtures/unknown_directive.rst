==========================
Fixture: unknown_directive
==========================

.. A need type that is not declared. Markup-level lints reach the gate too, not
   only the needs-aware ones, and this is the cheapest proof of it. It also
   inverts usefully: this fixture must START failing differently the moment
   feat_req becomes a declared type, which is a change worth noticing.

.. feat_req:: A type that is not declared yet
   :id: FEAT_NOT_A_TYPE

   A node shall receive exactly the contexts that are wired to it.
