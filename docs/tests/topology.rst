===================
Topology test cases
===================

How each requirement in ``components/topology`` is to be checked. Results are
never written here: they are imported from the test runner, and each case records
only what it asserts.

Every failure mode listed in ``components/topology`` has a case that catches it,
and every "must" list has a case naming its shapes. As for the wiring feature, the
two pull in opposite directions on purpose, and here the positive half carries more
weight than it did there: a strict reader and a regenerating writer are both the
natural first implementation, each refuses or drops something legitimate, and no
case derived from the failure modes alone would notice.

Nine failure modes have no case named after them, because the shape that catches
each is part of a broader case under the same requirement. They are named here so
the derivation can be audited rather than taken on trust:

- ``CREQ_READER_WORKFLOW``'s "an absent entry key is read as an entry node" is
  caught by ``TEST_READER_READS_WHAT_IS_WRITTEN``, whose generator writes the
  entry key absent, present and true, and present and false.
- ``CREQ_READER_FAULT_LOCATED``'s "reading panics instead of refusing" is caught by
  ``TEST_READER_ANY_TEXT_IS_READ_OR_REFUSED``, since no table of inputs reaches the
  shape a panic hides in.
- ``CREQ_WRITER_WRITES``'s "a removed binding or output stays behind" and "a
  renamed instance is written twice" are both caught by
  ``TEST_WRITER_REMOVALS_LEAVE_NOTHING_BEHIND``, which is about one definition
  losing things, and would be split in three by filing each under its own failure.
- ``CREQ_READER_FAULT_LOCATED``'s "the column counts bytes" is caught by
  ``TEST_READER_FAULTS_CARRY_THEIR_PLACE``, one of whose documents puts a
  character wider than one byte before the fault on its line.
- ``CREQ_READER_FAULT_LOCATED``'s "the repetition is placed at the first of the
  two" is caught by ``TEST_READER_PARAMETER_DECLARED_TWICE_IS_A_FAULT``, which
  writes the two lists in both orders.
- ``CREQ_WRITER_KEEPS_UNREAD``'s "a changed value loses the comment beside it" is
  caught by ``TEST_WRITER_UNREAD_KEYS_SURVIVE_A_CHANGE``, whose repointed binding
  carries a comment of its own.
- ``CREQ_WRITER_KEEPS_UNREAD``'s "a value is written although it has not changed"
  and "line endings are rewritten" are both caught by
  ``TEST_WRITER_UNCHANGED_DOCUMENT_IS_BYTE_IDENTICAL``, which writes back a
  document quoting its strings as literals, once with each line ending.

A case's id is the path of the Rust test that implements it, uppercased, with
``::`` written as ``_``. The paths are ``<module>::<case>`` in ``agconflo-core``,
with the modules ``reader``, ``catalogue`` and ``writer`` - one per component, as
``wiring`` and ``defect`` are for the feature before this one - and the ids hold
only for bare test functions in those modules (``EVD_NEXTEST_TEST_PATHS``). The
correspondence runs both ways: every test in the crate is imported as a run naming
its case, so this document has exactly as many cases as those modules will have
tests, and a test with no case here is a dead link that fails the build.

Many cases below compare a document the test wrote with what the reader made of
it. The test writes that document with its own few lines of formatting rather than
with the writer, so that a reader and a writer wrong in matching ways cannot pass
together.

.. test_case:: A document reads as exactly what is written in it
   :id: TEST_READER_READS_WHAT_IS_WRITTEN
   :verifies: CREQ_READER_WORKFLOW
   :test_kind: property
   :coverage: partial

   For any workflow the test writes as a document - a name, instances each with a
   type and bindings, an entry key on some of them and an output or none - reading
   it with a catalogue gives a definition with exactly that name, those instances,
   those bindings, those entry nodes and that output, and with the catalogue's
   declarations as its node types.

   The generator must write the entry key absent, present and true, and present
   and false, since a reader defaulting an absent key to true reads every instance
   as an entry node and nothing else here would see it: the validator exempts an
   entry node's parameters, so the resulting definition passes validation.

.. test_case:: An absent output reads as no output
   :id: TEST_READER_ABSENT_OUTPUT_IS_NO_OUTPUT
   :verifies: CREQ_READER_WORKFLOW
   :test_kind: error_path
   :coverage: partial

   A document with no output key reads, and its definition designates nothing. The
   document has exactly one instance, because that is the shape where a reader
   filling the gap has an obvious candidate - and designating it would give the
   workflow an output its author never named.

   What holds afterwards is that the validator reports the signature defect, so
   the absence is refused where every other wiring defect is refused, and with
   them.

.. test_case:: Keys the model does not name are read past
   :id: TEST_READER_UNNAMED_KEYS_ARE_READ
   :verifies: CREQ_READER_WORKFLOW
   :test_kind: positive
   :coverage: partial

   A document carrying a table the model does not name at its top level, and keys
   it does not name on an instance, reads, and its definition is the one it would
   be without them.

   This is the first thing a strict reader refuses, and it is the document every
   editor produces.

.. test_case:: Empty documents read
   :id: TEST_READER_EMPTY_DOCUMENTS_READ
   :verifies: CREQ_READER_WORKFLOW
   :test_kind: positive
   :coverage: partial

   A workflow document with no instances reads as a definition with none, and one
   whose instances carry no bindings reads as a definition whose instances have
   none. Neither is refused: both are wiring questions, and the first is the one
   the validator refuses for its signature alone.

.. test_case:: Parameters keep the order they are written in
   :id: TEST_READER_PARAMETER_ORDER_IS_KEPT
   :verifies: CREQ_READER_TYPES
   :test_kind: property
   :coverage: partial

   For any list of distinct parameter names, written as a node type's required
   parameters in that order, the declaration read back lists them in that order -
   in an inline table and in a dotted table alike, since both were measured.

   The generator must reach orders that are not alphabetical, and does so almost
   always by chance; a reverse-sorted list is written out as well, because it is
   the shape in which every name moves. A sorting reader passes every list that
   happens to be sorted, and a reader built without the parser's
   order-preserving feature is a sorting reader (``EVD_TOML_ORDER_NEEDS_FEATURE``).

.. test_case:: The three declared lists are kept apart
   :id: TEST_READER_THREE_LISTS_KEPT_APART
   :verifies: CREQ_READER_TYPES
   :test_kind: positive
   :coverage: partial

   A type declaring required parameters, optional parameters and requested global
   types reads each into its own list, and a type declaring none of the three reads
   as a type with three empty lists and its output.

   The three are written in the document in an order other than required, optional,
   globals, so that a reader taking them by position rather than by key would put
   each in the wrong list.

.. test_case:: A fault carries its document, line and column
   :id: TEST_READER_FAULTS_CARRY_THEIR_PLACE
   :verifies: CREQ_READER_FAULT_LOCATED
   :test_kind: error_path
   :coverage: partial

   Each of four documents is refused: text that is not TOML, a value of the wrong
   type, an instance without its type, and a name repeated within one document.
   Each refusal is an error whose document name is the one the test gave, and whose
   line and column are the fault's, counted from one - read from the error's fields,
   never from its message.

   The exact values are the assertion. A place present only in the message passes
   every test that reads the message, and a place counted from zero is one off from
   the message beside it and passes every test that checks only that a line was
   given.

   One of the four puts a character wider than one byte before the fault on its
   line. The library's spans are in bytes and its message counts characters
   (``EVD_TOML_EDIT_SPANS``), so a column taken from the span alone is past the
   message's there, and right on every line written in ASCII.

.. test_case:: An empty context type name is a fault in the text
   :id: TEST_READER_EMPTY_CONTEXT_TYPE_IS_A_FAULT
   :verifies: CREQ_READER_FAULT_LOCATED
   :test_kind: error_path
   :coverage: partial

   A node type document declaring a parameter, and another declaring an output, of
   the context type ``""`` are each refused with the document, line and column of
   that value.

   The empty string is well-formed TOML, so the parser accepts it, and the model
   refuses it (``CREQ_VALUE_DECLARED_TYPE``). The case is the join between the two:
   the model's refusal has to come out as a fault in the text with a place, rather
   than as a panic or an error that has lost where it came from.

.. test_case:: A parameter declared in both lists is a fault in the text
   :id: TEST_READER_PARAMETER_DECLARED_TWICE_IS_A_FAULT
   :verifies: CREQ_READER_FAULT_LOCATED
   :test_kind: error_path
   :coverage: partial

   A node type document declaring one parameter as both required and optional is
   refused with the document, the line and column of whichever declaration is
   written later, and the key naming that list and parameter - once with the
   required list written first as inline tables, and once with the optional list
   written first as header tables. A document declaring ``input`` as required and
   ``Input`` as optional reads, as two parameters.

   The parser cannot refuse the shape, since the two lists are two tables and a
   name in both is no repeated key, so the reader has to. Both
   orders, because a reader always pointing at one list is right in exactly one
   of them. And the names differing in case are the control: comparing them
   loosely is the tidy-looking way to refuse this, and it refuses two parameters
   that are really different.

.. test_case:: Any text is read or refused
   :id: TEST_READER_ANY_TEXT_IS_READ_OR_REFUSED
   :verifies: CREQ_READER_FAULT_LOCATED
   :test_kind: property
   :coverage: partial

   For any text, and for any valid document with bytes deleted, inserted or
   swapped, reading returns either a definition or a refusal with a place, and
   never ends the run. Where the parser is what refused the text, the place is the
   line and column its own message gives, since a caller sees the two side by
   side.

   Arbitrary text alone would almost never parse far enough to reach a value the
   reader interprets, which is where an ``unwrap`` sits; mutating a valid document
   is what gets the generator there.

.. test_case:: A document carrying every wiring defect reads
   :id: TEST_READER_EVERY_WIRING_DEFECT_READS
   :verifies: CREQ_READER_NAMES_UNRESOLVED
   :test_kind: error_path
   :coverage: partial

   A document with an instance of a type no document declares, a binding to an
   instance that is not there, a binding to a parameter its type does not declare,
   a wire whose ends declare different context types, and no output reads, and
   the validator then reports all five. So does a second document, whose one
   output names no instance, and the validator reports that. Two documents,
   because no output and an output naming nothing cannot be written in one.

   The validator's half is the point. A reader that looks one name up reports
   that defect alone, and the author meets the others after fixing it; the
   assertion that every one reaches the validator together is what separates a
   reader that lets defects through from one that happens to let this document
   through.

.. test_case:: Reading resolves nothing
   :id: TEST_READER_READING_RESOLVES_NOTHING
   :verifies: CREQ_READER_NAMES_UNRESOLVED
   :test_kind: property
   :coverage: partial

   For any document whose instances name types drawn from a pool that the
   catalogue only partly declares, whose bindings name sources and parameters drawn
   from pools only partly present, and whose output names an instance that may not
   exist, reading succeeds.

   The pools include a binding to a parameter its type does not declare and an
   output naming no instance, since those are names too, and a reader resolving
   them would refuse a document for a defect the validator reports beside every
   other.

.. test_case:: A repeated type name names every document declaring it
   :id: TEST_CATALOGUE_REPEATED_NAME_NAMES_EVERY_DOCUMENT
   :verifies: CREQ_CATALOGUE_DECLARED_ONCE
   :test_kind: error_path
   :coverage: partial

   A type declared, differently, in three documents is refused with an error naming
   the type and all three documents.

   Three rather than two, because two is the number at which "the first and the
   last" and "every one" are the same answer. A catalogue keeping the last
   declaration, one keeping the first, and one naming only the document it was
   reading when it noticed all fail this, each differently.

.. test_case:: Identical copies of a declaration are refused
   :id: TEST_CATALOGUE_IDENTICAL_COPIES_REFUSED
   :verifies: CREQ_CATALOGUE_DECLARED_ONCE
   :test_kind: error_path
   :coverage: partial

   Two documents holding byte-identical declarations of one type are refused as
   any repetition is.

   The tempting catalogue compares the two declarations and keeps one when they
   agree, which is harmless on the day and is how two definitions of one type come
   to drift apart afterwards.

.. test_case:: Every repeated type name is reported
   :id: TEST_CATALOGUE_EVERY_REPEATED_NAME_REPORTED
   :verifies: CREQ_CATALOGUE_DECLARED_ONCE
   :test_kind: property
   :coverage: partial

   For any set of documents drawing type names from a small pool, the refusal names
   exactly the names declared more than once, each with exactly the documents
   declaring it, computed independently by the test; and a set with no repetition
   is not refused.

   The pool is kept small so that several names repeat at once, since a catalogue
   stopping at the first repetition passes every set with only one.

.. test_case:: Distinct declarations are all held
   :id: TEST_CATALOGUE_DISTINCT_DECLARATIONS_HELD
   :verifies: CREQ_CATALOGUE_DECLARED_ONCE
   :test_kind: positive
   :coverage: partial

   Two documents declaring different types, a document declaring several, and a
   type no workflow instance names are all held, each under its own name.

.. test_case:: Every change reads back
   :id: TEST_WRITER_EVERY_CHANGE_READS_BACK
   :verifies: CREQ_WRITER_WRITES
   :test_kind: property
   :coverage: partial

   For any document the test writes, and any sequence of changes to the definition
   read from it - an instance added, removed or renamed, a binding added, repointed
   or removed, the output changed or removed - writing the changed definition into
   the document and reading it back gives the changed name, instances, bindings
   and output - the instances and bindings compared under their names, since the
   document keeps its own order (``DEC_DOCUMENT_KEEPS_ITS_ORDER``).

   The generator must reach adding and removing whole instances, since those are
   the edits that write or delete a table rather than a value
   (``EVD_TOML_EDIT_WHOLE_TABLES``).

.. test_case:: Removals leave nothing behind
   :id: TEST_WRITER_REMOVALS_LEAVE_NOTHING_BEHIND
   :verifies: CREQ_WRITER_WRITES
   :test_kind: positive
   :coverage: partial

   A definition read from a document, then changed by removing an instance, removing
   a binding, removing the output and renaming an instance, is written, and the
   document read back has no trace of any of the four: no table for the removed
   instance, no binding for the removed parameter, no output key, and no table under
   the old name.

   Each is what an edit rewriting only what the definition still has leaves behind,
   and a property generating mostly additions and changes reaches them rarely.

.. test_case:: A document written back unchanged is the same text
   :id: TEST_WRITER_UNCHANGED_DOCUMENT_IS_BYTE_IDENTICAL
   :verifies: CREQ_WRITER_KEEPS_UNREAD
   :test_kind: positive
   :coverage: partial

   A document carrying comments, keys the model does not name, blank lines,
   uneven spacing and strings quoted as literals is read and written back with
   nothing changed, and the text is identical byte for byte - once with LF line
   endings and once with CRLF.

   A regenerating writer fails this on its first comment, and passes every test
   that compares what was read back rather than the text. So do two subtler ones
   (``EVD_TOML_EDIT_ASSIGNING_LOSES_FORMAT``, ``EVD_TOML_EDIT_WRITES_LF``): a
   writer that sets every value it holds, equal or not, turns ``'sink'`` into
   ``"sink"``, and one that hands on the library's text turns every CRLF into LF.

.. test_case:: A change keeps the keys and comments around it
   :id: TEST_WRITER_UNREAD_KEYS_SURVIVE_A_CHANGE
   :verifies: CREQ_WRITER_KEEPS_UNREAD
   :test_kind: property
   :coverage: partial

   For any document carrying comments and keys the model does not name, on the
   document and on its instances, repointing any one binding and writing the
   definition back changes that binding's value and nothing else in the text.

   The instance whose binding changes is always one carrying keys of its own, since
   a writer rewriting an instance's table whole keeps every other instance intact
   and loses exactly those - and a document where only other instances carry
   them would never show it.

   The binding that changes carries a comment after its value, too. Assigning the
   new value drops it (``EVD_TOML_EDIT_ASSIGNING_LOSES_FORMAT``), and it is the one
   comment a writer changing exactly the right value can still lose.

.. test_case:: Every unwritable shape is named and nothing is written
   :id: TEST_WRITER_UNWRITABLE_SHAPES_ALL_NAMED
   :verifies: CREQ_WRITER_UNWRITABLE_REFUSED
   :test_kind: error_path
   :coverage: partial

   A definition holding two instances under one name, a parameter bound twice and
   two designated outputs is refused, the refusal names all three shapes, and the
   document it was to be written into is afterwards the same text byte for byte.

   All three at once, because a writer naming the first it meets and stopping
   passes a case holding one. And the document is compared afterwards because a
   refusal that comes halfway through an edit leaves a document nobody asked for.

.. test_case:: Wiring defects are written
   :id: TEST_WRITER_WIRING_DEFECTS_ARE_WRITTEN
   :verifies: CREQ_WRITER_UNWRITABLE_REFUSED
   :test_kind: positive
   :coverage: partial

   A definition with an instance of an unknown type, a binding to an instance that
   is not there, a binding to a parameter its type does not declare, a wire whose
   ends disagree and no designated output is written, and reads back with the same
   name, instances, bindings and output. Then the same definition, designating an
   output that names no instance, is written and reads back the same way.

   The control on the case above. A writer that refuses every definition refuses
   every unwritable one, and a writer that runs the validator first refuses these -
   acting as a validator it is not, and making a half-finished workflow impossible
   to save.
