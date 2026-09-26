=========================================
Test cases of a workflow repeating itself
=========================================

How each requirement in ``components/repetition`` is to be checked, with one
feature-level case where the claim is about a whole run. Results are never
written here: they are imported from the test runner.

A case's id is the path of the Rust test that implements it, uppercased: the
scheduler's in the module ``scheduler``, the run's in ``run`` and the reader's
in ``reader``, all in ``agconflo-core``, and the whole run's in ``scripted``
in ``agconflo-lua``. Every failure mode listed in ``components/repetition`` is
named by the case that catches it.

.. test_case:: Each edge gives its earliest untaken context
   :id: TEST_SCHEDULER_TAKES_EARLIEST
   :verifies: CREQ_SCHEDULER_TAKES_EARLIEST
   :test_kind: positive
   :coverage: full

   An instance reading two edges, one holding three contexts it has not taken
   and the other one: its activation is given the first of the three and the
   one. Given another context on the second edge, its next activation is given
   the second of the three and that one, and no context twice.

   Catches: the latest context taken; one pass's contexts taken from edges
   moving at different rates; a context taken twice.

.. test_case:: An instance runs again only when something new has arrived
   :id: TEST_SCHEDULER_OFFERED_AGAIN_ON_SOMETHING_NEW
   :verifies: CREQ_SCHEDULER_OFFERS_AGAIN
   :test_kind: error_path
   :coverage: full

   An instance that has run, reading an edge from a standing output and an
   edge from a node that has produced again: it is offered. The same instance
   with the second edge holding nothing new: it is not, however many times it
   is asked. An instance reading nothing, and one reading only standing
   outputs, are offered once and never again.

   Catches: an instance offered once and never again; one offered on its
   standing inputs alone; one offered while an edge holds nothing new or
   standing.

.. test_case:: An output goes along every edge out of its instance
   :id: TEST_RUN_OUTPUT_WALKS_EVERY_EDGE
   :verifies: CREQ_RUN_WALKS_EVERY_EDGE
   :test_kind: positive
   :coverage: full

   An instance bound by two consumers, producing twice before either runs:
   each consumer is then given the first output, and on its next activation
   the second.

   Catches: the output replacing what an edge holds; one edge of several
   walked.

.. test_case:: A node type's standing declaration is read
   :id: TEST_READER_STANDING_READ
   :verifies: CREQ_READER_READS_STANDING
   :test_kind: positive
   :coverage: full

   Types declaring ``standing = true``, ``standing = false`` and neither read
   as standing, not standing and not standing; ``standing = "yes"`` is
   refused at its value.

   Catches: the key read past; ``standing = false`` read as standing.

.. test_case:: An output that stands serves every later activation
   :id: TEST_SCHEDULER_STANDING_SERVES_LATER
   :verifies: CREQ_SCHEDULER_STANDING_SERVES
   :test_kind: positive
   :coverage: full

   A consumer reading a declared-standing output and a changing one, activated
   three times: each is given the standing output, and once its producer
   produces again, the new one from then on. A consumer reading an entry
   instance's output, declared standing nowhere, is given it on every pass.

   Catches: a standing output taken once; the first of a standing node's
   outputs kept after a second; an entry instance's output taken once.

.. test_case:: A review loop runs until its router says it is done
   :id: TEST_SCRIPTED_REVIEW_LOOP
   :verifies: FEAT_REPEAT_ON_NEW_CONTEXTS
   :test_kind: positive
   :coverage: partial

   A workflow of an entry giving a brief, a drafter reading the brief and a
   router's input back, a reviewer, and a router sending the draft back to the
   drafter twice and then on to a finisher: the run completes with the third
   draft, the drafter having run three times, each time with the brief and the
   draft before it. Performed by scripts, with no model.

   Catches, as a whole: every requirement of the feature, and
   ``FEAT_STANDING_SERVES_LATER_PASSES`` and ``FEAT_ROUTE_WALKS_CHOSEN_EDGES``
   with it, where each unit case above could pass while their composition
   deadlocks.
