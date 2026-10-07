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

.. dec:: Revised: a model is sent exactly the contexts its call is made with
   :id: DEC_CHANGE_MODEL_WINDOW
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_IMPACT_MODEL_WINDOW
   :statement: Agconflo's requirements project shall state what a model call sends as exactly the contexts the call is made with, and leave how many there are and how their text is divided into messages to DEC_PROMPT_IS_A_CONTEXT.

   Amends ``FEAT_MODEL_WINDOW_IS_THE_PROMPT``, and ``CREQ_ROSTER_ONE_MESSAGE``,
   which becomes ``CREQ_ROSTER_CONTEXTS_WHOLE``.

   Before, the feature: "Agconflo shall send a model the rendering of the
   prompt context a script passes it as the call's only content." After:
   "Agconflo shall send a model exactly the contexts a call to it is made
   with."

   Before, the component requirement: "Model roster shall send a call's prompt
   as one user message holding the prompt's rendering and no other content."
   After: "Model roster shall send a call no text but the renderings of the
   contexts the call is made with, whole and byte for byte."

   Raised while planning ``STKH_MODEL_YIELDS``, whose calls send a model more
   than one message. That is the trigger and not the argument, which follows
   from the feature's parent alone.

   Justification: the feature is wrong against its own parent, because it
   claims more than the parent needs. ``STKH_EXPLICIT_CONTEXT`` gives a node
   exactly the contexts wired to it, so that "what was in this call's context
   window, and where did every byte come from?" has an exact answer. Of a model
   call it needs the window to be those contexts, whole, with nothing added.
   The statement said three things more: that a call is made with one context,
   that a script passes it, and that it goes as one rendering. The parent can
   hold while each of them is false. A call made with two contexts sent one
   after the other, every byte of both from contexts the node was given,
   answers the parent's question as exactly as one does. Those three are
   ``DEC_PROMPT_IS_A_CONTEXT``'s choices, and a choice of mechanism belongs in
   a decision, where evidence can move it, rather than in a statement that is
   not meant to (``AGENTS.md``, "The V-Model").

   The revised feature moves the parent's own sentence to the call. Its
   "exactly" is the parent's closure, and it names no script, no count and no
   message. That is the test the procedure sets a correction: it is what a
   reader of the parent alone would write, and it would read the same had no
   yield been proposed.

   The component requirement follows its parent, the first justification, and
   was wrong on the same ground besides, since "one user message" is the
   mechanism on the wire. The revised statement keeps what its failure modes
   guarded - no text added, none altered - and says "whole", which the parent's
   "exactly" asks for and the old statement left to its body. It drops one
   failure mode, a prompt's parts sent as separate messages. That was a defect
   only against the old statement: against the parent, how the text is divided
   is a choice, and the decision makes it, as one user message today. It is
   renamed because an identifier saying one message would assert what its
   statement no longer does.

   Nothing the code does changes: it sends one user message holding the
   prompt's rendering, which meets the revised component requirement and is
   what the decision says.

   Verdicts on the impact analysis (``EVD_IMPACT_MODEL_WINDOW``):

   - Up, ``STKH_EXPLICIT_CONTEXT``: the justification above; it does not
     change.
   - Down, ``CREQ_ROSTER_ONE_MESSAGE``: changes, in this record.
   - Down, ``CREQ_HOST_PROMPT_IS_A_CONTEXT``: unaffected. A prompt that is not a
     context is text no context holds, and the revised feature refuses it as
     the old one did. Its own analysis was run too and is in the evidence.
   - Down, ``ARCH_MODELS``: unaffected. The allocation is the same, and its body
     describes ``DEC_PROMPT_IS_A_CONTEXT``, which stands.
   - Down, ``IMPL_MODELS_CALL``: its marker names the renamed requirement. The
     code under it is unchanged.
   - Down, ``IMPL_HOST_COMPLETE``: unaffected.
   - Down, ``TEST_MODELS_PROMPT_SENT_EXACTLY``: verifies the renamed
     requirement. It still asserts one message, and its body now says that is
     the decision's division rather than the requirement's.
   - Down, ``TEST_HOST_PROMPT_MUST_BE_A_CONTEXT``: unaffected.
   - Down, both runs: re-run at the head of this change; both pass.
   - Sideways, the four other requirements of the model roster: unaffected.
     Which model a role reaches, an unmapped role, a provider's failure and the
     answer as sent concern where a call goes and what comes back, not what it
     sends.
   - Sideways, the sixteen requirements of the script host: unaffected; none
     says what a call sends.
   - Sideways, the four features ``ARCH_MODELS`` also realises: unaffected.
   - Text: none names the feature. The one line naming the component
     requirement is its trace marker in ``models.rs``, which names the new
     identifier.

.. dec:: Revised: a run stops when its activations reach its budget
   :id: DEC_CHANGE_RUN_STOPS_AT_BUDGET
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_IMPACT_RUN_STOPS_AT_BUDGET
   :statement: Agconflo's requirements project shall state the budget CREQ_RUN_STOPS_AT_BUDGET holds a run to as a number of activations, as its parent and its own body do.

   Amends ``CREQ_RUN_STOPS_AT_BUDGET``.

   Before: "If a run has activated as many instances as its budget allows, then
   Workflow run shall end that run without offering another activation."
   After: "If a run has made as many activations as its budget allows, then
   Workflow run shall end that run without offering another activation."

   Raised while planning ``STKH_MODEL_YIELDS``, where a node type a model calls
   is activated with no instance of its own. That is the trigger, not the
   argument.

   Justification: it cannot be verified as written against its own parent, and
   so it is wrong against it. ``FEAT_RUN_BUDGET_STOPS`` stops a run that "has
   activated as many nodes as its step budget allows", and
   ``DEC_BUDGET_COUNTS_ACTIVATIONS``, which the requirement's body cites, makes
   the unit an activation because an activation is what costs money. "Activated
   as many instances" reads two ways: as the number of activations, or as the
   number of instances that have activated. The second is a failure mode the
   requirement's own body lists: "Counted per instance rather than per
   activation. Indistinguishable while no instance activates twice, and wrong
   the moment loops exist." A statement one of whose readings is its own named
   defect is ambiguous whatever else exists. It also names instances where the
   parent names nodes, which is the mechanism of its day rather than the
   property.

   "Made as many activations" is the reading the body already gave it and the
   unit of the decision it cites. It names no kind of node and nothing a yield
   brings, so it is what a reader of the parent alone would write.

   Verdicts on the impact analysis (``EVD_IMPACT_RUN_STOPS_AT_BUDGET``):

   - Up, ``FEAT_RUN_BUDGET_STOPS`` and ``STKH_STEP_BUDGET``: the justification
     above; neither changes.
   - Down, ``IMPL_RUN_STEP``: unaffected. The run counts one per activation it
     offers, which is the revised statement's count.
   - Down, ``TEST_RUN_ACTIVATIONS_NEVER_EXCEED_BUDGET``,
     ``TEST_RUN_BUDGET_STOPS_AT_THE_LIMIT``,
     ``TEST_RUN_LAST_PERMITTED_ACTIVATION_COMPLETES`` and
     ``TEST_RUN_ZERO_BUDGET_ACTIVATES_NOTHING``: unaffected. Each counts the
     activations offered, and all four runs pass at the head of this change.
   - Sideways, the eleven other requirements of the workflow run: unaffected.
     ``CREQ_RUN_REFUSED_OUTPUT_OUTSTANDING`` keeps a refused output's activation
     from being counted again, which already speaks of activations, and
     ``CREQ_RUN_REFUSES_UNDECLARED_OUTPUT`` is revised in its own record.
   - Text: the doc comment on the budget ending in ``run.rs`` repeated "activated
     as many instances" and now says the revised words. The trace marker on the
     run's step still meets it. The note in ``tests/run`` that the per-instance
     failure mode cannot be told apart while no instance activates twice is
     about the failure mode, which is unchanged, and still holds.

.. dec:: Revised: an output is checked against its activation's node type
   :id: DEC_CHANGE_RUN_REFUSES_UNDECLARED_OUTPUT
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_IMPACT_RUN_REFUSES_UNDECLARED_OUTPUT
   :statement: Agconflo's requirements project shall state the type CREQ_RUN_REFUSES_UNDECLARED_OUTPUT compares an output with as the one its activation's node type declares, as its parent does.

   Amends ``CREQ_RUN_REFUSES_UNDECLARED_OUTPUT``.

   Before: "If an output reported for an activation is not of the context type
   its instance's node type declares, then Workflow run shall refuse that
   output naming the instance, the declared type and the reported type." After:
   "If an output reported for an activation is not of the context type that
   activation's node type declares for its output, then Workflow run shall
   refuse that output naming the instance, the declared type and the reported
   type."

   Raised while planning ``STKH_MODEL_YIELDS``, like the budget's record above,
   and judged apart from it.

   Justification: it is wrong against its own parent by one hop.
   ``FEAT_RUN_OUTPUT_OF_DECLARED_TYPE`` refuses "an output whose context type
   differs from the output type its node type declares". The requirement
   reached that node type through "its instance's", which the parent does not:
   it named how an activation came by its node type in the run as it stood,
   rather than the node type. The revised statement takes the parent's term.
   It still names the instance, which is now the instance the activation is
   performed for, as it is in the run's other refusals; the body says so.

   This is the weaker of the two run records, and it says so. While every
   activation belongs to an instance, the two wordings pick out the same type,
   and no test can tell them apart. The case for changing it is the rule that a
   statement names the property and not the mechanism of its day (``AGENTS.md``,
   "The V-Model"), and here the property is the parent's own word. Nothing the
   code does changes: it compares with the type the activation carries, which
   is what the revised statement says.

   Verdicts on the impact analysis
   (``EVD_IMPACT_RUN_REFUSES_UNDECLARED_OUTPUT``):

   - Up, ``FEAT_RUN_OUTPUT_OF_DECLARED_TYPE`` and ``STKH_WIRING_CHECKED``: the
     justification above; neither changes.
   - Down, ``IMPL_RUN_PRODUCED``: unaffected. It compares the output with the
     type the outstanding activation carries, and names its instance.
   - Down, ``TEST_RUN_DESIGNATED_UNDECLARED_OUTPUT_IS_REFUSED``,
     ``TEST_RUN_OUTPUT_OF_DECLARED_TYPE_IS_ACCEPTED`` and
     ``TEST_RUN_UNDECLARED_OUTPUT_IS_REFUSED``: unaffected, and all three runs
     pass at the head of this change.
   - Sideways, the eleven other requirements of the workflow run: unaffected.
     ``CREQ_RUN_REFUSES_HELD_IDENTIFIER`` and
     ``CREQ_RUN_REFUSES_SHARED_OUTPUT_IDENTIFIER`` name "the instance" in their
     refusals with no antecedent in their statements: the instance the
     activation is performed for, the reading this record now gives its own.
     ``CREQ_RUN_ENDS_ON_FAILURE`` names "the instance it was reported for".
     ``CREQ_RUN_STOPS_AT_BUDGET`` is revised in its own record.
   - Text, six lines. The doc comment on the refusal in ``run.rs`` said "the
     instance's node type" and now says the activation's; so does the one on
     reporting an output, which does not name the requirement and so is not
     among the six, but repeated its words. The doc comment on the type an
     activation carries, in ``scheduler.rs``, says "the instance's node type"
     and is left: every activation the scheduler builds is an instance's. The
     trace marker on the run's output check still meets it. ``features/behaviour``
     cites it for refusing an output of the wrong type, and ``tests/run`` twice
     for failure modes its cases assert; all three still hold.

.. dec:: Revised: a step runs as a user the person can answer for, not as the process's own
   :id: DEC_CHANGE_SANDBOX_STEP_USER
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_IMPACT_SANDBOX_STEP_USER
   :statement: Agconflo's requirements project shall state CREQ_SANDBOX_STEP_USER as what its parent needs of the user a step runs as, rather than the user it is run as today.

   Amends ``CREQ_SANDBOX_STEP_USER``.

   Before: "When a step is run, Sandbox shall run it on Linux as the user and
   group the running process has and elsewhere as user and group 1000."
   After: "When a step is run, Sandbox shall run it as a user whose files the
   person running the run can change and who can change no file on the host
   that person could not."

   Raised by the review of #48, which found that for a person running as root
   the requirement asked for a step run as root, which
   ``DEC_CONTAINER_LOCKED_DOWN`` rules out: the cleanup after each step then
   kills the container's own process.

   Justification: it is wrong against its own parent. ``FEAT_TOOL_KEPT_TO_ITS_GRANT``
   keeps a tool from changing anything outside what the grants allow, and
   what it needs of the user a step runs as is what the requirement's own body
   gave as its reason: a file the step writes is the person's to change, and
   the step can change nothing the person could not. The statement named the
   user that delivers that today - the process's, and 1000 elsewhere - which
   is a decision's to name, and in naming it claimed, for a root person, a
   root step the parent does not need. The revised statement is the property
   and names no user; the decision below it now says which
   (``DEC_STEP_USER_NEVER_ROOT``). The new work of this pull request, an
   environment per tool, does not show in it: it would read the same had
   that work never been proposed.

   Verdicts on the impact analysis (``EVD_IMPACT_SANDBOX_STEP_USER``):

   - Up, ``FEAT_TOOL_KEPT_TO_ITS_GRANT`` and ``STKH_TOOLS_CONFINED``: the
     justification above; neither changes.
   - Down, ``IMPL_SANDBOX_STEP`` and ``IMPL_SANDBOX_STEP_USER``: the code
     meets the revised statement unchanged - it already ran a root person's
     steps as 1000 - and each marker now follows ``DEC_STEP_USER_NEVER_ROOT``
     in place of the decision it supersedes.
   - Down, ``TEST_SANDBOX_STEP_USER`` and ``TEST_SANDBOX_USER_FROM_STATUS``:
     unaffected in what they assert - the first that a step runs as the ids
     the decision picks and never as root, and on Linux that its file belongs
     to the test's user; the second how those ids are read - and both runs
     pass at the head of this change. The first's body says it checks the
     property through the decision's user.
   - Sideways, the nine other requirements of the sandbox: unaffected.
     ``CREQ_SANDBOX_LOCKED_DOWN`` and ``CREQ_SANDBOX_NOTHING_LEFT_RUNNING`` are
     the two the old wording collided with for a root person, and the
     revision is what removes the collision; neither changes.
   - Text, the two markers above.

.. dec:: Revised: a run is refused for an absent image, not for an engine out of reach
   :id: DEC_CHANGE_RUNNER_REFUSES_UNGRANTED
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_IMPACT_RUNNER_REFUSES_UNGRANTED
   :statement: Agconflo's requirements project shall state what CREQ_RUNNER_REFUSES_UNGRANTED refuses a run for about its image as the image absent or lacking sh or timeout, as its parent does, rather than not ready.

   Amends ``CREQ_RUNNER_REFUSES_UNGRANTED``.

   Before: "If a tool the manifest names has an action its grants do not allow
   or lacks a parameter its action takes or the sandbox finds the image not
   ready, then Runner shall refuse the run before any node runs naming each
   tool and why." After: "If a tool the manifest names has an action its
   grants do not allow or lacks a parameter its action takes or the image is
   absent or lacks sh or timeout, then Runner shall refuse the run before any
   node runs naming each tool and why."

   Raised by the review of #48: while the engine cannot be reached, a person
   cannot answer a tool step by hand, since every answer asks for the image.

   Justification: it is wrong against its own parent. ``FEAT_TOOL_UNGRANTED_REFUSED``
   refuses a run whose tool lacks its grant or a parameter "or the image its
   grants name is absent", each, its body says, "a run that cannot finish as
   written". "Not ready", as the requirement's body read it, also covered an
   engine that could not be reached, which is no fault of the run as written
   and says nothing of the image: refusing for it claimed more than the parent
   asks. The revision names what is wrong with the image itself, and drops
   "the sandbox finds", which named the component that finds it.

   It keeps one surplus, and says so: an image lacking ``sh`` or ``timeout``
   is not "absent" in the parent's words. It is kept because the parent's
   body covers it - such an image performs no step, so the run cannot finish
   as written - and dropping it would let a run start that must fail at its
   first tool.

   Verdicts on the impact analysis (``EVD_IMPACT_RUNNER_REFUSES_UNGRANTED``):

   - Up, ``FEAT_TOOL_UNGRANTED_REFUSED``, ``STKH_WIRING_CHECKED`` and
     ``STKH_TOOLS_CONFINED``: the justification above; none changes.
   - Down, ``IMPL_RUNNER_PREPARED``: changes. An engine that cannot be reached
     no longer refuses a start, a resume or an answer
     (``DEC_UNREACHABLE_ENGINE_REFUSES_NO_RUN``); the first tool step then
     comes to the engine's failure through the sandbox, as it does when the
     engine stops mid-run.
   - Down, ``TEST_RUNNER_REFUSES_UNGRANTED``: unaffected in what it asserts,
     which is an absent image refusing the run, and its run passes. A new case
     checks the narrowing: a start and an answer with the engine out of reach
     go on to the tool step and leave it awaiting.
   - Sideways, the twelve other requirements of the runner: unaffected.
     ``CREQ_RUNNER_CHECKS_TOOLS`` keeps reporting an engine out of reach, since
     a check reports what would stop a run as well as what refuses one.
     ``CREQ_RUNNER_ENGINE_FAILURE_STOPS`` now also covers the first tool step
     of a run started while the engine was out of reach, in its own words.
   - Text, the marker above, and the requirement's own body, whose sentence
     saying an unreachable engine is refused here is rewritten.

.. dec:: Revised: a tool container mounts nothing but what the grants name
   :id: DEC_CHANGE_SANDBOX_LOCKED_DOWN
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_IMPACT_SANDBOX_LOCKED_DOWN
   :statement: Agconflo's requirements project shall state what CREQ_SANDBOX_LOCKED_DOWN mounts as what the run's grants name, as its parent speaks of what the grants allow, rather than as the granted folders.

   Amends ``CREQ_SANDBOX_LOCKED_DOWN``.

   Before: "When a container is made for a run, Sandbox shall make it with an
   init process, no capabilities, no new privileges, a read-only root, only
   the granted folders mounted and no network unless granted." After: "When a
   container is made for a run, Sandbox shall make it with an init process,
   no capabilities, no new privileges, a read-only root, nothing mounted but
   what the run's grants name and no network unless granted."

   Raised by the maintainer's choice to mount a grants file's certificate
   authorities read-only rather than copy them, which the old wording ruled
   out.

   Justification: it is wrong against its own parent. ``FEAT_TOOL_KEPT_TO_ITS_GRANT``
   keeps a tool from changing anything outside what the run's grants allow,
   and ``STKH_TOOLS_CONFINED`` above it anything outside what the person
   granted. Both speak of the grants; the requirement spoke of the one thing
   a grants file could grant when it was written, folders, and so named that
   mechanism as the only one - the shape ``EVD_AMENDMENTS_CAME_SIDEWAYS``
   found behind five of the first six changes. Its parent could hold while it
   is false: a file the person names in the grants, mounted read-only,
   changes nothing outside the grant. The revision states the property in the
   parent's own terms and names nothing a grants file holds; it would read
   the same had the trust file never been proposed. It still claims more than
   its parent in one respect, kept on purpose: nothing ungranted is mounted at
   all, readable or not, where the parent speaks only of changing. Dropping
   that was the other option, a wider change nobody asked for that would let
   a host path the person never named be read.

   Verdicts on the impact analysis (``EVD_IMPACT_SANDBOX_LOCKED_DOWN``):

   - Up, ``FEAT_TOOL_KEPT_TO_ITS_GRANT`` and ``STKH_TOOLS_CONFINED``: the
     justification above; neither changes.
   - Down, ``IMPL_SANDBOX_MAKE``: meets the revised statement unchanged, since
     it mounts the granted folders; the trust file's mount is made there too
     and carries its own marker.
   - Down, ``TEST_SANDBOX_CONFINED_TO_GRANTS``: changing, adding what no test
     asserted before - that the container's mounts are the granted folders
     and nothing else. ``TEST_SANDBOX_OWN_PROCESS_SURVIVES`` and
     ``TEST_SANDBOX_ROOT_STEP_CLEANED_UP``: unaffected, being about the
     container's own process; all three runs pass at the head of this change.
   - Sideways, ``CREQ_SANDBOX_TRUSTS_GRANTED``: new in this change and written
     for the mount. The other fourteen requirements of the sandbox:
     unaffected, none being about what is mounted.
   - Outside the analysis, since it shares no component with this
     requirement: ``CREQ_GRANTS_READS_TRUST``, also new in this change, gives
     the trust file's path rather than its content, which a mount no longer
     needs, with any link in the path resolved so that the file mounted is the
     one the grants name.
   - Text, the marker in ``sandbox.rs``: unchanged. The line of
     ``components/environment`` in ``CREQ_SANDBOX_TRUSTS_GRANTED``'s body and
     the line of ``decisions/tools`` in the trust decision: rewritten for the
     mount. The other line of ``components/environment`` and the one of
     ``decisions/changes``: unaffected, neither being about mounts.

.. dec:: Removed: a parameter nothing binds is an input, not a wiring defect
   :id: DEC_CHANGE_WIRING_REQUIRED_BOUND
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_IMPACT_WIRING_REQUIRED_BOUND
   :statement: Agconflo's requirements project shall remove FEAT_WIRING_REQUIRED_BOUND and CREQ_VALIDATOR_REQUIRED_BOUND because the stakeholder decided a parameter nothing binds is one of a workflow's inputs.

   Removes ``FEAT_WIRING_REQUIRED_BOUND`` and ``CREQ_VALIDATOR_REQUIRED_BOUND``,
   and takes the first from what ``ARCH_WIRING`` realises.

   Before, the feature: "If a workflow leaves a required parameter of a node
   unbound, then Agconflo shall reject that workflow before running any node
   of it." And the component: "If a node instance leaves a required parameter
   unbound, then Wiring validator shall report a defect naming that
   parameter." After: neither exists.

   Raised by the maintainer, while the repetition feature was being planned:
   a loop's first pass needs a context from outside the loop, and every
   context a node is given should be a context, whichever node is given it,
   with no node marked as where a run begins.

   Justification: its parent changed, at the stakeholder's word. Both rested
   on ``STKH_WIRING_CHECKED``, and on reading its "invalid workflow" as
   including one that leaves a parameter unbound - which was right while only
   an instance marked as an entry could be given its parameters from outside.
   ``STKH_RUN_FROM_ANY_PARAMETER`` settles the meeting of the two goals:
   a parameter nothing binds is an input, and a workflow is not invalid for
   having one. ``STKH_WIRING_CHECKED`` itself is unchanged, and still holds:
   a run whose inputs are not each given is refused before any node runs
   (``FEAT_RUN_ENTRY_SOURCE_EXACT``, changed beside this). Keeping the defect
   and exempting parameters an invocation would give was the other option, and
   is not one: the validator reads a definition, and cannot know what a run
   will be given. What is lost is recorded in the stakeholder requirement: a
   forgotten binding is found when a run starts rather than by the validator.

   Verdicts on the impact analysis (``EVD_IMPACT_WIRING_REQUIRED_BOUND``):

   - Up, ``STKH_WIRING_CHECKED``: unchanged, as above.
   - Down, ``ARCH_WIRING``: no longer realises the feature; its body counts
     seven defect classes where it counted eight. ``IMPL_WIRING_REQUIRED_BOUND``:
     removed with the check it marked, and ``WiringDefect`` loses the variant.
     ``TEST_WIRING_EVERY_UNBOUND_REQUIRED_IS_REPORTED`` and
     ``TEST_WIRING_UNWIRED_INSTANCE_IS_REPORTED``: removed with their tests.
     ``TEST_WIRING_DEGENERATE_DECLARATIONS_PASS``: kept, verifying
     ``CREQ_VALIDATOR_ACCEPTS_WELL_FORMED`` now, with instances holding
     parameters nothing binds in place of the entry node. Their runs follow
     their cases.
   - Sideways, the twelve other requirements of ``COMP_WIRING_VALIDATOR`` and
     the ten features beside it in ``ARCH_WIRING``: unaffected in what they
     oblige, none being about a parameter with no binding. Six cases that used
     an unbound parameter only to show the walk goes on use another defect
     now - ``TEST_WIRING_ALL_FOUR_CLASSES_REPORTED``,
     ``TEST_WIRING_INSTANCE_OF_MISSING_TYPE_IS_REPORTED``,
     ``TEST_WIRING_UNDECLARED_PARAMETER_IS_REPORTED``,
     ``TEST_WIRING_SHARED_INSTANCE_NAME_IS_REPORTED``,
     ``TEST_WIRING_NAME_DEFECTS_REPORTED_TOGETHER`` and
     ``TEST_RUN_REFUSAL_CARRIES_EVERY_DEFECT`` - each keeping what it asserts.
   - Text, the line of ``features/run`` naming the feature, and the marker in
     ``wiring.rs``: the first reworded, the second removed.

.. dec:: Revised: every input of a run is given exactly once, wherever it is
   :id: DEC_CHANGE_RUN_ENTRY_SOURCE_EXACT
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_IMPACT_RUN_ENTRY_SOURCE_EXACT
   :statement: Agconflo's requirements project shall state FEAT_RUN_ENTRY_SOURCE_EXACT and CREQ_RUN_REFUSES_UNFILLED_SIGNATURE of the parameters nothing binds because the stakeholder decided those are a workflow's inputs.

   Amends ``FEAT_RUN_ENTRY_SOURCE_EXACT``, adding
   ``STKH_RUN_FROM_ANY_PARAMETER`` to what it derives from, and
   ``CREQ_RUN_REFUSES_UNFILLED_SIGNATURE``.

   Before, the feature: "If an entry parameter of a run is not filled by
   exactly one context of its declared type, then Agconflo shall refuse to
   start that run." After: "If a parameter of a run's workflow that nothing
   binds is not given exactly one context of its declared type, then Agconflo
   shall refuse to start that run." Before, the component: "If an entry
   parameter of a workflow is not filled by exactly one context of its
   declared type, then Workflow run shall refuse to start a run of that
   workflow." After: "If a parameter of a workflow that nothing binds is not
   given exactly one context of its declared type, then Workflow run shall
   refuse to start a run of that workflow."

   Raised with ``DEC_CHANGE_WIRING_REQUIRED_BOUND``, by the same decision.

   Justification: a parent was added, at the stakeholder's word.
   ``STKH_EXPLICIT_CONTEXT`` is unchanged, and the feature still answers it at
   the boundary where a node's context comes from outside the graph. What
   that boundary is was the entry mark; ``STKH_RUN_FROM_ANY_PARAMETER`` makes
   it every parameter nothing binds, and the feature now derives from both.
   The new statement names no mechanism: it is the old one asked of the
   inputs as the stakeholder defines them. It would read the same had the
   repetition feature never been planned. The id keeps its old word; renaming
   it was the alternative, a removal and an addition for one change of
   meaning.

   Verdicts on the impact analysis (``EVD_IMPACT_RUN_ENTRY_SOURCE_EXACT``):

   - Up, ``STKH_EXPLICIT_CONTEXT``: unchanged; ``STKH_RUN_FROM_ANY_PARAMETER``
     added.
   - Down, ``ARCH_RUN``: unchanged, still realising the feature.
     ``IMPL_RUN_SIGNATURE``: changing, walking every instance's parameters in
     place of entry instances'. ``TEST_RUN_MISSING_ARGUMENT_IS_REFUSED``,
     ``TEST_RUN_ARGUMENT_OF_WRONG_TYPE_IS_REFUSED``,
     ``TEST_RUN_ARGUMENT_FOR_NO_PARAMETER_IS_REFUSED`` and
     ``TEST_RUN_SIGNATURE_FILLED_EXACTLY_STARTS``: unchanged in what they
     assert, reworded. ``TEST_RUN_ENTRY_PARAMETER_ALSO_BOUND_IS_REFUSED``:
     replaced by ``TEST_RUN_BOUND_PARAMETER_GIVEN_AN_ARGUMENT_IS_REFUSED``,
     since a bound parameter with no argument is no longer refused and is
     that case's control. ``TEST_RUN_INPUTS_GIVEN_WHERE_NOTHING_BINDS`` is
     added for the shape the change exists for. Their runs follow their cases.
   - Sideways, the twenty other requirements of ``COMP_WORKFLOW_RUN`` and the
     nine features beside it in ``ARCH_RUN``: unaffected, none being about
     what a run is started with but
     ``CREQ_RUN_REFUSES_SHARED_ARGUMENT_IDENTIFIER``, which is about the
     arguments' identifiers whatever they fill.
   - Text, the marker in ``run.rs``: retitled.

.. dec:: Revised: text is given for any instance's parameter, not an entry's
   :id: DEC_CHANGE_RUNNER_ARGUMENTS
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_IMPACT_RUNNER_ARGUMENTS
   :statement: Agconflo's requirements project shall state CREQ_RUNNER_ARGUMENTS_AS_TEXT and CREQ_RUNNER_REFUSES_UNKNOWN_ARGUMENT of any instance's parameter because there are no entry instances.

   Amends ``CREQ_RUNNER_ARGUMENTS_AS_TEXT`` and
   ``CREQ_RUNNER_REFUSES_UNKNOWN_ARGUMENT``.

   Before: "When text is given for an entry instance's parameter, Runner shall
   supply the run a context of the type that parameter declares holding that
   text exactly." After: "When text is given for an instance's parameter,
   Runner shall supply the run a context of the type that parameter declares
   holding that text exactly." Before: "If text is given for a parameter no
   entry instance of the workflow declares, then Runner shall refuse the run
   before any node runs and name that instance and parameter." After: "If
   text is given for a parameter no instance of the workflow declares, then
   Runner shall refuse the run before any node runs and name that instance
   and parameter."

   Raised with ``DEC_CHANGE_WIRING_REQUIRED_BOUND``, by the same decision of
   the maintainer's (``STKH_RUN_FROM_ANY_PARAMETER``).

   Justification: each is wrong against its own parent now.
   ``FEAT_RUNNER_STARTS_FROM_DOCUMENTS`` performs a run "with the arguments
   that person gave", and ``STKH_RUN_FROM_DOCUMENTS`` above it asks nothing
   about entries; the statements named the one kind of instance that could be
   given its parameters when they were written, the shape
   ``EVD_AMENDMENTS_CAME_SIDEWAYS`` found behind most changes. With no entry
   instances, the old first statement obliges nothing and the second refuses
   every argument. The new statements drop the word and name no mechanism;
   which declared parameters a run takes stays the run's to say.

   Verdicts on the impact analysis (``EVD_IMPACT_RUNNER_ARGUMENTS``):

   - Up, ``FEAT_RUNNER_STARTS_FROM_DOCUMENTS`` and ``STKH_RUN_FROM_DOCUMENTS``:
     unchanged.
   - Down, ``IMPL_RUNNER_ARGUMENTS``: changing, looking for the declaration on
     any instance. ``TEST_RUNNER_ARGUMENTS_KEPT_EXACTLY``: unchanged in what it
     asserts. ``TEST_RUNNER_UNKNOWN_ARGUMENT_REFUSED``: its third shape, a
     parameter of an instance that is not an entry, is refused by the run now
     rather than the runner, and is kept as the case's control.
     ``TEST_RUNNER_ARGUMENT_FOR_ANY_INSTANCE_TAKEN`` is added. Their runs
     follow their cases.
   - Sideways, the other fourteen requirements of ``COMP_RUNNER``: unaffected,
     none being about what text is given for.
   - Outside the analysis, since it shares no component:
     ``CREQ_COMMAND_READS_THE_COMMAND`` passes the arguments through
     unchanged, and the command line's help text drops the word.
   - Text, the marker in ``runner.rs``: now following
     ``DEC_ARGUMENTS_AS_TEXT_PER_PARAMETER``.

.. dec:: Revised: quiescence is that nothing can activate, not that everything has run
   :id: DEC_CHANGE_SCHEDULER_NONE_READY
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_IMPACT_SCHEDULER_NONE_READY
   :statement: Agconflo's requirements project shall state CREQ_SCHEDULER_NONE_READY as its parent states quiescence because it named the once-per-run mechanism as the only way an instance cannot activate.

   Amends ``CREQ_SCHEDULER_NONE_READY``.

   Before: "If every instance of a workflow has either produced an output or
   lacks a context for a parameter it binds, then Run scheduler shall report
   that no instance may activate." After: "If no instance of a workflow can
   activate, then Run scheduler shall report that no instance may activate."

   Raised by the repetition feature, whose ``CREQ_SCHEDULER_OFFERS_AGAIN``
   offers an instance that has produced when an edge into it holds something
   new: under the old statement that instance meets "has produced" and the
   scheduler would be obliged to report quiescence while offering it.

   Justification: it is wrong against its own parent. ``FEAT_RUN_QUIESCENCE_ENDS``
   ends a run when "no node instance of a run can activate", and
   ``STKH_STUCK_RUN`` above it reports a run in which no node can make
   progress. Neither says why an instance cannot activate; the requirement
   named the two reasons of its day - it has run, which
   ``DEC_ACTIVATION_ONCE_PER_RUN`` made final, or it lacks an input - as the
   only ones, the shape ``EVD_AMENDMENTS_CAME_SIDEWAYS`` found behind most
   changes. That decision is superseded (``DEC_EDGE_GENERATIONS``), and the
   parent could hold while the old statement is false. The new one is the
   parent's own word asked of the scheduler, and would read the same had
   repetition never been proposed; which instances can activate is what the
   scheduler's other requirements say.

   Verdicts on the impact analysis (``EVD_IMPACT_SCHEDULER_NONE_READY``):

   - Up, ``FEAT_RUN_QUIESCENCE_ENDS`` and ``STKH_STUCK_RUN``: unchanged.
   - Down, ``IMPL_SCHEDULER_NONE_READY``: unchanged, asking every instance
     whether it can activate. ``TEST_SCHEDULER_NONE_READY_ONLY_WHEN_NONE``:
     unchanged, the scheduler reporting none exactly when it offers none,
     over the same generated definitions.
     ``TEST_SCHEDULER_PARTIAL_INPUTS_STILL_QUIESCENT``: unchanged, a cycle
     whose instances each wait for one input. Both runs pass.
   - Sideways, the other five requirements of ``COMP_RUN_SCHEDULER``:
     unaffected; ``CREQ_SCHEDULER_OFFERS_AGAIN`` is the one the old wording
     contradicted, and now agrees with it.
   - Text, the marker in ``scheduler.rs`` and the lines of
     ``components/run`` and ``tests/run`` naming it: unchanged, each about
     quiescence being the scheduler's answer rather than about its mechanism.

.. dec:: Revised: repetition's architecture also realises a context coming first on its edge
   :id: DEC_CHANGE_ARCH_REPETITION
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_IMPACT_ARCH_REPETITION
   :statement: Agconflo's requirements project shall have ARCH_REPETITION realise FEAT_ARGUMENT_FIRST_ON_ITS_EDGE because the components it already uses answer for that feature.

   Amends ``ARCH_REPETITION``'s links: it realises
   ``FEAT_ARGUMENT_FIRST_ON_ITS_EDGE`` beside the two features it realised.
   Its statement is unchanged.

   Raised by the new feature, which needs an architecture, and needs nothing
   beyond the workflow run that already answers for the edges here.

   Justification: the architecture answers to the features it realises, and
   this adds one without changing what it says of the others. A new
   architecture using only the workflow run was the alternative, and would
   put what an edge holds first and what it holds after in two places.

   Verdicts on the impact analysis (``EVD_IMPACT_ARCH_REPETITION``):

   - Up, ``FEAT_REPEAT_ON_NEW_CONTEXTS``, ``FEAT_STANDING_SERVES_LATER_PASSES``
     and ``STKH_REPETITION``: unchanged; ``FEAT_ARGUMENT_FIRST_ON_ITS_EDGE``
     and ``STKH_RUN_FROM_ANY_PARAMETER`` added above it.
   - Down: nothing links to it.
   - Sideways, the 35 requirements of the three components it uses:
     unaffected; ``CREQ_RUN_ARGUMENT_FIRST_ON_ITS_EDGE`` is added to the
     workflow run's.
   - Text, the line of ``components/repetition`` naming it: unchanged.

.. dec:: Revised: taking a branch's architecture also realises a router's declared branches
   :id: DEC_CHANGE_ARCH_ROUTING
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_IMPACT_ARCH_ROUTING
   :statement: Agconflo's requirements project shall have ARCH_ROUTING realise FEAT_ROUTE_IS_A_DECLARED_BRANCH and FEAT_BRANCHES_CHECKED because the five components it uses answer for both.

   Amends ``ARCH_ROUTING``'s links: it realises
   ``FEAT_ROUTE_IS_A_DECLARED_BRANCH`` and ``FEAT_BRANCHES_CHECKED`` beside
   the two features it realised. Its statement and the components it uses are
   unchanged.

   Raised by declaring a router's branches (``DEC_ROUTER_BRANCHES_DECLARED``),
   whose two features need an architecture: the reader reads the branches, the
   validator checks them, the run refuses a naming that is none of them, and
   the script host refuses a person's.

   Justification: the architecture answers to the features it realises, and
   this adds two without changing what it says of the others. Each is about
   taking a branch, and needs no component the architecture does not already
   use. A second architecture over the same five components was the
   alternative, and would split one router's naming across two allocations.

   Verdicts on the impact analysis (``EVD_IMPACT_ARCH_ROUTING``):

   - Up, ``FEAT_ROUTE_WALKS_CHOSEN_EDGES``, ``FEAT_ROUTE_RECORDED``,
     ``STKH_ROUTING``, ``STKH_ONE_OUTPUT`` and ``STKH_RESUMABLE_RUN``:
     unchanged; ``FEAT_ROUTE_IS_A_DECLARED_BRANCH``, ``FEAT_BRANCHES_CHECKED``,
     ``STKH_REPETITION`` and ``STKH_WIRING_CHECKED`` added above it.
   - Down: nothing links to it.
   - Sideways, the 86 requirements of the five components it uses:
     unaffected; ``CREQ_READER_READS_BRANCHES``, ``CREQ_VALIDATOR_BRANCHES``,
     ``CREQ_RUN_REFUSES_UNDECLARED_BRANCH`` and
     ``CREQ_HOST_REFUSES_PERSON_ROUTE_NOT_A_BRANCH`` are added to theirs.
     ``CREQ_RUN_REFUSES_BAD_ROUTE`` keeps its statement: a naming that is no
     branch is a further refusal, not a change to the three it lists.
   - Text, the line of ``components/routing`` naming it: unchanged.

The eight records below are one change, made on 2026-10-04: the passes a
repeated node is given its contexts on are worked out from the wiring rather
than counted per edge (``DEC_PASS_CLOCKS``), and no output is declared
standing (``DEC_STANDING_REFUSED``). Two measurements raised it
(``EVD_PASSES_MISPAIRED_ACROSS_A_BRANCH``, ``EVD_STANDING_BY_INSTANCE_ORDER``),
and each impact analysis was taken on ``2ea52d1``, before any of the eight
changed.

.. dec:: Restated: a node runs again on each pass its inputs reach, given that pass's contexts
   :id: DEC_CHANGE_REPEAT_ON_NEW_CONTEXTS
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_IMPACT_REPEAT_ON_NEW_CONTEXTS
   :statement: Agconflo's requirements project shall state FEAT_REPEAT_ON_NEW_CONTEXTS as a node given the contexts of one pass because the mechanism it named gave a node two passes' contexts while it held.

   Amends ``FEAT_REPEAT_ON_NEW_CONTEXTS``.

   Before: "When every edge into a node instance that has run holds a context
   that stands or one it has not taken and at least one holds one it has not
   taken, Agconflo shall activate that instance again." After: "When the
   inputs of a node instance hold the contexts of a pass it has not run,
   Agconflo shall activate the instance with the contexts of that pass."

   Raised by ``EVD_PASSES_MISPAIRED_ACROSS_A_BRANCH``: a node joining an edge
   a router walks on some passes with one walked on every pass was activated
   under the old statement, every edge holding a context not taken, and was
   given the first pass's context beside the second's.

   Justification: it is wrong against its own parent. ``STKH_REPETITION`` lets
   a workflow repeat part of itself and leaves how one pass's contexts are
   kept apart from the next's to a decision; the old statement named the
   mechanism of the day - edges, contexts taken and not taken, outputs that
   stand - and held while it failed to keep passes apart, so it claimed both
   more than its parent (a mechanism) and less (a pass given its own). The
   new statement says what the parent needs of any mechanism, and would read
   the same had no measurement been taken.

   Verdicts on the impact analysis (``EVD_IMPACT_REPEAT_ON_NEW_CONTEXTS``):

   - Up, ``STKH_REPETITION``: unchanged.
   - Down: ``CREQ_SCHEDULER_TAKES_EARLIEST`` is removed for
     ``CREQ_SCHEDULER_GIVES_ONE_PASS`` (``DEC_CHANGE_SCHEDULER_TAKES_EARLIEST``);
     ``CREQ_SCHEDULER_OFFERS_AGAIN`` and ``CREQ_RUN_WALKS_EVERY_EDGE`` are
     restated (``DEC_CHANGE_SCHEDULER_OFFERS_AGAIN``,
     ``DEC_CHANGE_RUN_WALKS_EVERY_EDGE``); ``ARCH_REPETITION`` changes its
     links (``DEC_CHANGE_ARCH_REPETITION_PASSES``). ``IMPL_SCHEDULER_READY``
     now implements the restated requirements; ``IMPL_SCHEDULER_TAKES_EARLIEST``
     goes with its code. ``TEST_SCHEDULER_TAKES_EARLIEST`` and
     ``TEST_SCHEDULER_OFFERED_AGAIN_ON_SOMETHING_NEW`` go with their tests, for
     ``TEST_SCHEDULER_OFFERED_ONCE_PER_PASS``, ``TEST_RUN_BRANCH_READS_ITS_OWN_PASS``
     and ``TEST_RUN_PASSES_IGNORE_INSTANCE_ORDER``.
     ``TEST_RUN_OUTPUT_WALKS_EVERY_EDGE`` and ``TEST_SCRIPTED_REVIEW_LOOP`` are
     unchanged and pass.
   - Sideways, the requirements of the run scheduler and the workflow run:
     unaffected but those named above; ``FEAT_STANDING_SERVES_LATER_PASSES``,
     beside it under ``ARCH_REPETITION``, is removed
     (``DEC_CHANGE_STANDING_SERVES_LATER_PASSES``), and
     ``FEAT_ARGUMENT_FIRST_ON_ITS_EDGE`` is unchanged.
   - Text, two lines of the records above naming it: unchanged, as history.

.. dec:: Removed: an output declared standing serving later passes
   :id: DEC_CHANGE_STANDING_SERVES_LATER_PASSES
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_IMPACT_STANDING_SERVES_LATER_PASSES
   :statement: Agconflo's requirements project shall replace FEAT_STANDING_SERVES_LATER_PASSES with FEAT_ENCLOSING_PASS_SERVES because it named a declaration whose result turned on the order instances are written in.

   Removes ``FEAT_STANDING_SERVES_LATER_PASSES``: "When a node instance
   produces an output that stands, Agconflo shall give that output to every
   later activation reading it until the instance produces another."
   ``FEAT_ENCLOSING_PASS_SERVES`` takes its place: "When a node instance
   reads an output made on passes enclosing its own, Agconflo shall give each
   of its activations the context made on the enclosing pass that activation
   belongs to."

   Raised by ``EVD_STANDING_BY_INSTANCE_ORDER``: a standing output made again
   reached its reader as each pass's own or as the first pass's on every
   pass, depending on which of two instances the definition wrote first.

   Justification: it is wrong against its own parent. ``STKH_REPETITION``
   needs a pass to read what does not change from pass to pass; the
   requirement named one mechanism for it, a standing output, and its "every
   later activation" served whichever context was latest when the reader
   ran, so it held while a pass was given another's. What the parent needs is
   that a context made outside a pass reaches it as made on the pass it
   belongs to, which the new requirement says without naming how.

   Verdicts on the impact analysis (``EVD_IMPACT_STANDING_SERVES_LATER_PASSES``):

   - Up, ``STKH_REPETITION``: unchanged.
   - Down: ``CREQ_READER_READS_STANDING`` and ``CREQ_SCHEDULER_STANDING_SERVES``
     are removed (``DEC_CHANGE_READER_READS_STANDING``,
     ``DEC_CHANGE_SCHEDULER_STANDING_SERVES``), with ``IMPL_READER_STANDING``,
     ``IMPL_SCHEDULER_STANDING`` and ``IMPL_SCHEDULER_TAKES_EARLIEST`` and the
     tests behind ``TEST_READER_STANDING_READ`` and
     ``TEST_SCHEDULER_STANDING_SERVES_LATER``. ``ARCH_REPETITION`` realises
     ``FEAT_ENCLOSING_PASS_SERVES`` instead.
   - Sideways, the reader's and the scheduler's requirements: unaffected but
     those named; the reader refuses the key under ``CREQ_READER_FAULT_LOCATED``
     (``DEC_STANDING_REFUSED``).
   - Text, the records above and ``TEST_SCRIPTED_REVIEW_LOOP``'s body: the
     records unchanged as history, the test case's body naming the new
     requirement.

.. dec:: Revised: repetition's architecture realises passes, and no longer uses the reader
   :id: DEC_CHANGE_ARCH_REPETITION_PASSES
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_IMPACT_ARCH_REPETITION_PASSES
   :statement: Agconflo's requirements project shall have ARCH_REPETITION realise the repetition features as they now stand and use the run scheduler and the workflow run because nothing of repetition is the topology reader's any more.

   Amends ``ARCH_REPETITION``. Before: it realised
   ``FEAT_REPEAT_ON_NEW_CONTEXTS``, ``FEAT_STANDING_SERVES_LATER_PASSES`` and
   ``FEAT_ARGUMENT_FIRST_ON_ITS_EDGE``, used the topology reader, the run
   scheduler and the workflow run, and stated "Agconflo shall allocate
   repetition to the topology reader, the run scheduler and the workflow
   run." After: it realises ``FEAT_REPEAT_ON_NEW_CONTEXTS``,
   ``FEAT_ENCLOSING_PASS_SERVES``, ``FEAT_ARGUMENT_FIRST_ON_ITS_EDGE`` and
   ``FEAT_PASSES_PAIRED_BEFORE_RUN``, uses the run scheduler and the workflow
   run, and states "Agconflo shall allocate repetition to the run scheduler
   and the workflow run."

   Justification: its parents changed. One feature it realised is removed
   and two are added, and the one requirement it allocated to the topology
   reader, reading ``standing``, is removed with the declaration; refusing the
   key is the reader's under its own feature (``CREQ_READER_FAULT_LOCATED``).

   Verdicts on the impact analysis (``EVD_IMPACT_ARCH_REPETITION_PASSES``):

   - Up: as above.
   - Down: nothing links to it.
   - Sideways, the 38 requirements of the three components it used: those of
     the reader unaffected, and no longer beside it; the scheduler's and the
     run's as the records above and below say.
   - Text, the line of ``components/repetition`` naming it: rewritten for two
     components; the records naming it unchanged as history.

.. dec:: Removed: an activation taking the earliest context each edge holds
   :id: DEC_CHANGE_SCHEDULER_TAKES_EARLIEST
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_IMPACT_SCHEDULER_TAKES_EARLIEST
   :statement: Agconflo's requirements project shall replace CREQ_SCHEDULER_TAKES_EARLIEST with CREQ_SCHEDULER_GIVES_ONE_PASS because its parent now asks for the contexts of one pass.

   Removes ``CREQ_SCHEDULER_TAKES_EARLIEST``: "Run scheduler shall give an
   instance's activation from each edge into it the earliest context that
   edge holds that the instance has not taken, or the context that stands on
   it when it holds none." ``CREQ_SCHEDULER_GIVES_ONE_PASS`` takes its place:
   "When an instance reads an output made on its own passes, Run scheduler
   shall give each of its activations the context made on the same pass."

   Justification: its parent changed (``DEC_CHANGE_REPEAT_ON_NEW_CONTEXTS``).
   The earliest context not taken is what gave the join two passes'
   contexts, and is now its first failure mode.

   Verdicts on the impact analysis (``EVD_IMPACT_SCHEDULER_TAKES_EARLIEST``):

   - Up, ``FEAT_REPEAT_ON_NEW_CONTEXTS`` and ``STKH_REPETITION``: as above.
   - Down: ``IMPL_SCHEDULER_TAKES_EARLIEST`` goes with its code;
     ``IMPL_SCHEDULER_READY`` implements the new requirement through
     ``IMPL_SCHEDULER_ONE_PASS``; ``TEST_SCHEDULER_TAKES_EARLIEST`` goes with
     its test.
   - Sideways, the scheduler's other requirements: unaffected but
     ``CREQ_SCHEDULER_OFFERS_AGAIN`` and ``CREQ_SCHEDULER_STANDING_SERVES``,
     recorded beside this.
   - Text, its two markers in ``scheduler.rs``: gone with the code.

.. dec:: Restated: an instance is offered on its next pass once its inputs hold it
   :id: DEC_CHANGE_SCHEDULER_OFFERS_AGAIN
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_IMPACT_SCHEDULER_OFFERS_AGAIN
   :statement: Agconflo's requirements project shall state CREQ_SCHEDULER_OFFERS_AGAIN as offering an instance on its next pass because its parent now asks for the contexts of one pass.

   Amends ``CREQ_SCHEDULER_OFFERS_AGAIN``. Before: "When every edge into an
   instance that has run holds a context that stands or one it has not taken
   and at least one holds one it has not taken, Run scheduler shall offer that
   instance for activation." After: "When every input of an instance holds a
   context of the next of its passes, Run scheduler shall offer that instance
   for activation."

   Justification: its parent changed (``DEC_CHANGE_REPEAT_ON_NEW_CONTEXTS``).
   "Something new on one edge" is what the old mechanism offered on; one
   pass at a time is what the parent asks, and an instance whose inputs all
   come once is on the run's one pass and offered once, which the old
   statement needed "at least one" to say.

   Verdicts on the impact analysis (``EVD_IMPACT_SCHEDULER_OFFERS_AGAIN``):

   - Up: as above.
   - Down: ``IMPL_SCHEDULER_READY`` implements the restated requirement;
     ``TEST_SCHEDULER_OFFERED_AGAIN_ON_SOMETHING_NEW`` is replaced by
     ``TEST_SCHEDULER_OFFERED_ONCE_PER_PASS``.
   - Sideways: as for ``DEC_CHANGE_SCHEDULER_TAKES_EARLIEST``.
   - Text, two lines of ``components/run``, one of ``components/yield`` and
     two of a record above: each names it for an instance that has run being
     offered again, which still holds; unchanged.

.. dec:: Removed: a standing output serving every later activation
   :id: DEC_CHANGE_SCHEDULER_STANDING_SERVES
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_IMPACT_SCHEDULER_STANDING_SERVES
   :statement: Agconflo's requirements project shall replace CREQ_SCHEDULER_STANDING_SERVES with CREQ_SCHEDULER_READS_ENCLOSING_PASS because its parent was replaced.

   Removes ``CREQ_SCHEDULER_STANDING_SERVES``: "When an instance whose output
   stands has produced, Run scheduler shall give that output to every later
   activation reading it until the instance produces another."
   ``CREQ_SCHEDULER_READS_ENCLOSING_PASS`` takes its place, derived from
   ``FEAT_ENCLOSING_PASS_SERVES``.

   Justification: its parent changed (``DEC_CHANGE_STANDING_SERVES_LATER_PASSES``).

   Verdicts on the impact analysis (``EVD_IMPACT_SCHEDULER_STANDING_SERVES``):

   - Up, ``FEAT_STANDING_SERVES_LATER_PASSES`` and ``STKH_REPETITION``: the
     first removed, the second unchanged.
   - Down: ``IMPL_SCHEDULER_STANDING`` and ``IMPL_SCHEDULER_TAKES_EARLIEST`` go
     with their code, and ``TEST_SCHEDULER_STANDING_SERVES_LATER`` with its
     test; ``IMPL_SCHEDULER_ENCLOSING_PASS`` and
     ``TEST_RUN_BRANCH_READS_ITS_OWN_PASS`` meet and check the new requirement.
   - Sideways: as for ``DEC_CHANGE_SCHEDULER_TAKES_EARLIEST``.
   - Text, its two markers in ``scheduler.rs``: gone with the code.

.. dec:: Removed: a node type declaring its output standing, read
   :id: DEC_CHANGE_READER_READS_STANDING
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_IMPACT_READER_READS_STANDING
   :statement: Agconflo's requirements project shall remove CREQ_READER_READS_STANDING because its parent was removed and the key it read is refused.

   Removes ``CREQ_READER_READS_STANDING``: "When a node type declares
   standing = true, Topology reader shall read that type's output as
   standing." Nothing takes its place: the key is refused at its place, under
   ``CREQ_READER_FAULT_LOCATED``, as ``optional`` and ``entry`` are
   (``DEC_STANDING_REFUSED``).

   Justification: its parent changed (``DEC_CHANGE_STANDING_SERVES_LATER_PASSES``).

   Verdicts on the impact analysis (``EVD_IMPACT_READER_READS_STANDING``):

   - Up: the first removed, ``STKH_REPETITION`` unchanged.
   - Down: ``IMPL_READER_STANDING`` is renamed ``IMPL_READER_ROUTES``, which
     reads ``routes`` alone; ``TEST_READER_STANDING_READ`` is replaced by
     ``TEST_READER_STANDING_REFUSED``, verifying ``CREQ_READER_FAULT_LOCATED``.
   - Sideways, the reader's other eight requirements: unaffected;
     ``CREQ_READER_FAULT_LOCATED`` gains the failure mode of a standing
     declaration read past.
   - Text, its marker in ``reader.rs``: replaced.

.. dec:: Restated: an output is held for every reader as its instance's next pass
   :id: DEC_CHANGE_RUN_WALKS_EVERY_EDGE
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_IMPACT_RUN_WALKS_EVERY_EDGE
   :statement: Agconflo's requirements project shall state CREQ_RUN_WALKS_EVERY_EDGE as an output held as its instance's next pass because its parent no longer names edges holding contexts in turn.

   Amends ``CREQ_RUN_WALKS_EVERY_EDGE``. Before: "When the caller reports the
   output of an instance whose node type does not route, Workflow run shall
   walk it along every edge out of that instance as the next context that
   edge holds." After: "When the caller reports the output of an instance
   whose node type does not route, Workflow run shall hold it as the output of
   the instance's next pass for every instance reading it."

   Justification: its parent changed (``DEC_CHANGE_REPEAT_ON_NEW_CONTEXTS``).
   What an edge holds next was the queue the old mechanism kept; what the
   parent needs of the run is that each output is there for every reader as
   the pass it was made on.

   Verdicts on the impact analysis (``EVD_IMPACT_RUN_WALKS_EVERY_EDGE``):

   - Up: as above.
   - Down: ``IMPL_RUN_WALKS_EVERY_EDGE`` moves to the code holding each
     output by pass; ``TEST_RUN_OUTPUT_WALKS_EVERY_EDGE`` is unchanged and
     passes, its two readers each given the first output and then the second.
   - Sideways, the run's other requirements: unaffected but
     ``CREQ_RUN_REFUSES_UNPAIRED``, added beside it.
   - Text, its marker in ``scheduler.rs``: moved with the code.

.. dec:: Removed: a context a run is given for a wired parameter coming first
   :id: DEC_CHANGE_ARGUMENT_FIRST_ON_ITS_EDGE
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_IMPACT_ARGUMENT_FIRST_ON_ITS_EDGE
   :statement: Agconflo's requirements project shall replace FEAT_ARGUMENT_FIRST_ON_ITS_EDGE with FEAT_FIRST_CONTEXT_DECLARED because it named the run's arguments as where a loop starts, which neither of its parents asks for.

   Removes ``FEAT_ARGUMENT_FIRST_ON_ITS_EDGE``: "When a run is started with a
   context for a parameter a binding fills, Agconflo shall give that context
   to the parameter's instance before any context walked along the binding."
   ``FEAT_FIRST_CONTEXT_DECLARED`` takes its place: "When a binding declares
   its first context, Agconflo shall give that context to the binding's
   instance before any context walked along the binding."

   Raised by ``EVD_LOOP_FIRST_CONTEXT_UNCHECKED``: a check passed a loop that
   every run of it left stuck or refused, because what started the loop was
   in no run but the one given it.

   Justification: it is wrong against its own parents, claiming more than
   they need. ``STKH_RUN_FROM_ANY_PARAMETER`` gives a run contexts for the
   parameters nothing binds - its body says a workflow is invoked by giving
   each of those its context - and asks nothing of a parameter a binding
   fills. ``STKH_REPETITION`` needs a loop to have a way into its first pass
   and leaves how to a decision. The requirement named one way, the run's
   argument, as the requirement itself; a loop started from what its
   workflow declares meets both parents and broke it. What
   ``STKH_REPETITION`` needs of a way in, said where ``STKH_WIRING_CHECKED``
   can see it, is the new requirement, and it no longer derives from
   ``STKH_RUN_FROM_ANY_PARAMETER``.

   Verdicts on the impact analysis (``EVD_IMPACT_ARGUMENT_FIRST_ON_ITS_EDGE``):

   - Up, ``STKH_REPETITION`` and ``STKH_RUN_FROM_ANY_PARAMETER``: unchanged.
   - Down: ``CREQ_RUN_ARGUMENT_FIRST_ON_ITS_EDGE`` is removed with it
     (``DEC_CHANGE_RUN_ARGUMENT_FIRST_ON_ITS_EDGE``); ``ARCH_REPETITION``
     realises the new requirement (``DEC_CHANGE_ARCH_REPETITION_FIRST``);
     ``TEST_RUN_ARGUMENT_FIRST_ON_ITS_EDGE`` goes with its test, for
     ``TEST_RUN_FIRST_CONTEXT_DECLARED_COMES_FIRST``, and its run with it.
   - Sideways, the workflow run's requirements: unaffected but
     ``CREQ_RUN_REFUSES_UNFILLED_SIGNATURE``, whose failure modes now say an
     argument for a bound parameter matches nothing of the workflow's - its
     statement, about the parameters nothing binds, is unchanged - and
     ``CREQ_RUN_HOLDS_DECLARED_FIRST``, added beside it. The three features
     beside it under ``ARCH_REPETITION``: unaffected, and
     ``FEAT_PASSES_PAIRED_BEFORE_RUN``'s body changes with the pairing check's
     place in a record of its own.
   - Text, six lines of the records above and one of their evidence:
     unchanged, as history.

.. dec:: Removed: a run holding a context given for a wired parameter first
   :id: DEC_CHANGE_RUN_ARGUMENT_FIRST_ON_ITS_EDGE
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_IMPACT_RUN_ARGUMENT_FIRST_ON_ITS_EDGE
   :statement: Agconflo's requirements project shall replace CREQ_RUN_ARGUMENT_FIRST_ON_ITS_EDGE with CREQ_SCHEDULER_GIVES_FIRST and CREQ_RUN_HOLDS_DECLARED_FIRST because its parent is removed.

   Removes ``CREQ_RUN_ARGUMENT_FIRST_ON_ITS_EDGE``: "When a run is started
   with a context for a parameter a binding fills, Workflow run shall hold
   that context as the first that edge holds, before any context walked along
   it." ``CREQ_SCHEDULER_GIVES_FIRST`` takes its place for when the context is
   given: "When a binding declares its first context, Run scheduler shall give
   that context to the binding's instance on the binding's first pass and what
   the binding carries on each pass after." ``CREQ_RUN_HOLDS_DECLARED_FIRST``
   for where it comes from: "When a run starts, Workflow run shall hold for
   each binding declaring its first context a context holding the declared
   text, of the type the binding's parameter is declared for."

   Justification: its parent changed - removed for
   ``FEAT_FIRST_CONTEXT_DECLARED`` (``DEC_CHANGE_ARGUMENT_FIRST_ON_ITS_EDGE``).
   Its three failure modes stand: the first two are the scheduler's now, and
   the third, refusing the context as a second source, is what the run now
   does to an argument for a bound parameter, on purpose.

   Verdicts on the impact analysis
   (``EVD_IMPACT_RUN_ARGUMENT_FIRST_ON_ITS_EDGE``):

   - Up, ``FEAT_ARGUMENT_FIRST_ON_ITS_EDGE``: removed, above; its two
     stakeholder requirements unchanged.
   - Down: ``TEST_RUN_ARGUMENT_FIRST_ON_ITS_EDGE`` and its run go with the
     test; ``IMPL_SCHEDULER_ONE_PASS`` implements the scheduler's requirement,
     and ``IMPL_RUN_MAKES_FIRSTS`` and ``IMPL_RUN_HELD_ARGUMENTS`` the run's.
   - Sideways, the workflow run's requirements: as the record above says.
   - Text: the failure mode of ``CREQ_RUN_REFUSES_UNFILLED_SIGNATURE`` and
     the review loop's test case, rewritten for the new requirements; one line
     of the records above, unchanged as history.

.. dec:: Amended: repetition's architecture realises a declared first context and uses the reader and the validator
   :id: DEC_CHANGE_ARCH_REPETITION_FIRST
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_IMPACT_ARCH_REPETITION_FIRST
   :statement: Agconflo's requirements project shall have ARCH_REPETITION realise FEAT_FIRST_CONTEXT_DECLARED and use the topology reader and the wiring validator because a loop's first context is now read from the workflow and checked there.

   Amends ``ARCH_REPETITION``. Before: it realised
   ``FEAT_REPEAT_ON_NEW_CONTEXTS``, ``FEAT_ENCLOSING_PASS_SERVES``,
   ``FEAT_ARGUMENT_FIRST_ON_ITS_EDGE`` and ``FEAT_PASSES_PAIRED_BEFORE_RUN``,
   used the run scheduler and the workflow run, and stated "Agconflo shall
   allocate repetition to the run scheduler and the workflow run." After: it
   realises ``FEAT_FIRST_CONTEXT_DECLARED`` in place of
   ``FEAT_ARGUMENT_FIRST_ON_ITS_EDGE``, uses the topology reader, the wiring
   validator, the run scheduler and the workflow run, and states "Agconflo
   shall allocate repetition to the topology reader, the wiring validator, the
   run scheduler and the workflow run."

   Justification: its parents changed. One feature it realised is replaced
   (``DEC_CHANGE_ARGUMENT_FIRST_ON_ITS_EDGE``), and what the new one and
   ``FEAT_PASSES_PAIRED_BEFORE_RUN`` need is now partly the reader's - the
   first context is written in the workflow document - and the validator's -
   the passes follow from the definition alone, so a node whose inputs share
   none, and a cycle no first context starts, are defects of the wiring
   (``DEC_PAIRING_IS_WIRING``, ``DEC_CYCLE_STARTED_BY_A_FIRST``).

   Verdicts on the impact analysis (``EVD_IMPACT_ARCH_REPETITION_FIRST``):

   - Up, its four features and three stakeholder requirements: as above;
     ``STKH_RUN_FROM_ANY_PARAMETER`` is no longer among them, and nothing
     changes for it.
   - Down: nothing links to it.
   - Sideways, the 31 requirements of the scheduler and the run: unaffected
     but those the records beside this name. The reader's and the
     validator's requirements are beside it now: unaffected, the reader
     gaining ``CREQ_READER_READS_FIRST`` and the validator the requirements
     the pairing check moves into.
   - Text: the opening of ``components/repetition``, rewritten for four
     components; fourteen lines of the records and their evidence, unchanged
     as history.

.. dec:: Restated: one output asked of a script as its type declares it
   :id: DEC_CHANGE_BEHAVIOUR_ONE_CONTEXT
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_BEHAVIOUR_ONE_CONTEXT
   :statement: Agconflo's requirements project shall state FEAT_BEHAVIOUR_ONE_CONTEXT of a node whose type declares an output and of one whose type declares none because one output is not always a context.

   Old: "If a node's script returns anything other than exactly one context
   that the run accepts, then Agconflo shall fail that activation." New: "If
   the script of a node whose type declares an output returns anything other
   than exactly one context that the run accepts or the script of one whose
   type declares none returns anything, then Agconflo shall fail that
   activation."

   Justification: wrong against its own parent. ``STKH_ONE_OUTPUT`` restricts
   a node to one output and does not say every output is a context; asking a
   context of every script claims more than that. The surplus is where it met
   a node whose one output is of another kind: a router's is the instances it
   names (``DEC_ROUTER_PASSES_ON_ITS_INPUTS``), and its type declares none
   (``DEC_ROUTER_DECLARES_NO_OUTPUT``). The new statement speaks of what a
   type declares, the parent's own terms.

   Verdicts on the impact analysis (``EVD_IMPACT_BEHAVIOUR_ONE_CONTEXT``):

   - Up, ``STKH_ONE_OUTPUT``: unchanged.
   - Down: ``CREQ_HOST_ONE_CONTEXT`` changes under its own record;
     ``CREQ_HOST_OUTPUT_REFUSAL_CARRIED`` and ``IMPL_SCRIPTED_FAIL`` are
     unaffected, an output the run refuses still failing the activation;
     ``TEST_HOST_NOT_ONE_CONTEXT_FAILS`` gains a router's case, and the other
     three cases are unaffected.
   - Sideways, the script host's requirements and ``ARCH_BEHAVIOUR``'s six
     other features: unaffected but those whose records sit beside this.
   - Text: none.

.. dec:: Restated: the host holds a script's return to its type's declared output
   :id: DEC_CHANGE_HOST_ONE_CONTEXT
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_HOST_ONE_CONTEXT
   :statement: Agconflo's requirements project shall state CREQ_HOST_ONE_CONTEXT of a node type that declares an output and of one that declares none because its parent now does.

   Old: "If a script returns anything other than exactly one context, then
   Script host shall fail that activation naming what was returned." New:
   "If the script of a node type that declares an output returns anything
   other than exactly one context or the script of one that declares none
   returns anything, then Script host shall fail that activation naming what
   was returned."

   Justification: its parent changed (``DEC_CHANGE_BEHAVIOUR_ONE_CONTEXT``).
   Its four failure modes stand; a fifth is added, a context returned where
   none is declared dropped without a word.

   Verdicts on the impact analysis (``EVD_IMPACT_HOST_ONE_CONTEXT``):

   - Up: changed above; ``STKH_ONE_OUTPUT`` unchanged.
   - Down: ``IMPL_HOST_ONE_CONTEXT`` changes to tell the two kinds of type
     apart; ``TEST_HOST_NOT_ONE_CONTEXT_FAILS`` gains a router's script
     returning a context, and its run is taken again.
   - Sideways, the script host's requirements: unaffected but those whose
     records sit beside this.
   - Text: the marker, unchanged.

.. dec:: Restated: a person's text is the output of a step declaring one
   :id: DEC_CHANGE_PERSON_TEXT_IS_THE_OUTPUT
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_PERSON_TEXT_IS_THE_OUTPUT
   :statement: Agconflo's requirements project shall state FEAT_PERSON_TEXT_IS_THE_OUTPUT of an activation whose node type declares an output because its parent asks that a person can supply a context and not that every step makes one.

   Old: "When a person's text is supplied for the activation a scripted
   run's record awaits, Agconflo shall continue that run from the record with
   a context holding that text as the activation's output." New: "When a
   person's text is supplied for the activation a scripted run's record awaits
   and its node type declares an output, Agconflo shall continue that run
   from the record with a context holding that text as the activation's
   output."

   Justification: wrong against its own parent, claiming more than it.
   ``STKH_HUMAN_IN_RUN`` lets a person supply a context; it does not make
   every step a person performs one that takes text as its output. A step
   whose type declares no output has no context for text to become.

   Verdicts on the impact analysis (``EVD_IMPACT_PERSON_TEXT_IS_THE_OUTPUT``):

   - Up, ``STKH_HUMAN_IN_RUN``: unchanged.
   - Down: ``CREQ_HOST_TAKES_PERSON_TEXT`` changes under its own record;
     ``IMPL_SCRIPTED_ANSWER`` changes with it; the four test cases are
     unaffected, each answering a step that declares an output.
   - Sideways: unaffected but those whose records sit beside this.
   - Text: the openings of ``features/runner`` and ``features/tools``, which
     cite it for a step that declares an output, unchanged.

.. dec:: Restated: the host takes a person's text for a step declaring an output
   :id: DEC_CHANGE_HOST_TAKES_PERSON_TEXT
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_HOST_TAKES_PERSON_TEXT
   :statement: Agconflo's requirements project shall state CREQ_HOST_TAKES_PERSON_TEXT of an activation whose node type declares an output because its parent now does.

   Old: "When a person's text is supplied for the activation a record's run
   awaits, Script host shall report as that activation's output a context of
   its declared type holding exactly the text from the source resumed with
   the record." New: "When a person's text is supplied for the awaited
   activation of a node type that declares an output, Script host shall
   report as its output a context of its declared type holding exactly that
   text from the source resumed with the record."

   Justification: its parent changed
   (``DEC_CHANGE_PERSON_TEXT_IS_THE_OUTPUT``). "The awaited activation" is the
   activation a record's run awaits, shortened to stay within a statement's
   length.

   Verdicts on the impact analysis (``EVD_IMPACT_HOST_TAKES_PERSON_TEXT``):

   - Up: changed above; ``STKH_HUMAN_IN_RUN`` unchanged.
   - Down: ``IMPL_SCRIPTED_ANSWER`` changes; its two test cases are
     unaffected.
   - Sideways: unaffected but those whose records sit beside this.
   - Text: the marker, and the body of ``CREQ_HOST_TAKES_PERSON_ROUTE``,
     which no longer cites it.

.. dec:: Restated: the runner takes a person's text for a step declaring an output
   :id: DEC_CHANGE_RUNNER_TAKES_THE_ANSWER
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_RUNNER_TAKES_THE_ANSWER
   :statement: Agconflo's requirements project shall state FEAT_RUNNER_TAKES_THE_ANSWER of a step whose node type declares an output because neither of its parents asks that every step take text as its output.

   Old: "When a person gives text for the step the record in a file awaits,
   Agconflo shall continue that run with the text as the step's output." New:
   "When a person gives text for the step the record in a file awaits and its
   node type declares an output, Agconflo shall continue that run with the
   text as the step's output."

   Justification: wrong against its own parents, claiming more than them, as
   ``DEC_CHANGE_PERSON_TEXT_IS_THE_OUTPUT`` says of its sibling.
   ``STKH_RUN_FROM_DOCUMENTS`` asks nothing of what an answer is.

   Verdicts on the impact analysis (``EVD_IMPACT_RUNNER_TAKES_THE_ANSWER``):

   - Up, ``STKH_HUMAN_IN_RUN`` and ``STKH_RUN_FROM_DOCUMENTS``: unchanged.
   - Down: ``CREQ_COMMAND_READS_THE_COMMAND`` and ``CREQ_RUNNER_ANSWERS``
     unaffected, each handing text on as given; their implementations change
     only for a route given without text; the four test cases are unaffected.
   - Sideways, the runner's and the command line's requirements and
     ``ARCH_RUNNER``'s ten other features: unaffected.
   - Text: none.

.. dec:: Restated: a router's script names its route and no output
   :id: DEC_CHANGE_HOST_ROUTE_NAMED
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_HOST_ROUTE_NAMED
   :statement: Agconflo's requirements project shall state CREQ_HOST_ROUTE_NAMED as reporting a router's names as its route and no output and derive it from FEAT_ROUTER_MAKES_NO_CONTEXT too because a router makes no context.

   Old: "When a router's script names the instances its run goes on to,
   Script host shall report those names with the activation's output." New:
   "When a router's script names the instances its run goes on to, Script
   host shall report those names as the activation's route and no output."
   Its parents gain ``FEAT_ROUTER_MAKES_NO_CONTEXT``.

   Justification: wrong against its own parents. ``STKH_ROUTING``, under
   ``FEAT_ROUTE_WALKS_CHOSEN_EDGES``, has a router pass on the contexts it was
   given rather than make new ones, and an output reported beside the names
   is a context the router made (``DEC_ROUTER_PASSES_ON_ITS_INPUTS``).

   Verdicts on the impact analysis (``EVD_IMPACT_HOST_ROUTE_NAMED``):

   - Up: unchanged, and the new feature added.
   - Down: ``IMPL_HOST_ROUTE`` changes; ``TEST_HOST_ROUTE_NAMED`` changes to
     a router's script returning nothing, and its run is taken again.
   - Sideways: unaffected but those whose records sit beside this.
   - Text: the marker, and the body of ``CREQ_HOST_TAKES_PERSON_ROUTE``,
     which still cites it.

.. dec:: Restated: a person's route is a router's step's whole answer
   :id: DEC_CHANGE_HOST_TAKES_PERSON_ROUTE
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_HOST_TAKES_PERSON_ROUTE
   :statement: Agconflo's requirements project shall state CREQ_HOST_TAKES_PERSON_ROUTE as reporting a person's names as the router's route and no output because a router makes no context.

   Old: "When a person's answer to the router's step a record's run awaits
   names instances, Script host shall report the answer's text as that step's
   output with those names as its route." New: "When a person's answer to the
   router's step a record's run awaits names instances, Script host shall
   report those names as that step's route and no output."

   Justification: wrong against its own parents. ``FEAT_PERSON_ROUTES``
   answers to ``STKH_ROUTING`` as well as ``STKH_HUMAN_IN_RUN``, and a person
   performing a router is the router: text reported as its output is a context
   the router made (``DEC_PERSON_ROUTE_IS_THE_ANSWER``).

   Verdicts on the impact analysis (``EVD_IMPACT_HOST_TAKES_PERSON_ROUTE``):

   - Up: unchanged.
   - Down: ``IMPL_SCRIPTED_ANSWER`` changes; ``TEST_SCRIPTED_PERSON_ROUTES``
     answers with the route alone, and its run is taken again.
   - Sideways: unaffected but those whose records sit beside this.
   - Text: the marker.

.. dec:: Restated: text for a router's step is refused
   :id: DEC_CHANGE_HOST_REFUSES_BAD_PERSON_ROUTE
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_HOST_REFUSES_BAD_PERSON_ROUTE
   :statement: Agconflo's requirements project shall state CREQ_HOST_REFUSES_BAD_PERSON_ROUTE as also refusing text for a router's step because a router makes no context.

   Old: "If a person's answer gives no route for a router's step or gives one
   for any other step or names an instance no edge out of the router enters,
   then Script host shall refuse it having run nothing." New: "If a person's
   answer gives no route or gives text for a router's step or gives a route
   for any other step or names an instance no edge out of the router enters,
   then Script host shall refuse it having run nothing."

   Justification: wrong against its own parents, for the reason
   ``DEC_CHANGE_HOST_TAKES_PERSON_ROUTE`` gives. Text kept would be a context
   the router made; text dropped would be an answer lost without a word.

   Verdicts on the impact analysis
   (``EVD_IMPACT_HOST_REFUSES_BAD_PERSON_ROUTE``):

   - Up: unchanged.
   - Down: ``IMPL_SCRIPTED_ANSWER`` changes;
     ``TEST_SCRIPTED_PERSON_BAD_ROUTE_REFUSED`` gains text given for a
     router's step, and its run is taken again.
   - Sideways: unaffected but those whose records sit beside this.
   - Text: the marker, and the body of
     ``CREQ_HOST_REFUSES_PERSON_ROUTE_NOT_A_BRANCH``, unchanged.

.. dec:: Restated: the runner takes a route without text
   :id: DEC_CHANGE_RUNNER_ANSWERS_WITH_ROUTE
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_RUNNER_ANSWERS_WITH_ROUTE
   :statement: Agconflo's requirements project shall state CREQ_RUNNER_ANSWERS_WITH_ROUTE as answering with a route given alone because a router's step takes no text.

   Old: "When text and a route are given for the step of an instance, Runner
   shall answer the record its file holds with that text and that route for
   that instance." New: "When a route is given for the step of an instance,
   Runner shall answer the record its file holds with that route for that
   instance."

   Justification: wrong against its own parents, for the reason
   ``DEC_CHANGE_HOST_TAKES_PERSON_ROUTE`` gives: requiring text beside the
   route would make every person performing a router write an answer that is
   then refused.

   Verdicts on the impact analysis
   (``EVD_IMPACT_RUNNER_ANSWERS_WITH_ROUTE``):

   - Up: unchanged.
   - Down: ``IMPL_RUNNER_ANSWER`` changes to take text as optional;
     ``TEST_RUNNER_PERSON_ROUTES`` answers with the route alone, and its run
     is taken again.
   - Sideways, the runner's requirements: unaffected.
   - Text: the marker.

.. dec:: Restated: the record holds a router's names as its entry
   :id: DEC_CHANGE_RECORD_HOLDS_ROUTES
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_RECORD_HOLDS_ROUTES
   :statement: Agconflo's requirements project shall state CREQ_RECORD_HOLDS_ROUTES as writing a router's names as its activation's entry because a router's activation has no output to write them beside.

   Old: "Run record shall write each router's output with the instances its
   activation named and resume the run walking those names." New: "Run record
   shall write the instances each router's activation named as that
   activation's entry and resume the run walking those names."

   Justification: wrong against its own parents. ``FEAT_ROUTE_RECORDED`` asks
   that the names be held; ``STKH_ROUTING`` above it has a router make no
   context, so there is no output for them to be written beside
   (``DEC_ROUTE_RECORDED_ALONE``).

   Verdicts on the impact analysis (``EVD_IMPACT_RECORD_HOLDS_ROUTES``):

   - Up: unchanged.
   - Down: ``IMPL_RECORD_ROUTES`` changes, and the record's version with it;
     ``TEST_RECORD_ROUTES_KEPT`` reads a router's entry without an output, and
     its run is taken again.
   - Sideways, the run record's requirements: unaffected.
   - Text: the marker.

.. dec:: Restated: the run refuses an output for a router
   :id: DEC_CHANGE_RUN_REFUSES_BAD_ROUTE
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_RUN_REFUSES_BAD_ROUTE
   :statement: Agconflo's requirements project shall state CREQ_RUN_REFUSES_BAD_ROUTE as refusing an output reported for a router and derive it from FEAT_ROUTER_MAKES_NO_CONTEXT too because a router makes no context.

   Old: "If the caller reports a router's output without naming instances or
   names an instance no edge from it enters or names instances for a node that
   does not route, then Workflow run shall refuse that output." New: "If the
   caller reports an output for a router's activation or a route naming an
   instance no edge from the router enters or a route for a node that does
   not route, then Workflow run shall refuse that report." Its parents gain
   ``FEAT_ROUTER_MAKES_NO_CONTEXT``.

   Justification: wrong against its own parents, for the reason
   ``DEC_CHANGE_HOST_ROUTE_NAMED`` gives. Refusing a router's output reported
   without names took a router's output as something to accept.

   Verdicts on the impact analysis (``EVD_IMPACT_RUN_REFUSES_BAD_ROUTE``):

   - Up: unchanged, and the new feature added.
   - Down: ``IMPL_RUN_ROUTED`` changes; ``TEST_RUN_BAD_ROUTE_REFUSED`` gains an
     output reported for a router, and its run is taken again.
   - Sideways, the workflow run's requirements: unaffected but those whose
     records sit beside this.
   - Text: the marker; the body of ``CREQ_RUN_REFUSES_UNDECLARED_BRANCH``;
     one line of an earlier record, unchanged as history; one line of
     ``features/person``, unchanged.

.. dec:: Restated: a branch's refusal speaks of a router's route
   :id: DEC_CHANGE_RUN_REFUSES_UNDECLARED_BRANCH
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_RUN_REFUSES_UNDECLARED_BRANCH
   :statement: Agconflo's requirements project shall state CREQ_RUN_REFUSES_UNDECLARED_BRANCH of a router's route because a router has no output to refuse.

   Old: "If the caller reports a router's output naming instances that are not
   those of one branch its instance declares, then Workflow run shall refuse
   that output naming the branches declared." New: "If the caller reports a
   router's route naming instances that are not those of one branch its
   instance declares, then Workflow run shall refuse that route naming the
   branches declared."

   Justification: wrong against its own parents, ``STKH_ROUTING`` among them,
   in its wording only: what it refuses is unchanged.

   Verdicts on the impact analysis
   (``EVD_IMPACT_RUN_REFUSES_UNDECLARED_BRANCH``):

   - Up: unchanged.
   - Down: ``IMPL_RUN_ROUTE_IS_A_BRANCH`` unchanged;
     ``TEST_RUN_ROUTE_NOT_A_BRANCH_REFUSED`` reports routes without outputs,
     and its run is taken again.
   - Sideways: unaffected.
   - Text: the marker, and one line of an earlier record, unchanged as
     history.

.. dec:: Restated: a router's edges carry its inputs alone
   :id: DEC_CHANGE_RUN_WALKS_ROUTED
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_IMPACT_RUN_WALKS_ROUTED
   :statement: Agconflo's requirements project shall state CREQ_RUN_WALKS_ROUTED as walking a router's inputs alone and derive it from FEAT_ROUTER_MAKES_NO_CONTEXT too because a router makes no context.

   Old: "When the caller reports a router's output with the instances named,
   Workflow run shall walk along each edge out of the router into those
   instances its output or the input the edge names and along no other edge."
   New: "When the caller reports a router's route, Workflow run shall walk
   along each edge out of the router into an instance named the input the
   edge names and along no other edge." Its parents gain
   ``FEAT_ROUTER_MAKES_NO_CONTEXT``.

   Justification: wrong against its own parents, for the reason
   ``DEC_CHANGE_HOST_ROUTE_NAMED`` gives: an edge carrying the router's
   output carried a context the router made.

   Verdicts on the impact analysis (``EVD_IMPACT_RUN_WALKS_ROUTED``):

   - Up: unchanged, and the new feature added.
   - Down: ``IMPL_RUN_ROUTED`` and ``IMPL_RUN_WALKS_EVERY_EDGE`` change, a
     router's passes counted by its routes where they were counted by its
     outputs; ``TEST_RUN_ROUTED_WALKS_CHOSEN`` walks inputs alone, and its run
     is taken again.
   - Sideways, the workflow run's requirements: unaffected but those whose
     records sit beside this.
   - Text: the two markers; one line of ``features/person`` and one of
     ``tests/repetition``, unchanged.

.. dec:: Restated: a resumed run performs again no activation its record holds as performed
   :id: DEC_CHANGE_RESUME_REPEATS_NO_OUTPUT
   :dec_status: accepted
   :decided_on: 2026-10-07
   :supported_by: EVD_IMPACT_RESUME_REPEATS_NO_OUTPUT
   :statement: Agconflo's requirements project shall state FEAT_RESUME_REPEATS_NO_OUTPUT of every activation a record holds as performed because its parent keeps all of a run's work and not only the work that made an output.

   Old: "When a run is resumed from its record, Agconflo shall continue it
   without performing again any activation whose output the record holds."
   New: "When a run is resumed from its record, Agconflo shall continue it
   without performing again any activation the record holds as performed."

   Justification: wrong against its own parent, claiming less than it.
   ``STKH_RESUMABLE_RUN`` resumes the run that was interrupted, and the
   feature's own body says what that rules out: work performed a second
   time. "Whose output the record holds" named the one mark of finished work
   there was, the mechanism of its day. An activation that finishes having
   given no output - a router's, whose report is its route
   (``DEC_ROUTER_PASSES_ON_ITS_INPUTS``) - is work all the same, and
   performing it again asks its model or its person again. The new statement
   names the property, as the feature's title always did.

   Verdicts on the impact analysis (``EVD_IMPACT_RESUME_REPEATS_NO_OUTPUT``):

   - Up, ``STKH_RESUMABLE_RUN``: unchanged.
   - Down: ``CREQ_RECORD_CONTINUES_THE_RUN`` changes under its own record;
     ``ARCH_RESUME`` unaffected; ``IMPL_RECORD_RESUME`` unchanged, handing
     the run a router's recorded route as it hands an output; the five test
     cases unaffected, each resuming activations that gave an output.
   - Sideways, the run record's requirements and ``ARCH_RESUME``'s four
     other features: unaffected but those whose records sit beside this.
   - Text: the openings of ``features/runner`` and ``features/tools`` and
     the body of a ``components/yield`` requirement cite it for steps that
     give an output, unchanged; the body of ``FEAT_RESUME_REPEATS_NO_ANSWER``
     restates the old statement and is reworded.

.. dec:: Restated: a resumed run accepts every report its record holds
   :id: DEC_CHANGE_RECORD_CONTINUES_THE_RUN
   :dec_status: accepted
   :decided_on: 2026-10-07
   :supported_by: EVD_IMPACT_RECORD_CONTINUES_THE_RUN
   :statement: Agconflo's requirements project shall state CREQ_RECORD_CONTINUES_THE_RUN of every report its record holds because its parent now keeps every activation the record holds as performed.

   Old: "Run record shall resume a run holding every output its record holds,
   having spent the activations its record spent, and offering again the
   activation that was outstanding when it was recorded." New: "Run record
   shall resume a run having accepted every report its record holds, having
   spent the activations its record spent, and offering again the activation
   that was outstanding when it was recorded."

   Justification: its parent changed (``DEC_CHANGE_RESUME_REPEATS_NO_OUTPUT``).
   A report is what the run accepted for an activation performed, an output
   or a route (``CREQ_RECORD_HOLDS_THE_RUN``), and accepting it from the
   record is what keeps that activation from being performed again.

   Verdicts on the impact analysis (``EVD_IMPACT_RECORD_CONTINUES_THE_RUN``):

   - Up: changed above; ``STKH_RESUMABLE_RUN`` unchanged.
   - Down: ``IMPL_RECORD_RESUME`` unchanged; its three test cases unaffected.
   - Sideways, the run record's requirements: unaffected but those whose
     records sit beside this.
   - Text: the marker; two lines of ``decisions/routing``, each citing it for
     a resumed run running no script again, unchanged; the body of
     ``FEAT_ROUTE_RECORDED``, which says a router's names are not in its
     output, reworded.

.. dec:: Restated: the record holds each report the run accepted
   :id: DEC_CHANGE_RECORD_HOLDS_THE_RUN
   :dec_status: accepted
   :decided_on: 2026-10-07
   :supported_by: EVD_IMPACT_RECORD_HOLDS_THE_RUN
   :statement: Agconflo's requirements project shall state CREQ_RECORD_HOLDS_THE_RUN of each report a run accepted because its parent asks for a record of the run and an activation's report is not always an output.

   Old: "Run record shall write a run as text holding its budget, the
   activations it spent, its arguments, each output it accepted in the order
   accepted with the inputs its activation was given, and every context those
   hold once each." New: "Run record shall write a run as text holding its
   budget, the activations it spent, its arguments, each report it accepted
   in the order accepted with the inputs its activation was given, and every
   context those hold once each."

   Justification: wrong against its own parent, claiming less than it.
   ``FEAT_RUN_RECORDED`` asks for a record of the run, and
   ``EVD_RUN_STATE_DERIVABLE`` measured the outputs as the whole of a run's
   state when every activation gave one. An activation that gives a route in
   place of an output (``DEC_ROUTE_RECORDED_ALONE``) is part of that state
   too: held without its inputs or its place among the rest, the record
   describes no run.

   Verdicts on the impact analysis (``EVD_IMPACT_RECORD_HOLDS_THE_RUN``):

   - Up, ``FEAT_RUN_RECORDED`` and ``STKH_RESUMABLE_RUN``: unchanged.
   - Down: ``IMPL_RECORD_WRITE`` and ``IMPL_RECORD_EVENTS`` unchanged, each
     writing a router's entry with its inputs in its place;
     ``TEST_RECORD_HOLDS_THE_RUN`` unaffected.
   - Sideways, the run record's requirements: unaffected but those whose
     records sit beside this.
   - Text: the two markers; seven lines of ``scripts/comment-rules.sh``, which
     plant its id only as a well-formed marker, unaffected.

.. dec:: Restated: a record diverging at any report is refused
   :id: DEC_CHANGE_RECORD_REFUSES_DIVERGENCE
   :dec_status: accepted
   :decided_on: 2026-10-07
   :supported_by: EVD_IMPACT_RECORD_REFUSES_DIVERGENCE
   :statement: Agconflo's requirements project shall state CREQ_RECORD_REFUSES_DIVERGENCE of each recorded report because its parent refuses any record that describes no run of its workflow.

   Old: "If the workflow a record is resumed against would not have offered
   each recorded output's activation in the recorded order with the recorded
   inputs, then Run record shall refuse to resume it naming the first
   recorded output that differs." New: "If the workflow a record is resumed
   against would not have offered each recorded report's activation in the
   recorded order with the recorded inputs, then Run record shall refuse to
   resume it naming the first recorded report that differs."

   Justification: wrong against its own parent, claiming less than it.
   ``FEAT_RESUME_REFUSES_ANOTHER_RUN`` refuses a record that does not
   describe a run of the workflow. One whose router's entry was given inputs
   the workflow would not give it is such a record, and "each recorded
   output's activation" does not reach it.

   Verdicts on the impact analysis (``EVD_IMPACT_RECORD_REFUSES_DIVERGENCE``):

   - Up: unchanged.
   - Down: ``IMPL_RECORD_RESUME`` changes: it already compared a router's
     entry, and now counts it among the reports the activations spent are
     held to, and its refusals name a report where they named an output;
     ``TEST_RECORD_DIVERGED_RECORD_REFUSED`` unaffected, and
     ``TEST_RECORD_ROUTES_KEPT`` gains a spent count below the reports of a
     record holding a route, and verifies this as well.
   - Sideways, the run record's requirements: unaffected but those whose
     records sit beside this.
   - Text: the marker; the body of ``CREQ_RECORD_HOLDS_ROUTES``, citing it for
     a route the workflow no longer has, unchanged; one line of
     ``components/yield``, for a call's output, unchanged.

.. dec:: Restated: an untyped model answer where no output is declared fails
   :id: DEC_CHANGE_HOST_MODEL_ANSWER
   :dec_status: accepted
   :decided_on: 2026-10-07
   :supported_by: EVD_IMPACT_HOST_MODEL_ANSWER
   :statement: Agconflo's requirements project shall state CREQ_HOST_MODEL_ANSWER as failing a model call that names no type for a node type declaring no output because as written it asks for a type that does not exist.

   Old: "Script host shall give a script a model's answer as a new context of
   the type the script names for it, or of its output's declared type when it
   names none." New: "Script host shall give a script a model's answer as a
   new context of the type the script names for it, or of its output's
   declared type when it names none, failing the call where it names none and
   its node type declares none."

   Justification: wrong against its own parent, in that it cannot be
   verified as written. ``FEAT_MODEL_ANSWER_IS_A_CONTEXT`` asks for the
   answer as a new context, which needs a type; the statement took the
   declared output's wherever the script named none, and a node type may
   declare none (``DEC_ROUTER_DECLARES_NO_OUTPUT``). Failing the call, before
   any request, is the only answer that neither chooses a type for the
   script nor pays for an answer nothing can hold.

   Verdicts on the impact analysis (``EVD_IMPACT_HOST_MODEL_ANSWER``):

   - Up, ``FEAT_MODEL_ANSWER_IS_A_CONTEXT`` and ``STKH_PROVENANCE``:
     unchanged.
   - Down: ``IMPL_HOST_COMPLETE`` changes, failing such a call before any
     request; ``TEST_HOST_MODEL_ANSWER_IS_A_CONTEXT`` unaffected, and a new
     case, ``TEST_HOST_UNTYPED_ANSWER_IN_ROUTER_FAILS``, verifies the
     failure.
   - Sideways, the script host's requirements: unaffected but those whose
     records sit beside this.
   - Text: the marker.

.. dec:: Restated: a stuck run names the instances that never activated
   :id: DEC_CHANGE_RUN_ENDS_QUIESCENT
   :dec_status: accepted
   :decided_on: 2026-10-07
   :supported_by: EVD_IMPACT_RUN_ENDS_QUIESCENT
   :statement: Agconflo's requirements project shall state CREQ_RUN_ENDS_QUIESCENT as naming every instance that never activated because naming one that did its work without an output points at the wrong part of the workflow.

   Old: "If no instance of a run may activate and its designated instance has
   produced no output, then Workflow run shall end that run naming every
   instance that produced none." New: "If no instance of a run may activate
   and its designated instance has produced no output, then Workflow run
   shall end that run naming every instance that never activated."

   Justification: wrong against its own parents. ``FEAT_RUN_QUIESCENCE_ENDS``
   and ``STKH_STUCK_RUN`` above it report a run in which no node can make
   further progress, and the requirement's own failure modes say what the
   names are for: the instances that did no work, and not those that did.
   "Produced none" took giving an output for doing work; an instance whose
   activations gave none - a router's, whose report is its route
   (``DEC_ROUTER_PASSES_ON_ITS_INPUTS``) - did its work and would be named.

   Verdicts on the impact analysis (``EVD_IMPACT_RUN_ENDS_QUIESCENT``):

   - Up: unchanged.
   - Down: ``IMPL_RUN_STEP`` unchanged, already naming an instance by whether
     it ran; ``TEST_RUN_QUIESCENT_NAMES_ONLY_UNPRODUCED``, whose router names
     nothing and is not named, unchanged but for its body, which says so;
     ``TEST_RUN_IDLE_INSTANCE_DOES_NOT_MAKE_IT_STUCK`` unaffected.
   - Sideways, the workflow run's requirements: unaffected but those whose
     records sit beside this.
   - Text: the marker; one line each of ``components/run`` and
     ``features/behaviour``, citing it for quiescence, unchanged.
