========================================
Evidence about pairing a pass's contexts
========================================

Measurements behind how a repeated node is given its contexts, taken on
2026-10-04 before anything was decided for it. The first two ran the engine as
it stood at ``00fc67b``, through a throwaway test driving a run by hand, each
activation producing a fresh context labelled with what it was given. The third
ran a model of the decision being weighed, in Python, over the wiring of the
first development workflow written for Agconflo, at ``52b4088``. Nothing of
any of them was kept but what is written here.

The fourth was taken live once that decision was built. The last two were taken
the same day at ``6d84dcf``, before deciding where a loop's first context comes
from: one ran the ``agconflo`` command over the fourth's workflow, the other
read every first context the repository and the first development workflow
give.

.. evd:: A join after a branch some passes skip is given contexts of two passes
   :id: EVD_PASSES_MISPAIRED_ACROSS_A_BRANCH
   :evd_kind: measurement
   :observed_on: 2026-10-04
   :observation: A node joining a node run on every pass with one a router sent the draft to on its second pass only was given the first pass's context beside the second pass's, with no fault or warning.

   The workflow: a drafter ``d`` reading a brief and what a router ``r`` sends
   back, the run giving it an empty first; ``x`` and ``r`` both reading the
   draft; ``y`` taking the draft through ``r``; and ``j`` joining ``x`` and
   ``y``. The router named ``d`` on its first pass, ``d`` and ``y`` on its
   second, and nothing on its third. As run::

     activate x pass 1: inputs [draft=D1] -> X1(draft=D1)
     activate r pass 1: inputs [draft=D1] -> R1(draft=D1)
        r routes to ["d"]
     activate x pass 2: inputs [draft=D2] -> X2(draft=D2)
     activate r pass 2: inputs [draft=D2] -> R2(draft=D2)
        r routes to ["d", "y"]
     activate y pass 1: inputs [draft=D2] -> Y1(draft=D2)
     activate j pass 1: inputs [x=X1(draft=D1), y=Y1(draft=D2)]

   Each edge into ``j`` held its contexts in the order walked, and ``j`` took
   the first of each: one walked on every pass, the other on one pass of
   three, so the first of each came from different passes. The run then ended
   quiescent, as the workflow's designated instance never ran.

.. evd:: A standing output that is made again reaches its readers by the order of the instances
   :id: EVD_STANDING_BY_INSTANCE_ORDER
   :evd_kind: measurement
   :observed_on: 2026-10-04
   :observation: A standing summary made again on every pass of a review loop reached a node reading it beside the draft as each pass's own when written before that node, and as the first pass's on every pass when written after it.

   The workflow: the same drafter and router, the router sending the draft
   back twice; ``s``, of a node type declared ``standing = true``, summarising
   each draft; and ``j`` reading the draft and the summary. Run twice, the
   definitions differing in nothing but the order ``s`` and ``j`` are
   written in::

     s before j: b[] d[B1,arg] r[D1] d[B1,R1] r[D2] d[B1,R2] r[D3]
                 s[D1] s[D2] s[D3] j[D1,S(D1)] j[D2,S(D2)] j[D3,S(D3)]
     j before s: b[] d[B1,arg] r[D1] d[B1,R1] r[D2] d[B1,R2] r[D3]
                 s[D1] j[D1,S(D1)] j[D2,S(D1)] j[D3,S(D1)] s[D2] s[D3]

   Each activation is written as its instance and what it was given. A
   standing output serves the latest context its node has made, and which is
   latest when ``j`` runs is decided by which of the two the scheduler offers
   first, which is the first in the definition's order: written first, ``j``
   ran all three passes before ``s`` had summarised the second draft. Two documents
   meaning the same workflow give two answers, as ``EVD_RUN_OPTIONAL_BY_ORDER``
   found of optional parameters.

.. evd:: The first development workflow pairs pass by pass once its router's branches are known
   :id: EVD_RECORD_GOALS_NEEDS_BRANCHES
   :evd_kind: prototype
   :observed_on: 2026-10-04
   :observation: Over record-goals' 55 instances, a model of pairing by clocks accepted the wiring with its router's two branches as its script names them, and refused it with each instance the router names taken as a branch of its own.

   ``workflows/record-goals`` at ``52b4088`` has one router, ``judge``, whose
   script names ``head`` and ``judge`` together to go round again and
   ``cmd_fin``, ``describe`` and ``approval`` together to finish, and the run
   gives ``head.state`` and ``judge.previous`` their first contexts. With
   those two branches the model put 29 instances on the loop's passes, 9 on
   the finishing branch's and 17 on the run's one pass, every node's inputs
   on one of them. With one branch per instance named, it refused ``judge``,
   whose inputs came on the passes of three branches no one of which holds
   the others. The same model accepted the review loop of
   ``TEST_SCRIPTED_REVIEW_LOOP``, put ``j`` of
   ``EVD_PASSES_MISPAIRED_ACROSS_A_BRANCH`` on the branch's passes with
   ``x`` read from the same pass, put ``s`` and ``j`` of
   ``EVD_STANDING_BY_INSTANCE_ORDER`` on the loop's passes with no order
   between them, and refused a node joining two branches of one router.

.. evd:: A review loop runs live, each pass given its own, and the last node the review of the last pass
   :id: EVD_REVIEW_LOOP_LIVE
   :evd_kind: measurement
   :observed_on: 2026-10-04
   :observation: Against qwen3.8-27b-ridge on LM Studio, a review loop with declared branches revised once and ended in 19 s, every activation given its own pass's contexts and the node after the loop the review of the pass it ended on.

   Taken after #72 merged, with the ``agconflo`` binary built at ``f878ace``
   for debugging, in a scratch directory, the model mapping sending the role
   ``drafting`` to ``openai::qwen3.8-27b-ridge`` at LM Studio's endpoint with
   its key in ``LM_API_TOKEN``, read from the repository's ``.env``, and every
   ``*_API_KEY`` variable removed from the process's environment.

   The workflow: a brief; a drafter asking the model for one sentence,
   reading the brief, the draft the router sends back and the router's
   verdict, the run giving it an empty first of each; a reviewer asking the
   model to approve only a sentence naming a sound, a colour and a smell; a
   router declaring ``branches = { again = ["drafter"], done = ["finisher"] }``
   and sending the draft on when the review approves it; a finisher; and a
   last node joining the finisher and the reviewer, on the passes of
   ``done`` reading the reviewer from every pass of the loop. Each activation
   in the record, written as what made each input::

     drafter#0  given brief#0, empty, empty
     reviewer#0 given drafter#0
     router#0   given drafter#0, reviewer#0   route ["drafter"]
     drafter#1  given brief#0, drafter#0, router#0
     reviewer#1 given drafter#1
     router#1   given drafter#1, reviewer#1   route ["finisher"]
     finisher#0 given drafter#1
     final#0    given finisher#0, reviewer#1

   The last node was given ``reviewer#1``, of the pass the loop ended on;
   counted per edge, it would have been ``reviewer#0``, the first its edge
   held. Two runs before it: with the reviewer approving freely, the first
   draft was approved and the run ended in one pass in 11 s; with the
   drafter not yet wired to the verdict, nothing it wrote was approved and
   the budget of 20 stopped the run at the drafter's seventh pass, after six
   of the router's, each drafter, reviewer and router given its own pass's
   contexts and every drafter the one brief.

   Given the first ``previous`` and not the first ``feedback``, the run was
   refused before any model was called, exit status 4, no record file
   written, the drafter's two inputs from the router coming on different
   passes. The message said only "the workflow carries 1 node whose inputs
   cannot be paired": every refusal a run starts with is printed as a count
   by ``agconflo run``, which names neither the node nor its inputs.

.. evd:: A check passes a loop that its runs leave stuck or refuse
   :id: EVD_LOOP_FIRST_CONTEXT_UNCHECKED
   :evd_kind: measurement
   :observed_on: 2026-10-04
   :observation: agconflo check reported nothing that would refuse the review loop, while a run given no first context ran one node and ended with no node able to go on, and a run given one of its two was refused.

   The workflow of ``EVD_REVIEW_LOOP_LIVE``, with the ``agconflo`` binary
   built at ``6d84dcf`` and its model mapping read as for that run. Its
   drafter takes ``previous`` and ``feedback`` from the router, and the run is
   what gave each its first context. Three commands, as printed::

     agconflo check  -> agconflo: nothing found that would refuse a run   (exit 0)
     agconflo run --arg brief input "a lighthouse"
                     -> agconflo: no node can make further progress; nothing
                        came from drafter, reviewer, router, finisher, final  (exit 7)
     agconflo run --arg brief input "a lighthouse" --arg drafter previous ""
                     -> agconflo: refused: the workflow carries 1 node whose
                        inputs cannot be paired: the inputs of 'drafter' ...  (exit 4)

   The second run's record holds one activation, ``brief``'s: a part of the
   workflow ran before the run found it could not go on. A check cannot know
   which parameters a run will give a first context to, so the loop's passes,
   and whether its nodes share one, are left to each run.

.. evd:: Every first context a loop is given is the same text on every run
   :id: EVD_FIRST_CONTEXTS_ARE_CONSTANT
   :evd_kind: measurement
   :observed_on: 2026-10-04
   :observation: Of the four first contexts the live review loop and the first development workflow give a loop, all four are a fixed text, two of them empty and two of four lines, and none varies from one run to the next.

   Read at ``6d84dcf`` and, for the development workflow, at ``52b4088`` on
   its open branch. The live review loop gives its drafter an empty
   ``previous`` and ``feedback``, and the repository's tests give a fixed
   placeholder text or an empty one. The development
   workflow keeps one file per first context, ``arguments/head.state.txt``
   holding ``goal 1``, ``pass 1``, ``stage no`` and ``feedback:``, and
   ``arguments/judge.previous.txt`` holding ``start``, ``notes:``, an empty
   line and ``feedback:``; its ``run.sh`` hands every file there to every run
   as an argument. What the run gives the loop is part of the workflow, kept
   beside it, and nothing about it is the caller's.
