================================
Changes to existing requirements
================================

Every change to what an existing requirement obliges, one decision each
(``DEC_REQUIREMENT_CHANGES_RECORDED``), and every decision here is one: their
ids begin ``DEC_CHANGE_`` and no other decision's does. A requirement changes only when its parents
change or when it is found wrong against its own parents
(``DEC_REQUIREMENT_ANSWERS_TO_ITS_PARENTS``); the needs of other work are never
the reason.

Each record is a ``dec`` whose body names the requirement it changes, gives the
statement before and after, says what raised the change and which of the two
justifications applies, and carries the analysis against the requirement's own
parents with a verdict on every need the impact analysis found. It is
``supported_by`` an ``evd`` in ``evidence/changes`` holding that impact analysis
as run: the date, the command and the counts in each direction. The changed
requirement's body names its record.

``scripts/change-records.sh`` refuses a change to an existing requirement's
statement, pattern, verification method, stakeholder or placing links, or its
removal, unless the same range adds a line here naming it. The procedure is in
``AGENTS.md``, under "The V-Model".

The first five records are the revision the procedure began with: every change
made to an existing requirement before the procedure existed, from #22 to #27,
put through it afterwards (``EVD_AMENDMENTS_CAME_SIDEWAYS``). Three were right
against their own parents and are kept as they stand; two had let the feature
that changed them show through, and are revised here.

.. dec:: Kept: a held identifier is refused whatever held it
   :id: DEC_CHANGE_RUN_REFUSES_HELD_IDENTIFIER
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_IMPACT_RUN_REFUSES_HELD_IDENTIFIER
   :statement: Agconflo's requirements project shall keep the statement #22 gave CREQ_RUN_REFUSES_HELD_IDENTIFIER because it claims what its own parent claims and no more.

   Amends nothing now: records the change #22 made to
   ``CREQ_RUN_REFUSES_HELD_IDENTIFIER``, after the fact.

   Before: "If an output reported for an activation carries the identifier of
   an argument or of an output the run has accepted, then Workflow run shall
   refuse that output naming the instance and the identifier." After, and
   still: "If an output reported for an activation carries an identifier the
   run holds, then Workflow run shall refuse that output naming the instance
   and the identifier."

   Raised by the identity slice, under ``STKH_PROVENANCE``. Justification: it
   was wrong against its own parent. ``FEAT_RUN_OUTPUT_IS_NEW`` refuses an
   output "whose identifier the run already holds", and a run holds more than
   its arguments and accepted outputs - everything they were composed from. The
   old list let a part of an input be handed back as a node's own output,
   crediting it to a node that did not make it, while the parent held. The new
   wording is the parent's own word and names no mechanism.

   Verdicts on the impact analysis (``EVD_IMPACT_RUN_REFUSES_HELD_IDENTIFIER``):

   - Up, ``FEAT_RUN_OUTPUT_IS_NEW`` and ``STKH_PROVENANCE``: the justification
     above; neither changes.
   - Down, the two code markers and four test cases with their runs:
     unaffected now. #22 brought them along - the refusal walks what the run
     holds, and ``TEST_RUN_PASSED_THROUGH_ARGUMENT_IS_REFUSED`` covers a part
     handed back - and all four runs pass.
   - Sideways, the eleven other requirements of the workflow run: unaffected,
     since the statement does not change now.
   - Text, ten lines naming it in code comments and bodies: each cites it for
     the refusal of a held identifier, which is what it still says.

.. dec:: Kept: a feature of the same line placed on an existing architecture and requirement
   :id: DEC_CHANGE_ONE_CONTEXT_PER_IDENTIFIER_PLACED
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_IMPACT_ONE_CONTEXT_PER_IDENTIFIER_PLACED
   :statement: Agconflo's requirements project shall keep FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER among the features ARCH_RUN realises and the parents of CREQ_RUN_REFUSED_OUTPUT_OUTSTANDING.

   Amends nothing now: records the two links #22 added, after the fact.
   ``ARCH_RUN`` came to realise ``FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER``, and
   ``CREQ_RUN_REFUSED_OUTPUT_OUTSTANDING`` gained it as a third parent beside
   ``FEAT_RUN_OUTPUT_OF_DECLARED_TYPE`` and ``FEAT_RUN_OUTPUT_IS_NEW``. Neither
   statement changed.

   Raised by the identity slice, under ``STKH_PROVENANCE``. Justification: the
   parent set changed, which is the first of the two. A requirement given a new
   parent must suit it without changing, and both do. ``ARCH_RUN`` allocates
   "running a workflow" to the workflow run and its scheduler, and refusing a
   second context under a held identifier is part of running one.
   ``CREQ_RUN_REFUSED_OUTPUT_OUTSTANDING`` keeps an activation outstanding
   when "an output reported for an activation is refused" - by any refusal, so
   the new feature's is one more it covers without a word changing.

   Verdicts on the impact analysis
   (``EVD_IMPACT_ONE_CONTEXT_PER_IDENTIFIER_PLACED``):

   - Up: for ``ARCH_RUN``, the ten features it realises and the seven
     stakeholder requirements behind them; for the component requirement, its
     three parents and two stakeholder requirements. The justification above;
     none changes.
   - Down, the code marker and three test cases with their runs below the
     component requirement: unaffected, the statement being unchanged, and all
     three runs pass. An architecture has nothing below it by link.
   - Sideways, the fifteen requirements allocated to the components
     ``ARCH_RUN`` uses, and the eleven sharing the workflow run with the
     component requirement: unaffected; no statement moved.
   - Text, seven lines: they cite the architecture or the requirement for what
     each still says.

.. dec:: Revised: only the source makes an identifier outside the core
   :id: DEC_CHANGE_SOURCE_SOLE_ISSUER
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_IMPACT_SOURCE_SOLE_ISSUER
   :statement: Agconflo's requirements project shall restrict CREQ_SOURCE_SOLE_ISSUER to how code outside the core can make an identifier and leave restored identifiers to the run record.

   Amends ``CREQ_SOURCE_SOLE_ISSUER``.

   Before: "Identifier source shall be the only means of obtaining a context
   identifier, except by resuming a run's record together with a source
   positioned past every identifier the record holds." After: "Identifier
   source shall be the only means by which code outside the core can make a
   context identifier."

   Raised by the revision, after #25 had added the exception for resuming a
   run, under ``STKH_RESUMABLE_RUN``. Justification: it is wrong against its own
   parent, twice. ``FEAT_CONTEXT_IDENTITY`` asks that no two contexts of a run
   share an identifier, and needs two things of this component: that it never
   repeats (``CREQ_SOURCE_NO_REPEAT``), and that nothing else can make an
   identifier to repeat one. "The only means of obtaining" claimed more than
   the second - code obtains identifiers from contexts constantly, which its
   own test case has to allow - and the exception then named another feature's
   mechanism in the statement, which is the sign of a change made sideways.

   Three wordings were weighed. Keeping the exception keeps the resume feature
   in a requirement that does not answer to it. Dropping the exception without
   narrowing makes the statement false: the core does make identifiers again,
   when it resumes a record. Drawing the line at code outside the core names
   exactly the boundary a caller can cross, which is what the compile-time test
   checks, and leaves what the core does inside to the requirement that answers
   for it - ``CREQ_RECORD_SOURCE_CONTINUES``, under
   ``FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER``, the same stakeholder's.

   Verdicts on the impact analysis (``EVD_IMPACT_SOURCE_SOLE_ISSUER``):

   - Up, ``FEAT_CONTEXT_IDENTITY`` and ``STKH_PROVENANCE``: the justification
     above; neither changes.
   - Down, ``IMPL_ID_CONTEXT_ID``: unaffected; the identifier still has no
     constructor outside the core.
   - Down, ``IMPL_RECORD_SOURCE``: changes. The record's check that a
     restored identifier sits below its source's position no longer meets this
     requirement, so its marker names ``CREQ_RECORD_SOURCE_CONTINUES`` alone.
   - Down, ``TEST_ID_CANNOT_BE_FORGED``: changes, back to full coverage, since
     the exception it could not cover is gone.
   - Down, ``TEST_RECORD_IDENTIFIERS_ONLY_WITH_A_SOURCE``: changes; it now
     verifies ``CREQ_RECORD_SOURCE_CONTINUES``, whose body takes over the
     failure mode it catches. Its test is unchanged, so its run is imported
     under the same case.
   - Down, both runs: re-run at the head of this change; both pass.
   - Sideways, ``CREQ_SOURCE_NO_REPEAT``: unaffected. Its body already says an
     identifier made without the source is what this requirement exists for.
   - Text: the doc comment on the crate-private constructor in ``id.rs`` now
     cites ``CREQ_RECORD_SOURCE_CONTINUES``, which is what that code meets.
     ``decisions/resume`` cites the requirement twice, for "only the core can
     make a context identifier" and for the question the resume feature had to
     answer, and both remain true of the revised statement. The two lines in
     ``evidence/requirements`` are the measurement of the change itself.

.. dec:: Kept: running a script is how a script's activations are performed, not every activation
   :id: DEC_CHANGE_BEHAVIOUR_FROM_SCRIPT
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_IMPACT_BEHAVIOUR_FROM_SCRIPT
   :statement: Agconflo's requirements project shall keep the statements #27 gave FEAT_BEHAVIOUR_FROM_SCRIPT and CREQ_HOST_RUNS_THE_SCRIPT because they close nothing their parents do not.

   Amends nothing now: records the change #27 made to
   ``FEAT_BEHAVIOUR_FROM_SCRIPT`` and ``CREQ_HOST_RUNS_THE_SCRIPT``, after the
   fact.

   Before: "Agconflo shall perform each activation by running the script its
   caller supplied for the node type when starting the run", and "Script host
   shall perform an activation by running its node type's script given the
   activation's inputs by parameter name, its declared output type and the host
   functions." After, and still: "Agconflo shall perform each activation of a
   node type its caller supplied a script for by running that script", and
   "Script host shall perform an activation of a node type given a script by
   running that script given the activation's inputs by parameter name, its
   declared output type and the host functions."

   Raised by #27, for a person taking part in a run, under
   ``STKH_HUMAN_IN_RUN`` - sideways. Justification, found now against their own
   parents: they were wrong against them. ``STKH_LIVE_BEHAVIOUR`` asks that
   behaviour can change without recompiling the engine, and "each activation
   ... by running the script" claimed every activation is scripted, a closure
   it never made. The narrowed wording names no person and no other way of
   performing a step; it is what a reader of the parent alone would write, so
   the sideways trigger left no trace in it, and the change stands.

   Verdicts on the impact analysis (``EVD_IMPACT_BEHAVIOUR_FROM_SCRIPT``):

   - Up, ``STKH_LIVE_BEHAVIOUR``: the justification above.
   - Down, below the feature: ``CREQ_BEHAVIOURS_REFUSE_TWICE``,
     ``CREQ_HOST_RUNS_THE_SCRIPT`` (this record), ``ARCH_BEHAVIOUR``, three
     code markers, six test cases and their runs; below the component
     requirement, its marker and four of those test cases. Unaffected now; all
     runs pass.
   - Sideways, the twenty-one requirements of the behaviour set and the script
     host, and the six features ``ARCH_BEHAVIOUR`` also realises: unaffected;
     no statement moves now.
   - Text, two lines each. For the feature, the evidence of the change and
     ``ARCH_BEHAVIOUR``'s account of how it is split between the behaviour set
     and the script host, which still holds. For the component requirement, the
     evidence and the trace marker on the host's perform, which still meets it.

.. dec:: Revised: a run is refused for a node type with nothing to perform it
   :id: DEC_CHANGE_BEHAVIOUR_REFUSED_BEFORE_START
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_IMPACT_BEHAVIOUR_REFUSED_BEFORE_START
   :statement: Agconflo's requirements project shall state what refuses a run before it starts in terms of a supplied behaviour rather than of the scripts and people that supply one today.

   Amends ``FEAT_BEHAVIOUR_REFUSED_BEFORE_START`` and
   ``CREQ_BEHAVIOURS_REFUSE_MISSING``.

   Before, the feature: "If a node type named by an instance of a workflow has
   neither a script nor a person performing it or has a script that does not
   compile, then Agconflo shall refuse to start the run." After: "If a node type
   named by an instance of a workflow has no behaviour supplied for it or has a
   supplied behaviour found defective without running it, then Agconflo shall
   refuse to start the run."

   Before, the component requirement: "If a node type named by an instance of a
   workflow has no script and is not named as performed by a person, then
   Behaviour set shall refuse to start the run naming that node type." After:
   "If a node type named by an instance of a workflow has no behaviour supplied
   for it, then Behaviour set shall refuse to start the run naming that node
   type."

   Raised by the revision, after #27 had added the person to both, under
   ``STKH_HUMAN_IN_RUN``. Justification: both are wrong against their own
   parents. ``STKH_WIRING_CHECKED`` asks that a workflow known to be unrunnable
   is refused before any node runs. It says nothing about scripts or people;
   "no script" and then "neither a script nor a person" each listed the ways of
   supplying a behaviour that existed at the time as the only ones, and the
   second is the person feature showing through. The next way - a behaviour a
   caller writes in Rust, which ``STKH_TOOLS_AS_NODES`` will need - would have
   had to change both a third time.

   The revised feature names the property: a behaviour supplied, and not found
   defective before it runs. Not compiling stays the one defect a script can be
   found by without running it (``EVD_LUA_COMPILES_WITHOUT_RUNNING``), which
   ``CREQ_BEHAVIOURS_REFUSE_UNCOMPILABLE`` keeps. The component requirement
   takes the parent's term; its body says which two ways the behaviour set
   takes a behaviour today, and its must-pass cases cover both. Nothing the
   code does changes, which is the check that this is a correction of wording
   against the parent rather than a new obligation.

   Verdicts on the impact analysis
   (``EVD_IMPACT_BEHAVIOUR_REFUSED_BEFORE_START``):

   - Up, ``STKH_WIRING_CHECKED``: the justification above.
   - Down, ``CREQ_BEHAVIOURS_REFUSE_MISSING``: changes, with the feature, in
     this record.
   - Down, ``CREQ_BEHAVIOURS_REFUSE_UNCOMPILABLE``: unaffected; it is the
     script's case of a behaviour found defective without running it.
   - Down, ``CREQ_BEHAVIOURS_EVERY_FAULT`` and ``CREQ_BEHAVIOURS_NOTHING_RUN``:
     unaffected. Each speaks of scripts, and neither closes anything: the first
     promises every fault of the scripts in one refusal and does not say only
     scripts have faults, and the second is about checking without running, which
     only a script could be made to do. A later kind of behaviour adds its own
     requirement beside them.
   - Down, ``ARCH_BEHAVIOUR``: unaffected; the allocation is the same.
   - Down, the three code markers: unaffected; the code already refuses exactly
     a node type with neither a script nor a person, which is "no behaviour
     supplied" for this component.
   - Down, the eight test cases and their runs: unaffected in what they check.
     ``TEST_BEHAVIOURS_MISSING_SCRIPT_IS_REFUSED``,
     ``TEST_BEHAVIOURS_PERSON_NEEDS_NO_SCRIPT`` and
     ``TEST_BEHAVIOURS_UNINSTANTIATED_TYPE_NEEDS_NO_SCRIPT`` verify the revised
     component requirement from its two supplied sides and its unnamed one; all
     eight runs pass at the head of this change.
   - Sideways, ``CREQ_BEHAVIOURS_PERSON_OR_SCRIPT`` and
     ``CREQ_BEHAVIOURS_REFUSE_TWICE``: unaffected; two behaviours supplied for
     one type is their fault, under their own parents.
   - Sideways, the six features ``ARCH_BEHAVIOUR`` also realises: unaffected.
   - Text: the introduction of ``tests/person`` called the component
     requirement "narrowed", which it no longer is, and now says what its case
     checks. The doc comments in ``behaviours.rs`` and ``scripted.rs`` describe
     the code, which does not change. ``evidence/person`` cites the refusal the
     person feature measured, which still happens. The lines in
     ``evidence/requirements`` are the measurement of the change itself.
