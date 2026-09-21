=================
Wiring test cases
=================

How each requirement in ``components/wiring`` is to be checked. Results are never
written here: they are imported from the test runner, and each case records only
what it asserts.

Every failure mode listed in ``components/wiring`` has a case that catches it,
and every "must pass unreported" list has a positive case naming its shapes. The
two lists pull in opposite directions on purpose - one is the validator failing
to report a defect, the other is the validator reporting one that is not there -
and a set of cases derived from only one of them would leave the other free.

That matters more here than in the feature before this one. Four of the seven
requirements say what must be refused and none of those says what must not be, so
a validator refusing everything satisfies all four and no case built from them
would notice. The positive cases and ``TEST_WIRING_WELL_FORMED_DEFINITIONS_PASS``
are the half that does.

Three failure modes have no case of their own, because each is an interaction
between requirements and is asserted where that interaction happens. They are
named here so the derivation can be audited rather than taken on trust:

- ``CREQ_VALIDATOR_ONE_OUTPUT``'s "the check refuses the definition before the
  wiring is examined" and ``CREQ_VALIDATOR_TYPES_AGREE``'s "the check is skipped
  because something else about the wire was reported" are both what
  ``TEST_WIRING_ALL_FOUR_CLASSES_REPORTED`` is built to catch: its definition
  carries a signature defect and three wiring defects, two of them on one
  instance, so a walk that stops early anywhere reports three rather than four.
- ``CREQ_VALIDATOR_TYPES_AGREE``'s "a wire whose producer has no declaration is
  reported as a mismatch" is asserted by
  ``TEST_WIRING_INSTANCE_OF_MISSING_TYPE_IS_REPORTED``, which is about one
  definition and one report, and would be split in two by filing half of it
  under each requirement.

A case's id is the path of the Rust test that implements it, uppercased, with
``::`` written as ``_``. The paths here are ``<module>::<case>`` in
``agconflo-core``, with the modules ``wiring`` and ``defect`` - one per
component, as ``context``, ``id`` and ``lineage`` are for the feature before this
one. The ids hold as written only for tests that are bare test functions in those
modules, not inside a ``tests`` module (``EVD_NEXTEST_TEST_PATHS``).

That correspondence runs both ways, which is why this document has exactly as many
cases as those two modules will have tests. Every test in the crate is imported as
a ``test_run`` whose ``executes`` link names its case, so a test with no case here
is a dead link that fails the build rather than a result nobody noticed.

Three shapes are asserted on but not classified, and the distinction is the whole
reason ``TEST_WIRING_MALFORMED_DEFINITION_STILL_REPORTS`` reads the way it does: a
binding to a parameter no declaration carries, a designated output naming an
instance that is not there, and two instances sharing one name.
``CREQ_VALIDATOR_BINDING_RESOLVES`` records all three as not yet answered, so
which defect each earns is unsettled, and a case asserting one would be inventing
a requirement rather than checking one. What is settled is that none of them may
stop the walk, and that is what the case asserts. The third can no longer be read
from a document, where every name is a table key (``DEC_NAMES_AS_KEYS``), but the
model can still be given it, which is why the case keeps it.

.. test_case:: Every unbound required parameter is reported
   :id: TEST_WIRING_EVERY_UNBOUND_REQUIRED_IS_REPORTED
   :verifies: CREQ_VALIDATOR_REQUIRED_BOUND
   :test_kind: property
   :coverage: partial

   For any definition whose node types declare required parameters, optional
   parameters and requested global types, with an arbitrary subset of each bound,
   the unbound-parameter defects are exactly the required parameters carrying no
   binding, computed independently by the test.

   The generator must reach an instance with no bindings at all, since that is
   the case a walk over the bindings never visits, and it must declare optionals
   and globals on the same instances as required parameters, since a walk that
   reports those is over-blocking and no case built only from required parameters
   would see it.

.. test_case:: An instance with no bindings is reported once per parameter
   :id: TEST_WIRING_UNWIRED_INSTANCE_IS_REPORTED
   :verifies: CREQ_VALIDATOR_REQUIRED_BOUND
   :test_kind: error_path
   :coverage: partial

   An instance carrying no bindings at all, of a node type requiring two
   parameters, is reported as two defects rather than one: each names that
   instance and one of the parameters, and the two differ. What holds afterwards
   is that the walk continues - a second instance's own unbound parameter is in
   the same report.

   The granularity is the assertion. One defect saying the instance is unwired
   would satisfy "a defect is reported" and leave the author to work out which
   parameters it meant.

.. test_case:: Optionals, globals and empty declarations pass
   :id: TEST_WIRING_DEGENERATE_DECLARATIONS_PASS
   :verifies: CREQ_VALIDATOR_REQUIRED_BOUND
   :test_kind: positive
   :coverage: partial

   Nothing is reported for each shape the requirement names as legal: a node
   whose optional parameters are all unbound; a node type declaring no parameters
   at all; a node requesting a global context type that no binding carries; an
   entry node, whose parameters are the workflow's own rather than wires.

   A generator reaches a node type with an empty parameter list rarely, and never
   reaches the entry node at all, since what makes one is its position in the
   definition rather than anything about its declaration.

.. test_case:: A binding to an instance that is not there is reported
   :id: TEST_WIRING_BINDING_TO_MISSING_INSTANCE_IS_REPORTED
   :verifies: CREQ_VALIDATOR_BINDING_RESOLVES
   :test_kind: error_path
   :coverage: partial

   A binding naming an instance the definition does not carry is reported as one
   unresolved defect, naming the consuming instance and parameter and echoing the
   name that resolved to nothing.

   What must not also appear is the point of the case: that parameter is bound,
   so no unbound-parameter defect is reported for it. The two checks answer
   different questions about the same parameter, and a report carrying both would
   send the author to add a wire that is already there.

.. test_case:: Each wire to a missing name is reported
   :id: TEST_WIRING_EVERY_BROKEN_WIRE_IS_REPORTED
   :verifies: CREQ_VALIDATOR_BINDING_RESOLVES
   :test_kind: property
   :coverage: partial

   For any number of parameters bound to one instance name the definition does
   not carry, there is one defect per binding, each naming its own consumer.

   Collapsing them into a single defect about the name is what this rules out.
   Each binding is a separate wire to repoint, a defect is one thing wrong with
   one definition and carries one place, so a single defect could name only one
   of them and would leave every other broken wire unnamed.

.. test_case:: An instance of a type that was not supplied is reported
   :id: TEST_WIRING_INSTANCE_OF_MISSING_TYPE_IS_REPORTED
   :verifies: CREQ_VALIDATOR_BINDING_RESOLVES
   :test_kind: error_path
   :coverage: partial

   An instance whose node type the definition was not given is reported as one
   defect, naming that instance and echoing the type name.

   Nothing else about that instance is reported, and that is the half of the case
   that can regress quietly: its parameters are unknowable, so no
   unbound-parameter defect is invented for them, and no type-agreement defect is
   invented for the wires it feeds. The rest of the definition is still walked,
   so another instance's defect appears in the same report.

.. test_case:: A self-binding and shared node types pass
   :id: TEST_WIRING_SELF_BINDING_AND_SHARED_TYPES_PASS
   :verifies: CREQ_VALIDATOR_BINDING_RESOLVES
   :test_kind: positive
   :coverage: partial

   Nothing is reported for a binding from an instance to itself, which is a cycle
   of length one and legal (``DEC_BACK_EDGES_ALLOWED``), nor for two instances of
   the same node type, which share a declaration and nothing else.

   Both resolve, and both are what a resolver written as if the definition were a
   tree of distinct nodes refuses by accident.

.. test_case:: A wire across two context types is reported
   :id: TEST_WIRING_TYPE_DISAGREEMENT_IS_REPORTED
   :verifies: CREQ_VALIDATOR_TYPES_AGREE
   :test_kind: error_path
   :coverage: partial

   A binding joining an output declared of one context type to a parameter
   declared of another is reported as one defect, naming the consuming instance
   and parameter and carrying both type names.

   Both names are asserted on, because what a reader has to act on is which type
   was expected and which arrived; a defect saying only that the two differ sends
   them back to the declarations to find out.

.. test_case:: Context type names are compared exactly
   :id: TEST_WIRING_TYPE_NAMES_COMPARE_EXACTLY
   :verifies: CREQ_VALIDATOR_TYPES_AGREE
   :test_kind: property
   :coverage: partial

   For any two context type names that differ as strings, a binding joining them
   is reported; for any one name, a binding joining it to itself is not.

   The generator must produce pairs differing only in case, pairs differing only
   by surrounding whitespace, and pairs where one is a prefix of the other. Those
   are the three ways the comparison gets loosened, and a pair of random names is
   unlikely to be any of them, so a generator left to itself would prove only
   that unrelated names differ.

.. test_case:: Agreeing wires and fan-out pass
   :id: TEST_WIRING_AGREEING_WIRES_PASS
   :verifies: CREQ_VALIDATOR_TYPES_AGREE
   :test_kind: positive
   :coverage: partial

   Nothing is reported for a binding whose output and parameter declare the same
   context type on two different node types, nor for one output bound by several
   parameters, which the model allows (``DEC_BINDING_BY_PORT``).

   Fan-out is a structural shape rather than a comparison, which is why it is
   here rather than left to the property above: a check recording one consumer per
   output would refuse it while comparing every name correctly.

.. test_case:: A signature without exactly one output is reported
   :id: TEST_WIRING_SIGNATURE_WITHOUT_ONE_OUTPUT_IS_REPORTED
   :verifies: CREQ_VALIDATOR_ONE_OUTPUT
   :test_kind: error_path
   :coverage: partial

   A definition designating no output, and one designating two, each report
   exactly one signature defect naming the definition. The wiring of both is
   otherwise sound, so the signature defect is the whole report.

   Both are asserted, separately. A check that counts designations and compares
   against one catches them both; a check that only tests for absence passes the
   definition designating two, and a workflow whose result is whichever output a
   caller happens to bind is the failure that requirement exists to rule out.

.. test_case:: Legal signatures pass
   :id: TEST_WIRING_LEGAL_SIGNATURES_PASS
   :verifies: CREQ_VALIDATOR_ONE_OUTPUT
   :test_kind: positive
   :coverage: partial

   Nothing is reported for a workflow with no entry parameters, which is legal,
   nor for a designated output whose node also feeds other nodes, which does not
   make it less terminal.

   The second is what a check written around "terminal" rather than around
   "designated" refuses, and it is the ordinary shape of a loop
   (``DEC_BACK_EDGES_ALLOWED``), where the designated output feeds the node that
   starts the next pass.

.. test_case:: A definition with all four defect classes reports all of them
   :id: TEST_WIRING_ALL_FOUR_CLASSES_REPORTED
   :verifies: CREQ_VALIDATOR_EVERY_DEFECT
   :test_kind: error_path
   :coverage: partial

   A definition carrying one defect of each of the four classes at once - an
   unbound required parameter, a binding to a name that is not there, a wire
   across two context types, and no designated output - reports exactly four
   defects, one of each class, each once.

   Where the four sit is as much of the case as the count. The unbound parameter
   and the mismatched wire are on the same instance, so a walk that moves on once
   an instance has a defect reports three; and the signature is malformed, so a
   validator that refuses the definition for that before examining any wiring
   reports one. Both are failure modes their own requirements enumerate, and
   neither is visible in a definition whose defects are one per instance.

   The author learns about the rest only after a second submission in either
   case, which is the round trip ``FEAT_WIRING_ALL_DEFECTS`` exists to prevent.

.. test_case:: No defect is reported twice
   :id: TEST_WIRING_NO_DEFECT_IS_REPORTED_TWICE
   :verifies: CREQ_VALIDATOR_EVERY_DEFECT
   :test_kind: property
   :coverage: partial

   For any definition, defective or not, no two reported defects are equal, and
   no place is named twice by defects of the same class.

   The second half is what catches a duplicate carrying a different message for
   the same wire - reported once per direction of a binding, or once per check
   that touched it. Equality alone would let those through, and a report that
   grows without saying more is what an author paying per round trip reads.

.. test_case:: A malformed definition is reported rather than fatal
   :id: TEST_WIRING_MALFORMED_DEFINITION_STILL_REPORTS
   :verifies: CREQ_VALIDATOR_EVERY_DEFECT
   :test_kind: error_path
   :coverage: partial

   Each of these returns a report rather than ending the run: a binding naming a
   parameter that no declaration carries; a designated output naming an instance
   that is not there; two instances sharing one name; a binding naming an empty
   instance name; a definition declaring node types it has no instances of.

   Not ending the run is the whole assertion, and it is deliberately the whole of
   it. An index out of range, or an unwrap on a declaration that is not there,
   stops the walk at the first defect wearing different clothes, and a process
   that aborts reports nothing at all - so ``CREQ_VALIDATOR_EVERY_DEFECT`` is
   what these shapes are held to. Which defect, if any, three of them earn is
   recorded as not yet answered under ``CREQ_VALIDATOR_BINDING_RESOLVES``, and
   asserting a class here would pin behaviour no requirement asks for and make
   the next slice's answer a regression.

   The report is read rather than discarded, so the test fails if the call
   returns something unreadable, but nothing is asserted about its contents. That
   is a weaker case than the others in this document, and knowingly so: the
   alternative is inventing the classification it is missing.

.. test_case:: A well-formed definition is reported as nothing
   :id: TEST_WIRING_WELL_FORMED_DEFINITIONS_PASS
   :verifies: CREQ_VALIDATOR_ACCEPTS_WELL_FORMED
   :test_kind: property
   :coverage: partial

   For any definition generated well formed by construction - every required
   parameter bound, every name resolving, every wire agreeing on its context
   type, exactly one designated output - the report is empty.

   The generator carries this case, and it must reach the shapes a validator
   written to be strict refuses by accident: a cycle, a node no entry node
   reaches, an unbound optional parameter, a declared global that no binding
   carries, and an output bound by several parameters. A generator producing only
   trees would pass against a validator that refuses every one of them, which
   would make this the weakest case in the document rather than the control the
   other sixteen are measured against.

.. test_case:: An empty definition is refused for its signature alone
   :id: TEST_WIRING_EMPTY_DEFINITION_IS_REFUSED_FOR_ITS_SIGNATURE
   :verifies: CREQ_VALIDATOR_ACCEPTS_WELL_FORMED
   :test_kind: error_path
   :coverage: partial

   A definition with no instances and no node types reports exactly one defect,
   the signature defect, and no wiring defect of any class.

   It is the case where two requirements could quietly both fire, since an empty
   definition has nothing to wire and nothing to designate. Asserting the count
   and the class is what separates "refused for the right reason" from "refused",
   and only one of those two readings survives adding an instance to it.

.. test_case:: A binding defect names its instance and parameter
   :id: TEST_DEFECT_NAMES_INSTANCE_AND_PARAMETER
   :verifies: CREQ_DEFECT_NAMES_PLACE
   :test_kind: property
   :coverage: partial

   For any definition, every reported defect that concerns a wire - a required
   parameter carrying no binding, a binding that resolves to nothing, and a
   binding whose two ends declare different context types - carries the
   consuming instance and the parameter as values read from the defect itself,
   and each names something the definition carries.

   The three are named here rather than counted off the requirements, because
   ``CREQ_VALIDATOR_BINDING_RESOLVES`` covers two shapes and only one of them is
   a wire. An instance of a node type that was not supplied concerns that
   instance and no parameter at all: its declaration is what is missing, so its
   parameter list is unknowable, and a defect per parameter would contradict the
   single defect ``TEST_WIRING_INSTANCE_OF_MISSING_TYPE_IS_REPORTED`` asks for.
   Filling the field anyway is the failure mode ``CREQ_DEFECT_NAMES_PLACE``
   forbids in its other form, where a signature defect invents a parameter and
   sends the author to a node that is not wrong. That case is where an instance
   defect's place is checked.

   Read from the defect rather than from its rendering, deliberately. A place
   that can be recovered only by parsing a message is a place an agent correcting
   its own workflow cannot use, and the two are indistinguishable to a human
   reading the output - which is how this requirement would be lost without
   anyone noticing.

.. test_case:: A signature defect names the definition and no node
   :id: TEST_DEFECT_SIGNATURE_NAMES_THE_DEFINITION
   :verifies: CREQ_DEFECT_NAMES_PLACE
   :test_kind: error_path
   :coverage: partial

   The defect reported for a definition designating no output names the
   definition, and carries no node instance and no parameter.

   The absence is the assertion. A defect shaped so that every one of them has an
   instance forces a signature defect to name some node, and the author is then
   sent to a node that is not wrong.

.. test_case:: An unresolved name is echoed back
   :id: TEST_DEFECT_ECHOES_AN_UNRESOLVED_NAME
   :verifies: CREQ_DEFECT_NAMES_PLACE
   :test_kind: error_path
   :coverage: partial

   A defect reported for a binding to an instance that is not there carries that
   name as it was written, and so does one reported for an instance of a node
   type that was not supplied.

   The name resolves to nothing, which is exactly why it has to be echoed: it is
   the only thing tying the defect to what the author typed. A defect reporting
   the place without it says a wire is broken without saying what it was meant to
   point at.
