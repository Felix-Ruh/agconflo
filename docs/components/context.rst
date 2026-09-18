==================
Context components
==================

The components the Context feature decomposes into, all in ``agconflo-core``.

A component is an object rather than a level: nothing is derived from it, and
component requirements are allocated to it. Each title is written to be used as
the grammatical subject of those requirements, which is why none of them carries
an article.

.. comp:: Context value
   :id: COMP_CONTEXT_VALUE
   :crate: agconflo-core

   The immutable value itself: its identifier, its declared type, and its
   content, which is either text or an ordered list of references to other
   values together with the join that combines them.

   It owns everything that is true of one value taken alone - that its content
   reads the same every time, that its parts stay addressable and in order, that
   its type can be asked for. It does not issue its own identifier and it does
   not walk its own ancestry, because both of those are questions about other
   values.

.. comp:: Identifier source
   :id: COMP_IDENTIFIER_SOURCE
   :crate: agconflo-core

   Issues the identifier each new context carries, and is the only thing that
   does. One source spans one run, so uniqueness within a run is a property of
   the source rather than a hope about the values.

   Kept apart from the value because uniqueness is a relation between contexts.
   A value can hold an identifier; it cannot know that no other value in the run
   holds the same one. Keeping issuance in one place is also what lets the
   scheduler take it over once activations exist, without the value changing.

.. comp:: Lineage walker
   :id: COMP_LINEAGE_WALKER
   :crate: agconflo-core

   Reports the ancestry of a context: every value reachable through its parts,
   at any depth, each one once.

   Kept apart from the value because ancestry is a traversal over many values,
   and it fails in ways no single value does. Parts are shared, so one ancestor
   can be reached along several paths and must still be reported once; nesting
   has no fixed depth, so the traversal cannot assume a shallow one.
