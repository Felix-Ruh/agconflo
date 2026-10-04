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
