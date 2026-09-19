==============================
Components of graph validation
==============================

The two parts ``ARCH_WIRING`` divides validating a workflow into, and the
requirements allocated to each. A component is an object rather than a level:
nothing derives from it, and it exists so that a requirement has one subject
answerable for it - which is also what lets a requirement's grammatical subject
be checked against the title of the component it is allocated to.

Each title below is that subject. "Workflow definition shall ..." reads as a
sentence about one thing, and the gate in ``scripts/gates`` refuses a component
requirement whose subject is anything else.

.. comp:: Wiring validator
   :id: COMP_WIRING_VALIDATOR
   :crate: agconflo-core

   The walk that compares one workflow definition against the node types it names,
   and collects what it finds. Every defect class belongs here: a parameter the
   definition never binds, a name that resolves to nothing, a type that disagrees
   across a wire, and a signature that does not designate exactly one output.

   The definition and the type declarations are its inputs rather than parts of
   this feature - data it reads, and the other layer it reads them against
   (``DEC_TWO_LAYERS``).

   It answers for the completeness of its own report. Continuing after the first
   defect is a property of the walk and of nothing else, which is why it is
   allocated here rather than to what the walk produces.

.. comp:: Wiring defect
   :id: COMP_WIRING_DEFECT
   :crate: agconflo-core

   One thing wrong with one definition: what is wrong, and where. It is a value
   rather than a message, so that a caller can act on it - an agent correcting its
   own workflow reads the location, not the prose.

   Its requirements are true or false of a single defect taken alone, which is
   what separates it from the validator that produced it.
