=================================
Components of reading and writing
=================================

The three parts ``ARCH_TOPOLOGY`` divides storing a workflow into, and the
requirements allocated to each. As for the wiring feature, a component is an
object rather than a level: it exists so that a requirement has one subject
answerable for it, and each title below is that subject, word for word, because
the gate in ``scripts/gates`` refuses a component requirement whose subject is
anything else.

.. comp:: Topology reader
   :id: COMP_TOPOLOGY_READER
   :crate: agconflo-core

   Turns one document's text into what it describes: a workflow document into the
   instances, bindings, entry nodes and output of a definition, and a node type
   document into the declarations it holds. It draws the line between a fault in
   the text, which it refuses with a location, and a fault in the workflow, which
   it lets through for the validator.

   It hands on the document it read as well as the definition, because the
   writer edits that document rather than writing a new one.

.. comp:: Type catalogue
   :id: COMP_TYPE_CATALOGUE
   :crate: agconflo-core

   The declarations of every node type document it is given, gathered into one
   set keyed by type name, which is what a workflow's instances are resolved
   against. Which documents it is given is its caller's business
   (``DEC_TYPES_IN_OWN_DOCUMENTS``); it neither searches for them nor watches them.

   It answers for the one question no single document can: whether a type name
   is declared twice.

.. comp:: Topology writer
   :id: COMP_TOPOLOGY_WRITER
   :crate: agconflo-core

   Puts a definition into a workflow document, editing the document it was read
   from in place rather than writing a new one, so that everything the definition
   does not carry stays where it was. It answers for what it writes, for what it
   keeps, and for the shapes of the model it refuses to write because a document
   cannot hold them.
