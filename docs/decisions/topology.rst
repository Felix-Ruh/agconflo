==================================
Decisions about storing a workflow
==================================

How a workflow and the node types it is built from are written down, as
distinct from what a workflow is made of, which ``decisions/workflow`` settles.
These are the choices the loading slice is written against.

Unlike the decisions about what a workflow is, the first two here rest on
measurements, and ``evidence/topology`` holds them. Each was taken against a
named release, so re-opening either starts with re-measuring it. The last two are
judgements that follow from decisions already taken.

.. dec:: Topology is stored as TOML
   :id: DEC_TOPOLOGY_IN_TOML
   :dec_status: accepted
   :decided_on: 2026-09-21
   :supported_by: EVD_TOML_KEEPS_UNKNOWN_KEYS, EVD_TOML_REFUSES_DUPLICATE_KEYS, EVD_TOML_EDIT_KEEPS_COMMENTS, EVD_FORMATS_LOCATE_FAULTS, EVD_JSON_KEEPS_LAST_DUPLICATE, EVD_RON_FLATTEN_REFUSES_STRUCTS, EVD_SERDE_YAML_DEPRECATED
   :statement: Agconflo shall store workflow topology and node type declarations as TOML documents.

   ``STKH_TOPOLOGY_AS_DATA`` names three things data is for: a graph checked
   before anything runs, an editor round-tripping a document without losing what
   it did not understand, and a machine changing it structurally. A fourth comes
   from ``STKH_MACHINE_AUTHORING``: a fault reported well enough for a model to
   correct itself. TOML is the one format measured doing all four. It keeps keys
   the model does not name, refuses a repeated name at the line and column where
   it repeats, locates every other fault the same way, and through ``toml_edit``
   an edit keeps every comment and line it did not touch.

   Three formats lost, each on a measurement rather than a preference:

   - **RON**, the notation written for Rust's own data model. Its struct syntax
     is refused outright once unknown keys are kept, so preserving them means
     writing RON as maps - JSON with trailing commas - and it then keeps the last
     of two duplicate names without a word.
   - **JSON**, which does most of what is asked, and silently keeps the last of
     two instances under one name or two bindings for one parameter. An instance
     disappearing without a report is the failure a validator exists to prevent,
     and recovering it would mean owning a map deserializer. That tool calls
     speak JSON decides nothing: an agent changes a workflow through tools acting
     on the model, and which text the model is stored as is the server's business.
   - **YAML**, whose serde crate of record is marked deprecated.

   The repository already writes its own configuration in TOML - the Cargo
   manifests, ``ubproject.toml``, the nextest and rustfmt settings - so a reader
   of this project already reads it.

   Two costs are accepted. Deep nesting is verbose in TOML, and a workflow is
   deliberately shallow - instances, their bindings, their parameters. And a
   repeated name is a fault in the text rather than a wiring defect, so it stops
   the load before validation and is reported alone. That is the one place a
   document gets refused for less than everything wrong with it, and it is the
   document being unreadable rather than the workflow being wrong.

   The measurements are of toml 1.1.6 and toml_edit 0.25.15. An implementation
   pins those exactly or re-measures first.

.. dec:: Every name is a table key
   :id: DEC_NAMES_AS_KEYS
   :dec_status: accepted
   :decided_on: 2026-09-21
   :supported_by: EVD_TOML_REFUSES_DUPLICATE_KEYS, EVD_TOML_ORDER_NEEDS_FEATURE, EVD_JSON_KEEPS_LAST_DUPLICATE
   :statement: Agconflo shall store each instance, binding, node type and parameter under its name as a table key.

   A name written twice is then a repeated key, and the parser refuses it at the
   line and column where it repeats. Two instances sharing one name - the first of
   the three shapes ``CREQ_VALIDATOR_BINDING_RESOLVES`` left open - cannot be read
   from a document at all. Nor can a parameter bound twice, which
   ``DEC_BINDING_BY_PORT`` already says cannot happen, nor a parameter declared
   twice by one node type.

   The in-memory model is unchanged and still holds lists, because a definition
   built by other means can still carry any of those shapes. The wiring case that
   holds a duplicated instance name to not stopping the walk stays for that
   reason.

   One cost is not optional to pay. Parameters are ordered
   (``DEC_DECLARED_PARAMETERS``), and a table keyed by name hands its keys on in
   the order written only when the parser is built with its ``preserve_order``
   feature and read into an order-keeping map. Without the feature the order is
   alphabetical and nothing says so (``EVD_TOML_ORDER_NEEDS_FEATURE``). An
   implementation has to pin the feature, and a test has to read back a
   declaration written out of alphabetical order, because nothing else would
   notice.

   Arrays of tables, each entry carrying a ``name``, were the alternative. They
   keep order for free, and a repeated name passes the parser - which would need a
   check of its own and would leave the shape readable.

.. dec:: Node types live in documents of their own
   :id: DEC_TYPES_IN_OWN_DOCUMENTS
   :dec_status: accepted
   :decided_on: 2026-09-21
   :statement: Agconflo shall read node type declarations from documents of their own rather than from the workflows that use them.

   ``DEC_TWO_LAYERS`` at the level of files: a node type is defined once and
   used by as many workflows as want it. A workflow document names the types its
   instances are of, and never carries one.

   Which type documents are read is the caller's to say. There is deliberately no
   search path, directory convention or registry yet - each of those is a policy
   with requirements of its own, and none is needed to read a workflow.

   An instance of a type that none of the documents declares is not a fault in
   the text. It reads, and the validator reports it with everything else wrong
   with that workflow, as it already does. A type declared by two documents is
   the question this raises that TOML cannot answer, since the repetition spans
   two documents, and it is a requirement of its own.

   Declaring types inside each workflow was the alternative, and it is the one
   ``DEC_TWO_LAYERS`` turned down: every workflow using a type repeats it, and
   the copies drift.

.. dec:: A workflow names its output with one optional key
   :id: DEC_ONE_OUTPUT_KEY
   :dec_status: accepted
   :decided_on: 2026-09-21
   :statement: Agconflo shall read a workflow's designated output from one optional key naming a single instance.

   Designating two outputs becomes unwritable, which is stronger than reporting
   it. Designating none is an absent key, and it is deliberately not a fault in
   the text: the document reads, and the validator reports the signature defect
   together with every wiring defect beside it, as
   ``FEAT_WIRING_ALL_DEFECTS`` requires. A required key would refuse the
   document for its signature before its wiring was examined, which is the round
   trip that requirement exists to prevent.

   The in-memory model keeps its list of designated outputs. A definition built by
   other means - by an agent through tool calls, say - can still designate
   several, and the validator still refuses that.

   An array of outputs was the alternative. It keeps writable a defect the format
   can simply rule out.
