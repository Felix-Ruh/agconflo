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
