=========================================
Test cases of a workflow repeating itself
=========================================

How each requirement in ``components/repetition`` is to be checked, with
feature-level cases where the claim is about a whole run. Results are never
written here: they are imported from the test runner.

A case's id is the path of the Rust test that implements it, uppercased: the
scheduler's in the module ``scheduler`` and the run's in ``run``, both in
``agconflo-core``, and the whole run's in ``scripted`` in ``agconflo-lua``.
Every failure mode listed in ``components/repetition`` is named by the case
that catches it.

Most cases drive one workflow: a review loop whose router sends the draft
back, back and to a node beside it, or on to a finishing node and that same
node, with a node reading every draft, a node joining that one with the one
beside, a node joining the draft with a node summarising it, and a node
joining the finishing node with the one beside on the pass the loop ends. A driver labels each context with the pass of the
draft it was made from, so that a case can say which pass every input came
from.

.. test_case:: An instance is offered once a pass, on its inputs of that pass
   :id: TEST_SCHEDULER_OFFERED_ONCE_PER_PASS
   :verifies: CREQ_SCHEDULER_OFFERS_AGAIN
   :test_kind: positive
   :coverage: full

   An instance reading nothing, offered once and never again; an instance
   reading its own output, declaring its first context, offered on that and
   then on each output it made; and an instance reading both, offered on each
   of the second's passes with the first's one context, and not offered
   between them however often it is asked.

   Catches: an instance offered once and never again; one offered again on
   inputs of a pass it has run; one offered before every input holds its
   next pass.

.. test_case:: A branch's node reads the pass the branch was taken on, and resumes so
   :id: TEST_RUN_BRANCH_READS_ITS_OWN_PASS
   :verifies: CREQ_SCHEDULER_READS_ENCLOSING_PASS, CREQ_SCHEDULER_GIVES_ONE_PASS, FEAT_ENCLOSING_PASS_SERVES
   :test_kind: positive
   :coverage: full

   The review loop, its router sending the draft back, then back and to the
   branch, then nowhere: the branch's node runs once, on the second pass, and
   the join beside it is given the second pass's draft from both its inputs,
   where the per-edge queues gave it the first pass's beside the second's
   (``EVD_PASSES_MISPAIRED_ACROSS_A_BRANCH``). The run's record taken between
   the branch's node and the join, resumed and driven on, does exactly what
   the run did.

   Catches: the earliest context not yet taken given; a branch's node given
   the enclosing pass's first or latest context; a context made once taken
   once.

.. test_case:: What each pass is given ignores the order instances are written in
   :id: TEST_RUN_PASSES_IGNORE_INSTANCE_ORDER
   :verifies: CREQ_SCHEDULER_GIVES_ONE_PASS, FEAT_REPEAT_ON_NEW_CONTEXTS
   :test_kind: property
   :coverage: full

   For any sequence of up to five router choices, each joining node of the
   review loop runs exactly as often as the passes its inputs share - one
   join once for each pass sending the draft beside, the last join once if
   the loop ended going on - and is given both its inputs from one pass on
   every activation; and the same workflow with its instances reversed and
   rotated gives every instance the same inputs pass for pass. The last join
   reads, on a branch's passes, a node on the passes of a larger set of the
   router's branches.

   Catches: the latest context given; a context given twice; and, as a whole,
   ``CREQ_RUN_WALKS_EVERY_EDGE`` and ``CREQ_RUN_WALKS_ROUTED``, which every
   choice exercises.

.. test_case:: Instances whose inputs share no pass are found, and no other
   :id: TEST_SCHEDULER_UNPAIRED_INPUTS_REPORTED
   :verifies: CREQ_SCHEDULER_REPORTS_UNPAIRED
   :test_kind: error_path
   :coverage: full

   A loop with three branches and nodes reading across them: none reported,
   a branch's node reading the loop's draft and a brief made once, a join of
   two branches on the branch naming both, and a cycle no router is on
   declaring its first context each put on the passes expected; the same loop
   declaring none, none reported and its loop on no pass; and a cycle of two
   instances no router is on, declaring its first context, read by a third on
   its passes. A join of two branches no route takes together, a loop's own
   node reading a branch of it, and a join of two separate cycles no router
   is on: each reported, naming the instance and the passes each of its
   inputs comes on.

   Catches: two branches no route takes together passed; a loop's node
   reading a branch of the loop passed; a join of two branches a route takes
   together reported; a cycle nothing starts reported as unpaired.

.. test_case:: A run whose node cannot be given one pass's contexts is refused
   :id: TEST_RUN_UNPAIRED_RUN_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNPAIRED, FEAT_PASSES_PAIRED_BEFORE_RUN
   :test_kind: error_path
   :coverage: full

   The review loop with its join reading two branches no route takes
   together: the run is refused before it starts, as for a wiring defect,
   naming the join and the passes of both its inputs, in its message as
   well. The loop with its join reading two branches a route takes together
   starts - the control.

   Catches: the run started; the first such instance named and the rest left;
   the instances counted in the message and not named.

.. test_case:: A cycle no first context starts is a defect, reported once
   :id: TEST_WIRING_CYCLE_UNSTARTED_REPORTED
   :verifies: CREQ_VALIDATOR_CYCLE_STARTED
   :test_kind: error_path
   :coverage: full

   A review loop whose drafter reads the router's input back on two edges:
   with both edges back declaring a first context it is sound, and with
   neither, or with only one of the two, the drafter and the router are
   reported once as a cycle. A cycle of two instances no router is on and an
   instance reading its own output are each reported once, in the
   definition's order, and an instance reading from one of them not at all;
   each names its first instance and no parameter, and says what it is in
   its message. The same definition with a broken binding reports that
   binding and nothing about its cycles.

   Catches: the cycle passed; a cycle passed because one binding on it
   declares a first context; each instance on it reported; a cycle a first
   context starts reported.

.. test_case:: An instance whose inputs share no pass is a defect
   :id: TEST_WIRING_UNPAIRED_REPORTED
   :verifies: CREQ_VALIDATOR_REPORTS_UNPAIRED
   :test_kind: error_path
   :coverage: full

   A loop whose node joins two branches no route takes together: the
   validator reports one defect, naming the join, saying it in its message.
   The control: the same join of two branches one route takes together
   reports nothing.

   Catches: the instance found only when a run starts.

.. test_case:: A run of a workflow with a cycle nothing starts is refused
   :id: TEST_RUN_CYCLE_UNSTARTED_REFUSED
   :verifies: CREQ_RUN_REFUSES_DEFECTS, FEAT_PASSES_PAIRED_BEFORE_RUN
   :test_kind: error_path
   :coverage: full

   Two instances bound to each other's output, with the designated output
   among them, beside an instance that could run: the run is refused before
   anything runs, naming the cycle, rather than ending quiescent once the
   other instance has run (``EVD_LOOP_FIRST_CONTEXT_UNCHECKED``). The control:
   one of the two bindings declaring its first context, the run completes.

   Catches, for ``CREQ_VALIDATOR_CYCLE_STARTED``: the cycle passed, part of a
   run done before it stops.

.. test_case:: An output goes to every instance reading it, pass by pass
   :id: TEST_RUN_OUTPUT_WALKS_EVERY_EDGE
   :verifies: CREQ_RUN_WALKS_EVERY_EDGE
   :test_kind: positive
   :coverage: full

   An instance read by two others, given its declared first context and
   then reading one made once, so that it runs twice before either reader
   does, and a join of the two readers coming last: each reader is then given
   its first output, and on its next
   activation the second.

   Catches: the output replacing an earlier pass's; the output held for one
   reader of several.

.. test_case:: A first context a binding declares comes before what the binding carries
   :id: TEST_RUN_FIRST_CONTEXT_DECLARED_COMES_FIRST
   :verifies: CREQ_SCHEDULER_GIVES_FIRST, CREQ_RUN_HOLDS_DECLARED_FIRST
   :test_kind: positive
   :coverage: full

   An instance whose binding declares the first context ``start``, the
   binding's source coming before it in the definition and producing before
   it runs: its first activation is given a context holding ``start`` of the
   parameter's type, which the run holds, and its second the source's output,
   and the instance reading it is given each in turn. The binding is on no
   cycle, so this is also a first context off one.

   Catches: the context placed after what was walked first; the context given
   for every pass; its type taken from elsewhere.

.. test_case:: A loop's state passed back by a router starts from the first context declared
   :id: TEST_RUN_LOOP_STATE_PASSED_BACK_BY_A_ROUTER
   :verifies: CREQ_SCHEDULER_GIVES_FIRST, FEAT_FIRST_CONTEXT_DECLARED
   :test_kind: positive
   :coverage: partial

   The first development workflow's shape, its state made by a node and not
   by the router: ``verdict`` reads a goal and the state the router ``judge``
   passes back as its input, declaring ``start`` as its first context;
   ``judge`` reads the goal and ``verdict``'s state, going round by naming
   ``verdict`` and on by naming the output's instance, which takes the goal
   from it. ``verdict``'s first pass is given ``start``, each pass after the
   very state of the pass before, and the output's instance the goal once the
   router names it; the run completes. The controls: declaring no first
   context, ``verdict`` and ``judge`` are reported as a cycle nothing starts;
   and the shape the case had before, a router reading its own output, is
   reported as taking an output a router does not make.

   Catches: the context placed after what was walked first, where what is
   walked is a router's input; the state passed back as another context than
   the one made.

.. test_case:: An argument for a parameter a binding fills is refused
   :id: TEST_RUN_ARGUMENT_FOR_BOUND_PARAMETER_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNFILLED_SIGNATURE
   :test_kind: error_path
   :coverage: full

   Arguments for a parameter whose binding declares a first context, and two
   for one whose binding declares none: each is refused as matching no
   parameter of the workflow, once for each argument, and nothing runs. The
   control: given nothing, the run starts.

   Catches: an argument for a parameter a binding fills taken as a second
   source.

.. test_case:: A first context's identifier is checked against the arguments'
   :id: TEST_RUN_FIRST_SHARES_NO_IDENTIFIER
   :verifies: CREQ_RUN_HOLDS_DECLARED_FIRST, CREQ_RUN_REFUSES_SHARED_ARGUMENT_IDENTIFIER
   :test_kind: error_path
   :coverage: full

   An argument made from one source and the run lent another that has issued
   nothing, so the first context is made under the argument's identifier: the
   run is refused naming that identifier. The control: lent the source the
   argument came from, it starts.

   Catches: the context made from a source of its own.

.. test_case:: A first context needing an exhausted source is refused
   :id: TEST_RUN_FIRST_FROM_EXHAUSTED_SOURCE_REFUSED
   :verifies: CREQ_RUN_HOLDS_DECLARED_FIRST
   :test_kind: error_path
   :coverage: full

   A workflow declaring a first context, started with a source that has
   issued every identifier: refused as such, with its message. The control:
   a workflow declaring none starts with the same source.

   Catches: an exhausted source passed over.

.. test_case:: A review loop runs until its router says it is done
   :id: TEST_SCRIPTED_REVIEW_LOOP
   :verifies: FEAT_REPEAT_ON_NEW_CONTEXTS
   :test_kind: positive
   :coverage: partial

   A workflow of an instance giving the brief the run gave it, a drafter reading
   the brief and a router's input back, a reviewer, and a router sending the
   draft back to the drafter twice and then on to a finisher, through its two
   declared branches. The drafter's binding declares an empty draft for its
   first pass (``CREQ_SCHEDULER_GIVES_FIRST``). The run completes with the
   third draft, the drafter having run three times, each time with the brief
   and the draft before it, and its record holds each route. Performed by
   scripts, with no model.

   Catches, as a whole: every requirement of the feature, and
   ``FEAT_ENCLOSING_PASS_SERVES`` and ``FEAT_ROUTE_WALKS_CHOSEN_EDGES`` with
   it, where each unit case above could pass while their composition
   deadlocks.

.. test_case:: A binding's first context is read as its text
   :id: TEST_READER_FIRST_READ
   :verifies: CREQ_READER_READS_FIRST
   :test_kind: positive
   :coverage: full

   A router's input declaring an empty first context and an output declaring
   one with a line break in it, written inline; one written as tables of
   their own, its text a multi-line string; and a binding declaring none.
   Each is read as written, and the one declaring none has none. A
   ``first`` that is a string, one holding no ``text``, one whose ``text`` is
   a number, and one holding a key beside ``text`` are each refused at the
   value or key at fault. ``reader::routed_input_read`` still refuses a
   binding table with neither ``input`` nor ``first``.

   Catches: the key read past; the text changed on the way; a malformed first
   context read as none; the input left optional for every binding table.

.. test_case:: A binding's first context is written back as declared
   :id: TEST_WRITER_FIRST_WRITTEN
   :verifies: CREQ_WRITER_WRITES
   :test_kind: positive
   :coverage: full

   A document whose router's input declares an empty first context, in
   single quotes, with a comment after it: written back unchanged, it is the
   same text. With that first context dropped and one with a line break
   declared on an output, each binding is written as the definition holds
   it, and the document reads back to the definition.

   Catches: a first context lost or added on the way out; a binding
   reformatted though nothing in it changed.
