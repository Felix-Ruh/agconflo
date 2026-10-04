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
   reading its own output, the run giving it the first, offered on that and
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
   two branches on the branch naming both, and a cycle no router is on given
   its first context each put on the passes expected; the same loop given
   nothing for it, none reported and its loop on no pass; and a cycle of two
   instances no router is on, given its first context, read by a third on
   its passes. A join of two branches no route takes together, a loop's own
   node reading a branch of it, and a join of two separate cycles no router
   is on: each reported, naming the instance and the passes each of its
   inputs comes on.

   Catches: two branches no route takes together passed; a loop's node
   reading a branch of the loop passed; a join of two branches a route takes
   together reported; a loop given nothing reported.

.. test_case:: A run whose node cannot be given one pass's contexts is refused
   :id: TEST_RUN_UNPAIRED_RUN_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNPAIRED, FEAT_PASSES_PAIRED_BEFORE_RUN
   :test_kind: error_path
   :coverage: full

   The review loop with its join reading two branches no route takes
   together: the run is refused before it starts, naming the join and the
   passes of both its inputs, in its message as well. The same workflow given
   nothing for its loop, whose nodes then run on no pass, starts - the control.

   Catches: the run started; the first such instance named and the rest left;
   the instances counted in the message and not named.

.. test_case:: An output goes to every instance reading it, pass by pass
   :id: TEST_RUN_OUTPUT_WALKS_EVERY_EDGE
   :verifies: CREQ_RUN_WALKS_EVERY_EDGE
   :test_kind: positive
   :coverage: full

   An instance read by two others, given its first context by the run and
   then reading one made once, so that it runs twice before either reader
   does: each reader is then given its first output, and on its next
   activation the second.

   Catches: the output replacing an earlier pass's; the output held for one
   reader of several.

.. test_case:: A context given for a wired parameter comes before what the wire carries
   :id: TEST_RUN_ARGUMENT_FIRST_ON_ITS_EDGE
   :verifies: CREQ_RUN_ARGUMENT_FIRST_ON_ITS_EDGE
   :test_kind: positive
   :coverage: full

   An instance given a context for its wired parameter, whose binding's source
   comes before it in the definition and produces before it runs: its first
   activation is given the context the run was given, and its second the
   source's output. Two contexts for that parameter are refused naming it.

   Catches: the context refused as a second source; the context placed after
   what was walked first; the context given for every pass.

.. test_case:: A review loop runs until its router says it is done
   :id: TEST_SCRIPTED_REVIEW_LOOP
   :verifies: FEAT_REPEAT_ON_NEW_CONTEXTS
   :test_kind: positive
   :coverage: partial

   A workflow of an instance giving the brief the run gave it, a drafter reading
   the brief and a router's input back, a reviewer, and a router sending the
   draft back to the drafter twice and then on to a finisher, through its two
   declared branches. The run gives the drafter an empty draft for its first
   pass (``CREQ_RUN_ARGUMENT_FIRST_ON_ITS_EDGE``). The run completes with the
   third draft, the drafter having run three times, each time with the brief
   and the draft before it, and its record holds each route. Performed by
   scripts, with no model.

   Catches, as a whole: every requirement of the feature, and
   ``FEAT_ENCLOSING_PASS_SERVES`` and ``FEAT_ROUTE_WALKS_CHOSEN_EDGES`` with
   it, where each unit case above could pass while their composition
   deadlocks.
