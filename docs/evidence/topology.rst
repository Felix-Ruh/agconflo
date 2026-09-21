===========================================
Evidence about storing a workflow as a file
===========================================

Measurements the choice of a storage format rests on. All of them were taken on
the same day in two throwaway crates outside the repository, against the newest
release of every library involved, and each names the version it was taken
against because a format library's behaviour is exactly what moves between
releases.

The first crate read one small document into a structure shaped like the
workflow a loader would produce - a ``name``, a table of instances keyed by
instance name, each instance carrying a ``node_type`` and a table of bindings
keyed by parameter - with ``#[serde(flatten)]`` collecting every key the
structure did not name into a map, at the document level and at the instance
level. The same document was written in JSON, in RON and in TOML, and the same
four faults were introduced into each. The second crate read node type
declarations whose parameters were tables keyed by parameter name. Both used
serde 1.0.229.

.. evd:: TOML keeps keys the model does not name
   :id: EVD_TOML_KEEPS_UNKNOWN_KEYS
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: toml 1.1.6 read keys missing from the model into a flattened map at document and instance level, wrote them back, and the re-read document equalled the first.

   The document carried an ``[editor]`` table at the top level and a
   ``position`` array on one instance, neither of which the structure named.
   Both were collected, written back in the output, and compared equal after a
   second read.

   Two things were not preserved, and neither was expected to be: the order of
   the collected keys, which came back sorted because the map collecting them was
   a ``BTreeMap``, and an empty table of bindings, which was written out as an
   empty ``[instances.a.bindings]`` header rather than left out. The second is a
   serialisation setting, not a loss.

.. evd:: TOML refuses a repeated key and says where
   :id: EVD_TOML_REFUSES_DUPLICATE_KEYS
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: toml 1.1.6 refused a second table under one instance name, a parameter bound twice and a parameter declared twice, each naming the line and column of the repeated key.

   Three shapes, each refused by the parser before any structure saw it:

   - a second ``[instances.a]`` table, at line 6, column 12;
   - ``bindings = { input = "a", input = "c" }``, at line 5, column 27;
   - ``required = { apple = "note", apple = "diff" }`` in a node type
     declaration, at line 3, column 30.

   Each message reads ``TOML parse error at line L, column C``, quotes the line,
   marks the repeated key beneath it, and ends ``duplicate key``.

   The TOML specification forbids defining a key twice, so this is the format
   rather than a setting of one library. It is what makes a name that resolves to
   two things unwritable in a document, where the in-memory model can still hold
   one.

.. evd:: TOML keeps declaration order only with a feature switched on
   :id: EVD_TOML_ORDER_NEEDS_FEATURE
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: toml 1.1.6 gave keys to an indexmap 2.14.2 map in document order only with its preserve_order feature, and in alphabetical order without it, reporting nothing either way.

   Parameters declared as ``zebra``, ``apple``, ``mango`` in an inline table and
   as ``source``, ``beta``, ``alpha`` in a dotted table. With ``preserve_order``
   enabled, an ``IndexMap`` received both in the order written. With the feature
   off, the same ``IndexMap`` received ``apple``, ``mango``, ``zebra`` and
   ``alpha``, ``beta``, ``source`` - the order of the parser's own sorted table,
   handed on without a word. A ``BTreeMap`` sorted them either way, as it must.

   Declaration order is load-bearing here: a node assembles its inputs in the
   order its parameters are declared (``DEC_DECLARED_PARAMETERS``). So a build
   of the loader without that one feature flag reads every declaration, reports
   nothing, and hands every node its inputs in the wrong order.

.. evd:: JSON keeps the last of two duplicates without a word
   :id: EVD_JSON_KEEPS_LAST_DUPLICATE
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: serde_json 1.0.151 accepted two instances under one key and two bindings for one parameter without an error, keeping the last of each.

   ``{"a": {"node_type": "source"}, "a": {"node_type": "sink"}}`` read as one
   instance ``a`` of type ``sink``, and ``{"input": "a", "input": "c"}`` as one
   binding to ``c``. The first instance and the first binding were gone, and
   nothing said so.

   Otherwise it did everything asked of it: keys the structure did not name
   survived a round trip, and every fault was located. Refusing duplicates would
   need a hand-written map deserializer, which is code this project would own to
   recover what the TOML parser does by itself.

.. evd:: RON refuses its own struct syntax once unknown keys are kept
   :id: EVD_RON_FLATTEN_REFUSES_STRUCTS
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: ron 0.12.2 refused a document in RON struct syntax at 1:1 once unknown keys were flattened into a map, and read it only when the document was written as a map.

   Every one of the five inputs written in RON's struct syntax failed with the
   same message, because a flattened field makes serde ask for a map::

     1:1: Expected opening `{`

   Rewritten with the document as a map, it read and round-tripped, and wrote its
   output back as maps: the struct ``(zoom: 2)`` came back as a map and the tuple
   ``(10, 20)`` as a list. What is left is JSON with trailing commas.

   Written as a map, it also kept the last of two instances under one name
   without an error, as JSON does.

.. evd:: toml_edit keeps what an edit did not touch
   :id: EVD_TOML_EDIT_KEEPS_COMMENTS
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: toml_edit 0.25.15 changed one binding in place and wrote the document back byte for byte as it was read, apart from that one value.

   A document with a leading comment, a comment after a table header and a
   comment after a binding. The binding ``input`` was set from ``"a"`` to
   ``"c"`` through the document's own index, the result was written out, and it
   was compared with the input text after substituting that one value: equal.
   All three comments and every line's spacing survived.

   One edit of one value was measured. Adding or removing a whole instance was
   not, and is the first thing to measure before the writer's behaviour is built
   on this.

.. evd:: JSON and TOML both locate a fault by line and column
   :id: EVD_FORMATS_LOCATE_FAULTS
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: serde_json 1.0.151 and toml 1.1.6 both named the line and column of a syntax fault, a missing key and a value of the wrong type.

   The messages for an unclosed table, an instance without ``node_type`` and
   ``bindings = 5``, in that order::

     serde_json  EOF while parsing an object at line 6 column 1
     serde_json  missing field `node_type` at line 4 column 25
     serde_json  invalid type: integer `5`, expected a map at line 4 column 46
     toml        TOML parse error at line 3, column 13 ... unclosed table, expected `]`
     toml        TOML parse error at line 3, column 1 ... missing field `node_type`
     toml        TOML parse error at line 5, column 12 ... invalid type: integer `5`, expected a map

   toml also quotes the offending line and marks the column beneath it.

   RON is left out: every RON input failed at ``1:1`` for the reason recorded
   beside this, so its locations were never exercised.

.. evd:: serde_yaml is deprecated
   :id: EVD_SERDE_YAML_DEPRECATED
   :evd_kind: vendor_doc
   :observed_on: 2026-09-21
   :observation: The newest release of serde_yaml on crates.io is 0.9.34+deprecated, and the crate was last updated on 2024-03-25.

   Read from the crates.io registry rather than measured. It is recorded because
   YAML is the format a reader would otherwise ask about, and a format whose serde
   crate of record is marked deprecated by its author is not one to build a
   loader on.
