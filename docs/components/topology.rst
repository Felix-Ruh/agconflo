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
   instances, bindings and output of a definition, and a node type
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

.. comp_req:: A workflow document is read into a definition
   :id: CREQ_READER_WORKFLOW
   :derived_from: FEAT_TOPOLOGY_READS
   :allocated_to: COMP_TOPOLOGY_READER
   :ears_pattern: ubiquitous
   :statement: Topology reader shall read a workflow document and a type catalogue into one workflow definition.

   The document gives the definition its name, its instances - each with the type
   it names and its bindings keyed by parameter - and
   the one output it designates, if any. The catalogue gives it the node types its
   instances are checked against, all of them, whether or not an instance names
   them.

   Failure modes:

   - **A key the model does not name is refused.** Every document an editor has
     annotated is then unreadable, which defeats the round trip the writer exists
     for before the writer is ever reached (``FEAT_TOPOLOGY_KEEPS_UNREAD``). The
     natural way to write a strict reader is exactly this one.
   - **An absent output is read as something.** No designated output is an empty
     list in the definition, and the validator's to report. Reading it as a
     refusal reports the signature alone, and reading it as the last instance, or
     the only one, designates an output the author never named.

   Must read: a document with no bindings anywhere, a document with no instances,
   and a document whose instances carry keys the model does not name.

.. comp_req:: A node type document keeps its parameters in order
   :id: CREQ_READER_TYPES
   :derived_from: FEAT_TOPOLOGY_READS
   :allocated_to: COMP_TOPOLOGY_READER
   :ears_pattern: ubiquitous
   :statement: Topology reader shall read a node type document into declarations whose parameters keep the order they are written in.

   A node assembles its inputs in the order its parameters are declared
   (``DEC_DECLARED_PARAMETERS``), and every name is a table key
   (``DEC_NAMES_AS_KEYS``). So the order of a table's keys is data here, not
   presentation, and it is the one piece of data the obvious implementations lose.

   Failure modes:

   - **The order is lost without a word.** A sorted map loses it, and so does an
     order-keeping map fed by a parser built without its order-preserving feature -
     measured to hand keys on alphabetically and report nothing
     (``EVD_TOML_ORDER_NEEDS_FEATURE``). Nothing downstream can notice: every
     parameter is still there, and every node receives its inputs in the wrong
     order.
   - **The two lists are confused.** A requested global read as a parameter,
     or a parameter as a requested global, changes what the validator demands of
     every instance of that type.

   Must read: a type declaring no parameters of any kind, and a type whose
   parameters are written out of alphabetical order - the only kind of order that
   tells a sorting reader from a faithful one.

.. comp_req:: A document that cannot be read is refused with its place
   :id: CREQ_READER_FAULT_LOCATED
   :derived_from: FEAT_TOPOLOGY_UNREADABLE_LOCATED
   :allocated_to: COMP_TOPOLOGY_READER
   :ears_pattern: unwanted
   :statement: If a document cannot be read, then Topology reader shall refuse it with an error carrying the document's name and the line and column as values.

   As values for the reason a wiring defect carries its place as one
   (``CREQ_DEFECT_NAMES_PLACE``): the caller this is for is an agent correcting its
   own document, and it reads fields rather than prose. The document's name is
   whatever the caller gave it when handing it over, since which documents are read
   is the caller's to decide (``DEC_TYPES_IN_OWN_DOCUMENTS``).

   Failure modes:

   - **The place is only in the message.** Both libraries measured already put it
     there (``EVD_FORMATS_LOCATE_FAULTS``), so passing the library's message on
     looks finished and leaves an agent parsing prose.
   - **The line or column is counted from zero.** The library's own message counts
     from one, and a value one lower than the message beside it sends an author to
     the line above the fault.
   - **The column counts bytes.** The library locates a fault by a span of bytes
     and its message counts the column in characters (``EVD_TOML_EDIT_SPANS``). On
     a line holding a character wider than one byte, a column taken from the span
     alone is past the one the message gives.
   - **The document is not named.** Given four documents, a fault at line 12 of one
     of them is a fault in none in particular.
   - **A value the model refuses is not treated as a fault in the text.** A context
     type name has to be non-empty (``CREQ_VALUE_DECLARED_TYPE``), and an empty one
     is a well-formed TOML string. Refusing it without a location, or accepting it
     by some other route, are both wrong.
   - **Reading panics instead of refusing.** An ``unwrap`` on a value the document
     did not have ends the process, which reports nothing at all.
   - **An optional list read as nothing.** Every parameter is required
     (``DEC_EVERY_INPUT_REQUIRED``), and the reader reads past keys it does not
     know, so a type still declaring ``optional`` would lose those parameters
     without a word, and its script would find them missing
     (``DEC_OPTIONAL_PARAMETERS_REFUSED``).
   - **An entry mark read past.** No instance is an entry
     (``DEC_SIGNATURE_IS_WHAT_NOTHING_BINDS``), so a document still marking one
     says something untrue, and the writer would keep it there
     (``DEC_ENTRY_MARK_REFUSED``).

   Must refuse, each with its place: text that is not TOML, a value of the wrong
   type, a key the reader needs that is missing, an ``optional`` list on a node
   type, at its key, and an empty context type name.

.. comp_req:: Reading resolves no name
   :id: CREQ_READER_NAMES_UNRESOLVED
   :derived_from: FEAT_TOPOLOGY_WIRING_DEFECTS_READ
   :allocated_to: COMP_TOPOLOGY_READER
   :ears_pattern: ubiquitous
   :statement: Topology reader shall read a workflow document without resolving any name it holds.

   Every wiring defect a document can hold but one is a name that resolves to
   nothing, or to something that disagrees with where it is used. A reader that
   resolves no name therefore cannot refuse for any of those, which makes this the
   mechanism behind its parent rather than a restatement of it: the parent says
   wiring defects must not stop reading, and this says what reading must not do
   for that to hold. The one exception - no designated output - is an absent key
   rather than a name, and ``CREQ_READER_WORKFLOW`` is where reading it as no
   output is required. The two wiring defects a document cannot hold, a name
   several instances share and a parameter bound twice, are repeated keys, which
   the parser refuses before anything is read (``DEC_NAMES_AS_KEYS``).

   Failure modes:

   - **An instance's type is looked up while reading.** The first unknown type
     refuses the document, and the author learns of the second after fixing the
     first - the round trip ``FEAT_WIRING_ALL_DEFECTS`` exists to prevent.
   - **A binding's source is looked up while reading.** The same failure through a
     different name.
   - **Wires are typed while reading.** Building a typed wire needs both ends'
     declarations, so a disagreement becomes a refusal in the text.

   Must read, and then be reported in full by the validator: documents carrying
   between them every wiring defect class a document can hold. Two of them,
   because designating no output and designating one that names no instance
   cannot be written in one document.

.. comp_req:: A type name declared twice is refused
   :id: CREQ_CATALOGUE_DECLARED_ONCE
   :derived_from: FEAT_TOPOLOGY_TYPE_DECLARED_ONCE
   :allocated_to: COMP_TYPE_CATALOGUE
   :ears_pattern: unwanted
   :statement: If more than one node type document declares a type name, then Type catalogue shall refuse them naming that type and every document declaring it.

   Every document declaring the name, because any of them may be the wrong one, and
   more than two can declare it.

   Failure modes:

   - **The last declaration read wins.** A map insert replaces the earlier one
     without a word - the behaviour measured in JSON for a repeated key
     (``EVD_JSON_KEEPS_LAST_DUPLICATE``), rebuilt one level up.
   - **The first one wins.** The same failure in the other direction, and the one
     a map entry that keeps its existing value produces.
   - **Identical declarations are merged.** Two byte-identical copies are harmless
     today and drift apart tomorrow, which is why ``DEC_TYPES_IN_OWN_DOCUMENTS``
     keeps one definition per type in the first place.
   - **Only the first repeated name is reported.** With several names each declared
     twice, the author fixes one and learns of the next.
   - **Only the second document is named.** It may be the correct one.

   Must hold, unrefused: two documents declaring different types, one document
   declaring several, and a type no workflow uses.

.. comp_req:: Every change is written, removals included
   :id: CREQ_WRITER_WRITES
   :derived_from: FEAT_TOPOLOGY_WRITES
   :allocated_to: COMP_TOPOLOGY_WRITER
   :ears_pattern: event
   :statement: When Topology writer writes a definition into a workflow document, Topology writer shall make the document's name, instances, bindings and output match the definition, removing what the definition no longer holds.

   Writing into a document rather than producing one: the writer edits in place so
   that the requirement beside this can hold, and every change the definition
   carries has to reach the document through that edit. What this adds to its
   parent is the half of a change an edit in place forgets. Reading back is how it
   is checked, and a match with removals included is what the read has to find.

   A match by name, not by position. Instances and bindings keep the order the
   document gives them, and one the definition adds goes after the others
   (``DEC_DOCUMENT_KEEPS_ITS_ORDER``), so the document read back holds exactly the
   definition's instances and bindings, each compared under its name.

   Failure modes:

   - **Only what exists is updated.** An edit that rewrites the tables already in
     the document writes a repointed binding, and never writes an instance that
     was added or removes one that was deleted. Both are edits the library makes in
     place (``EVD_TOML_EDIT_WHOLE_TABLES``), so leaving them out is the writer's
     failure rather than the format's.
   - **A removed binding or output stays behind.** The document then reads back
     with a wire or a designation the definition no longer has.
   - **A renamed instance is written twice.** Its new table is added and its old
     one kept, so the document reads back with both.

   Must write: a definition with an instance added, one removed, one renamed, a
   binding repointed, a binding removed, and the output changed and removed - each
   read back and compared with the definition that was written.

.. comp_req:: Writing leaves what it did not read in place
   :id: CREQ_WRITER_KEEPS_UNREAD
   :derived_from: FEAT_TOPOLOGY_KEEPS_UNREAD
   :allocated_to: COMP_TOPOLOGY_WRITER
   :ears_pattern: event
   :statement: When Topology writer writes a definition into the document it was read from, Topology writer shall leave every key and comment the definition does not carry in place.

   In place rather than merely present: a comment moved away from the line it
   describes has lost what it was for, and an editor's data attaches to what it
   annotates.

   Failure modes:

   - **The document is regenerated from the definition.** Every comment goes -
     measured (``EVD_TOML_EDIT_KEEPS_COMMENTS``) - and so does every key the
     definition does not carry, since the definition has nowhere to hold one. It is
     also the implementation that makes the requirement beside this easiest to
     meet.
   - **An instance's table is rewritten whole when one of its bindings changes.**
     The keys on that instance that the model does not name go with it, while every
     other instance keeps its own - which is why a test changing nothing would not
     see it.
   - **A changed value loses the comment beside it.** Assigning a new value drops
     the comment after the old one (``EVD_TOML_EDIT_ASSIGNING_LOSES_FORMAT``), so a
     writer changing exactly the right value still loses what annotated it.
   - **A value is written although it has not changed.** A value written anew is
     quoted the default way, so ``'sink'`` comes back as ``"sink"``
     (``EVD_TOML_EDIT_ASSIGNING_LOSES_FORMAT``), and a document asked to change
     nothing changes.
   - **Line endings are rewritten.** The library writes every line ending as LF
     (``EVD_TOML_EDIT_WRITES_LF``), so a document saved on Windows comes back with
     every line changed.

   What belongs to an instance goes with it when the definition removes it: its
   keys, the ones the model does not name included, and the comments above it and
   at the end of its line, which the library holds as part of it
   (``EVD_TOML_EDIT_WHOLE_TABLES``).
   None of that is carried by the definition, and none of it has anywhere left to
   stay. A renamed instance is, as far as a definition can say, one removed and
   one added, so it keeps none of them either.

   Must keep: a document written back with nothing changed is the same text, byte
   for byte, whichever line endings it was written with.

.. comp_req:: A shape the format cannot hold is refused before writing
   :id: CREQ_WRITER_UNWRITABLE_REFUSED
   :derived_from: FEAT_TOPOLOGY_UNWRITABLE_REFUSED
   :allocated_to: COMP_TOPOLOGY_WRITER
   :ears_pattern: unwanted
   :statement: If a definition holds a shape no workflow document can express, then Topology writer shall refuse to write it naming every such shape it holds.

   The shapes are three: two instances sharing a name, a parameter bound twice on
   one instance, and more than one designated output (``DEC_NAMES_AS_KEYS``,
   ``DEC_ONE_OUTPUT_KEY``). Every one of them it holds is named, for the reason the
   validator reports every defect: fixing one to learn of the next is a round trip.

   Failure modes:

   - **One of two same-named instances is written.** The document reads back
     cleanly with one instance where the definition had two, and the other has
     gone.
   - **The first designated output is written.** The same loss for the signature.
   - **The document is half edited when the refusal comes.** A refusal leaves the
     document as it was, because a document partly written is one nobody asked for.
   - **A wiring defect is refused as unwritable.** An instance of an unknown type,
     a binding to an instance that is not there and no designated output can all be
     written, and a writer that refuses them is acting as a validator it is not.
     Two wiring defects are unwritable shapes as well - a name two instances share
     and a parameter bound twice - and those are refused for being unwritable, not
     for being defects.

   Must write, unrefused: a definition carrying every wiring defect a document can
   express.
