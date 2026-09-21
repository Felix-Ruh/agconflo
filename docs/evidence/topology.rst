===========================================
Evidence about storing a workflow as a file
===========================================

Measurements the choice of a storage format rests on, and the ones the reader
and writer are built on. All of them were taken on the same day in three
throwaway crates outside the repository, against the newest release of every
library involved, and each names the version it was taken against because a
format library's behaviour is exactly what moves between releases.

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

   The control, which is what makes that mean something: a document with three
   comments read through serde into a structure and written again with toml
   1.1.6 came back with none of them. A comment is not data serde carries, so a
   writer that regenerates a document cannot keep one.

   One edit of one value was measured here, and it was a value with no comment of
   its own. Adding and removing whole instances, and a value that has one, were
   measured afterwards (``EVD_TOML_EDIT_WHOLE_TABLES``,
   ``EVD_TOML_EDIT_ASSIGNING_LOSES_FORMAT``).

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

The measurements below were taken in a third crate, which read and edited
documents with toml_edit alone - no ``toml``, no serde - because that is how the
reader and the writer are built: one parse gives both the values read and the
document the writer edits. So the two properties first measured on ``toml`` that
the reader leans on, the order of a table's keys and where a fault is, were
measured again on toml_edit rather than assumed to carry over.

.. evd:: toml_edit keeps the order keys are written in
   :id: EVD_TOML_EDIT_KEEPS_ORDER
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: toml_edit 0.25.15 handed on the keys of a header table, an inline table and a dotted table in the order they were written, with no feature switched on.

   Parameters written as ``source``, ``beta``, ``alpha`` under a
   ``[types.t.required]`` header, as ``zebra``, ``apple``, ``mango`` in an inline
   table, and as ``yak``, ``bee`` in dotted keys came back in exactly those
   orders.

   toml_edit has no ``preserve_order`` switch to forget: it keeps a document's
   order because editing one in place needs it. So the trap recorded for ``toml``
   (``EVD_TOML_ORDER_NEEDS_FEATURE``) does not arise in a reader built on
   toml_edit - and a reader that sorts its parameters by some other route, a
   ``BTreeMap`` say, still loses the order exactly as that one did.

.. evd:: toml_edit locates by byte span and counts columns in characters
   :id: EVD_TOML_EDIT_SPANS
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: toml_edit 0.25.15 gave a byte span for every table and value it read and for every refusal, with a line and column only in the refusal's message, and that column counted in characters.

   Every kind of table a workflow can be written with had a span: a header
   table its header, a table made implicit by a later header its key in that
   header, a dotted table its key, an inline table its braces, an array of tables
   its first header, and the document's root ``0..0``. Every value had one too.

   A repeated instance table and a parameter bound twice were both refused with
   the message ``duplicate key`` and the span of the repeated key, as ``toml``
   refused them (``EVD_TOML_REFUSES_DUPLICATE_KEYS``). The first was span
   ``52..53``, displayed as ``line 4, column 12``.

   Read from toml_edit's source rather than measured: the line and column are
   computed only while formatting that message, in a private function. On a line
   reading ``name = "Stra`` followed by a sharp s and ``e"   bad = 1``, the
   refusal was at byte 19 and the message said column 19: a
   column counted in bytes says 20, because the sharp s is two bytes and one
   character. A reader handing the place on as values has to count as the message
   does, or the two disagree beside each other.

.. evd:: toml_edit adds and removes whole instances in place
   :id: EVD_TOML_EDIT_WHOLE_TABLES
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: toml_edit 0.25.15 removed an instance table with its sub-table and the comment above it, and wrote a new instance table after the last one, leaving every other line of the document as it was.

   Removing ``b`` from a document that wrote it as ``[instances.b]`` and
   ``[instances.b.bindings]`` took both headers, everything under them, and the
   comment on the line above ``[instances.b]``, which toml_edit holds as part of
   that table - as it does a comment at the end of the header line, which went
   with it in a second document. Removing an instance written as an inline table
   under ``[instances]`` took its line, the comment on the line above it and the
   comment at the end of it. Each result was compared with the input less exactly
   those lines, and was equal; adding an instance was compared the same way with
   the input plus its lines.

   A new instance table read back correctly whatever the document already held:
   instances as header tables, as inline tables under ``[instances]``, as dotted
   keys at the top, or no instances at all. A binding and an entry key added to
   an inline instance read back too, with a space left before the comma that
   followed the old last value.

   A table that loses its last binding stays behind empty, as
   ``[instances.b.bindings]`` or ``bindings = {}``, and reads as no bindings.

.. evd:: toml_edit writes a header table where its position puts it
   :id: EVD_TOML_EDIT_TABLES_KEEP_PLACE
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: toml_edit 0.25.15 left header tables where they were written when their keys were reordered, and wrote a header table added among inline instances after all of them.

   Two ways of moving an instance, and neither moves it. Reordering the keys of
   the instances table so that ``c`` came first changed nothing in the text. Why
   is read from toml_edit's source: each header table keeps the position it was
   read at, and headers are written in that order. Adding an instance as a header
   table to a document holding its instances as inline tables under
   ``[instances]`` wrote it after the last of them, whatever the key order.

   The second is TOML rather than toml_edit. Every key after a header belongs to
   that header's table, so a table's own keys are written before any of its
   sub-tables and an inline instance can never follow a header one under the same
   parent. Clearing the positions of the header tables did move them, comments
   and all - but only among header tables, which is no help for the second case.

   A table added new has no position yet, and is written after the table before
   it in key order. So an instance renamed in the middle of a document could be
   put back between its neighbours by reordering the keys: as a header table among
   header tables, and as an inline table among inline ones. It is the mixed
   document that has no such order.

.. evd:: Assigning a value loses its comment and its quoting
   :id: EVD_TOML_EDIT_ASSIGNING_LOSES_FORMAT
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: toml_edit 0.25.15 dropped the comment beside a value assigned a new one, and rewrote a literal string as a basic string when an equal value was assigned over it with its comment carried across.

   ``input = "a"  # repointed later`` became ``input = "c"`` when ``"c"`` was
   assigned to it, the comment gone. Taking the old value's decor - toml_edit's
   name for the whitespace and comment around a value - and putting it on the new
   one kept the document byte for byte apart from that value.

   Carrying the decor is not enough on its own. ``node_type = 'sink'`` assigned
   the same string, decor carried, came back as ``node_type = "sink"``: a new
   value is written with default quoting, whatever the old one used. So a writer
   that assigns every value it holds changes the text of a document it was asked
   to change nothing in.

.. evd:: toml_edit writes every line ending as LF
   :id: EVD_TOML_EDIT_WRITES_LF
   :evd_kind: measurement
   :observed_on: 2026-09-21
   :observation: toml_edit 0.25.15 read a document whose 24 lines ended in CRLF and wrote every one of them back ending in LF, with nothing else changed.

   The same document with LF line endings, and one with its instances written
   inline, each came back byte for byte. Only the CRLF one did not: every line
   was the same apart from the carriage return it had lost.

   A document written by an editor on Windows ends its lines in CRLF, so a writer
   handing on what toml_edit writes rewrites every line of it on its first save,
   and the diff shows the whole document changed.
