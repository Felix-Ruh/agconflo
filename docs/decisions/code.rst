=============================
Decisions about the Rust code
=============================

Choices about how the Rust source is written that hold across its crates,
recorded here rather than in the comments that used to carry them
(``STKH_REASONS_IN_THE_GRAPH``). Each is followed, in the code, by a marker on
the items it shapes.

.. dec:: A failure type whose set is open is non-exhaustive
   :id: DEC_FAILURES_NON_EXHAUSTIVE
   :dec_status: accepted
   :decided_on: 2026-09-25
   :statement: Agconflo shall mark a public failure type non-exhaustive unless a decision fixes the set of its variants.

   Failures are values (``STKH_TYPED_FAILURE``), and a caller matches on them.
   Most sets are not finished: what can be found wrong with a script before it
   runs, what a script or a model call can do wrong, what a record or an
   output can get wrong, and the wiring shapes the component requirements
   still record as open. Marked non-exhaustive, a caller's match carries a
   case for what comes later, and adding a variant breaks nobody.

   A set a decision fixes is the exception, and marking it would contradict
   the decision: a run ends in exactly one of four ways
   (``DEC_RUN_ENDS_ONE_WAY``), so ``RunEnding`` is exhaustive, and a caller
   handling its four cases has handled everything.
