==============================
Reading and writing a workflow
==============================

The second feature of the engine: a workflow and the node types it is built from,
read from documents and written back to them. Every requirement here derives from
``STKH_TOPOLOGY_AS_DATA`` or ``STKH_MACHINE_AUTHORING``, and each is written
against the decisions in ``decisions/topology`` rather than re-opening them - the
format is TOML, every name is a table key, node types live in documents of their
own, and a workflow names its output with one optional key.

Three of them are about reading. One says a document is read at all, one says
what happens to a document that cannot be, and one - the easiest to leave out -
says what must *not* stop a document being read. Reading is the step before
validation, and a reader that refuses for a wiring defect reports it alone, which
undoes what ``FEAT_WIRING_ALL_DEFECTS`` bought. So the line between a document
that cannot be read and a workflow that is wrong is drawn here, and drawn on the
reader's side as narrowly as it will go.

One is about a set of documents rather than any one of them: a type name declared
twice. TOML refuses a key repeated within a document, and cannot see one repeated
across two.

Three are about writing. Two of them come as a pair for the same reason the
wiring feature's refusals come with a control: keeping what the writer did not
read is satisfied by a writer that writes nothing at all, so what it writes has to
be required as well. The third is what the writer must refuse, because the model
can hold shapes the format rules out, and writing one of them means dropping
something.

Each was checked by hand against the question no rule can ask: could this be
false while its parent is true? The body of each says how.

Deliberately not here: where documents come from. Which files are read is the
caller's to say (``DEC_TYPES_IN_OWN_DOCUMENTS``), so there is no search path,
directory convention or registry, and nothing about watching a document for
changes. Nor is writing a node type document - an agent authoring types is a
requirement for when authoring lands.

.. feat_req:: A workflow is read from its documents
   :id: FEAT_TOPOLOGY_READS
   :derived_from: STKH_TOPOLOGY_AS_DATA
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall read a workflow definition from one workflow document and the node type documents supplied with it.

   The requirement everything else here qualifies. A workflow document holds the
   instances, their bindings, which of them are entry nodes, and the one output
   the workflow designates; the node type documents hold the declarations those
   instances are checked against (``DEC_TYPES_IN_OWN_DOCUMENTS``). Reading them
   produces the same definition the validator already takes.

   It can be false while its parent holds. Topology can be stored as data, and
   that data can be validated without executing it, by a validator handed a
   definition somebody built by hand - while the document beside it says anything
   at all, because nothing reads it. Data that the engine never reads is data in
   name only.

.. feat_req:: A document that cannot be read says where
   :id: FEAT_TOPOLOGY_UNREADABLE_LOCATED
   :derived_from: STKH_MACHINE_AUTHORING
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a workflow document or a node type document cannot be read, then Agconflo shall refuse it naming the document and the line and column of the fault.

   A document cannot be read when it is not TOML, when a value has the wrong
   type, when a key the reader needs is missing, or when a name is repeated
   within it - which, since every name is a table key (``DEC_NAMES_AS_KEYS``), is a
   repeated key and not TOML at all. Each is a fault in the text rather than in
   the workflow, and nothing further can be checked until it is fixed.

   Naming the document matters as soon as there are several: a fault reported at
   line 12 of one of four documents is a fault in none of them in particular.

   It can be false while its parent holds. The parent asks that an agent be able
   to author a workflow and correct itself from what it is told. A reader that
   refuses a broken document with "invalid document" refuses correctly, and leaves
   the agent to search the text. Both libraries measured already give the line and
   column (``EVD_FORMATS_LOCATE_FAULTS``); this requires that they reach the
   caller rather than being flattened into a message on the way.

.. feat_req:: A wiring defect does not stop a document being read
   :id: FEAT_TOPOLOGY_WIRING_DEFECTS_READ
   :derived_from: STKH_MACHINE_AUTHORING
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall read a workflow document whose only faults are wiring defects.

   The control on the requirement above, and the line between them. An instance
   of a type no document declares, a binding to an instance that is not there, a
   wire whose ends declare different types, and no designated output at all are
   all wiring defects: the text is fine, the workflow is wrong, and the validator
   reports every one of them together.

   It can be false while its parent holds, and it is the easiest requirement in
   this document to break by being thorough. A reader that checks each instance's
   type while reading reports the first unknown type and stops, and the author
   learns about the second after fixing the first - the round trip
   ``FEAT_WIRING_ALL_DEFECTS`` exists to prevent, reintroduced one layer earlier.
   The requirement above is satisfied by that reader too, since it refuses
   something and says where.

   A missing output is on this side of the line deliberately
   (``DEC_ONE_OUTPUT_KEY``), and so is an unknown type
   (``DEC_TYPES_IN_OWN_DOCUMENTS``).

.. feat_req:: A node type is declared once
   :id: FEAT_TOPOLOGY_TYPE_DECLARED_ONCE
   :derived_from: STKH_TOPOLOGY_AS_DATA
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If two node type documents declare one type name, then Agconflo shall refuse them naming both documents.

   The one repetition TOML cannot see. Within a document a repeated type name is
   a repeated key and cannot be read; across two documents each is valid on its
   own, and the conflict exists only in the set.

   It can be false while its parent holds. Every document is data, and each is
   checked as it is read - the parser refuses any name repeated within it. The
   name declared in two of them still gets through, and whichever declaration is
   read last replaces the other without a word, which is what JSON was measured
   doing with a repeated key (``EVD_JSON_KEEPS_LAST_DUPLICATE``). The workflow is
   then checked against a declaration its author may not have meant, and passes.

   Both documents are named because either could be the wrong one, and a report
   naming only the second sends the author to a document that may be correct.

.. feat_req:: A written workflow reads back as itself
   :id: FEAT_TOPOLOGY_WRITES
   :derived_from: STKH_TOPOLOGY_AS_DATA
   :ears_pattern: event
   :verification_method: test
   :statement: When Agconflo writes a definition into a workflow document, Agconflo shall write it so that reading the document back gives the same name, instances, bindings and output.

   The positive half of writing. A definition that has been changed - an instance
   added, a binding repointed, a different output designated - is written, and
   reading the result with the same node type documents gives the changed
   definition, no more and no less. The node types are named rather than written:
   they live in documents of their own (``DEC_TYPES_IN_OWN_DOCUMENTS``), which is
   why the statement lists what reads back instead of saying the definition does.

   It can be false while its parent holds. The parent's round trip is phrased as
   not losing what an editor did not understand, and a writer that leaves the
   document untouched loses nothing at all. It satisfies the requirement below
   completely, and writes none of the change it was asked to make.

   Only workflow documents are written here. Writing a node type document is
   authoring a type, which is a requirement for when authoring lands.

.. feat_req:: A shape the format cannot hold is not written
   :id: FEAT_TOPOLOGY_UNWRITABLE_REFUSED
   :derived_from: STKH_TOPOLOGY_AS_DATA
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a definition carries a shape that a workflow document cannot express, then Agconflo shall refuse to write it naming that shape.

   The in-memory model holds shapes the format rules out, deliberately: two
   instances sharing a name, a parameter bound twice, and more than one designated
   output are all representable there (``DEC_NAMES_AS_KEYS``,
   ``DEC_ONE_OUTPUT_KEY``), because a definition built by other means can carry
   them and the validator has to see them to refuse them. A document cannot hold
   any of the three.

   It can be false while its parent holds, and while the requirement above holds
   for every definition that can be written. A writer given two instances named
   ``a`` writes one of them, the document reads back cleanly as a workflow with one
   ``a``, and the other has gone without a word - which is what JSON was measured
   doing on the way in (``EVD_JSON_KEEPS_LAST_DUPLICATE``), here on the way out.
   Refusing, and naming which shape and where, is the only answer that loses
   nothing.

.. feat_req:: Writing keeps what was not read
   :id: FEAT_TOPOLOGY_KEEPS_UNREAD
   :derived_from: STKH_TOPOLOGY_AS_DATA
   :ears_pattern: event
   :verification_method: test
   :statement: When Agconflo writes a definition into the workflow document it was read from, Agconflo shall keep every key and comment of that document it did not read.

   The round trip the parent names: an editor can read a document and write it
   back without losing what it did not understand. A visual editor keeps node
   positions in a document, a person keeps comments, a later version of this
   engine may add keys this one has never heard of. None of that is the reader's,
   and all of it has to survive a write.

   It can be false while its parent holds - or rather, while the requirement above
   holds, which is the sharper way to put it. A writer that regenerates the whole
   document from the definition writes something that reads back exactly as that
   definition, and drops every comment and every key it did not read. Measured
   both ways: a document regenerated through serde came back with none of its
   comments, and an edit in place through ``toml_edit`` kept every byte it did not
   change (``EVD_TOML_EDIT_KEEPS_COMMENTS``).

   Keys are kept wherever they sit - at the top of the document or on an instance -
   because an editor's data attaches to whatever it describes. A bindings table
   is the one place with no such keys: every key in it names a parameter, so every
   key in it is read.

.. feat_arch:: Storage splits into a reader, a writer and a type catalogue
   :id: ARCH_TOPOLOGY
   :realises: FEAT_TOPOLOGY_READS, FEAT_TOPOLOGY_UNREADABLE_LOCATED, FEAT_TOPOLOGY_WIRING_DEFECTS_READ, FEAT_TOPOLOGY_TYPE_DECLARED_ONCE, FEAT_TOPOLOGY_WRITES, FEAT_TOPOLOGY_UNWRITABLE_REFUSED, FEAT_TOPOLOGY_KEEPS_UNREAD
   :uses: COMP_TOPOLOGY_READER, COMP_TOPOLOGY_WRITER, COMP_TYPE_CATALOGUE
   :statement: Agconflo shall allocate reading and writing a workflow to the topology reader, the topology writer and the type catalogue.

   Three components, each answerable for what the others cannot guarantee:

   - The topology reader answers for ``FEAT_TOPOLOGY_READS``,
     ``FEAT_TOPOLOGY_UNREADABLE_LOCATED`` and
     ``FEAT_TOPOLOGY_WIRING_DEFECTS_READ``. All three are about one document and
     what becomes of its text, and the line between the last two - a fault in the
     text refused, a fault in the workflow let through - is a line one component
     has to draw, or two components draw it in different places.
   - The type catalogue answers for ``FEAT_TOPOLOGY_TYPE_DECLARED_ONCE``: it
     gathers the declarations of every node type document it is given into one set
     keyed by type name, and refuses a name that two of them declare.
   - The topology writer answers for ``FEAT_TOPOLOGY_WRITES``,
     ``FEAT_TOPOLOGY_KEEPS_UNREAD`` and ``FEAT_TOPOLOGY_UNWRITABLE_REFUSED``: what
     it writes, what it keeps, and what it will not write.

   **Reading and writing are separate because they fail separately.** A reader can
   be perfect while the writer regenerates the document and drops every comment;
   a writer can keep every byte while the reader stops at the first unknown type.
   One component owning both would carry requirements that no single piece of its
   behaviour answers for, which is what an allocation exists to prevent.

   **The catalogue is separate from the reader for the reason the wiring feature
   separated the defect from the validator.** Everything the reader answers for is
   true or false of one document; a type declared twice is true of a set and of no
   document in it. Folding it into the reader would put a question about several
   documents on a component whose other requirements are each about one, and a
   reader correct on every document alone would then own a failure that only
   appears when two are read together.

   **The writer needs the document the definition was read from**, not only the
   definition. Keeping what the reader did not read means editing that document
   in place (``EVD_TOML_EDIT_KEEPS_COMMENTS``), because regenerating it from the
   definition loses every comment and every key the definition does not carry.
   How the reader hands the document on is the component requirements' business,
   one level down.

   **Nothing here validates.** A definition read from documents goes to the wiring
   validator exactly as one built by hand does, and the reader's requirement to let
   wiring defects through is what keeps that true. The model the reader produces
   is the one ``workflow.rs`` already defines - data rather than a component, as
   ``ARCH_WIRING`` argued - which is also why ``DEC_NAMES_AS_KEYS`` could leave it
   unchanged.

   **All three sit in agconflo-core.** A crate of their own would keep the core
   free of a TOML dependency, and it was considered. It is not taken now because a
   second traced crate is a tooling change rather than a requirements one: a second
   ``[codelinks.projects]`` block in ``ubproject.toml``, a second
   ``docs/code/<crate>.rst``, and an importer that today names exactly one crate.
   That belongs in an ``infra/`` branch of its own, and the moment for it is when
   a second crate - the MCP server - needs to read a workflow too.

   The decisions this is built against are named here rather than linked:

   - ``DEC_TOPOLOGY_IN_TOML``: what the reader reads and the writer writes.
   - ``DEC_NAMES_AS_KEYS``: why a repeated name within one document is the
     reader's fault in the text, and why the writer has shapes it cannot write.
   - ``DEC_TYPES_IN_OWN_DOCUMENTS``: why there is a catalogue at all, and why an
     unknown type is not the reader's to refuse.
   - ``DEC_ONE_OUTPUT_KEY``: why a missing output reads, and why several cannot be
     written.
