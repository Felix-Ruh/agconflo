=================
A person in a run
=================

A person performing a step of a run: handed what the step was given, answering
with text, and the run going on from its record with that text as the step's
output - in another process, hours later, if that is when the answer comes.
Every requirement here derives from a goal in ``stakeholder/execution`` or
``stakeholder/context``, and each is written against the decisions in
``decisions/person``.

What this does not do is what keeps it a slice. Nothing reaches a person: the
scripted run hands its caller the step, and how a person is shown it and how
their answer comes back are the caller's. A person answers with text and
nothing else (``DEC_PERSON_SUPPLIES_TEXT``), and cannot decline a step. And the
core is not changed at all: measured, a caller driving the core alone already
has a person perform an activation across a restart
(``EVD_CORE_PARKS_FOR_A_PERSON``). What was missing is the scripted run, which
refused any workflow with a step no script performs
(``EVD_SCRIPTED_RUN_REFUSES_A_PERSON``).

Each requirement was checked by hand against the question no rule can ask:
could this be false while its parent is true? The body of each says how. Two
statements are ``event``, one is ``unwanted`` and one is ``state``, the first
``state`` in this project: a run awaiting a person is a condition that holds for
as long as nobody answers, rather than something that happens.

Two requirements of ``features/behaviour`` took every node type to have a
script: that each activation runs one, and that a type without one is refused.
Both are narrowed to the node types the caller gave a script, in the same
change.

The feature's architecture closes the file. It realises all four requirements
and names the components they are divided between, which are defined in
``components/behaviour``; ``components/person`` holds the requirements this
feature allocates to them.

.. feat_req:: A step a person performs is handed to the caller
   :id: FEAT_PERSON_STEP_HANDED_OVER
   :derived_from: STKH_HUMAN_IN_RUN
   :ears_pattern: event
   :verification_method: test
   :statement: When a scripted run reaches an activation of a node type its caller named as performed by a person, Agconflo shall hand that activation to the run's caller without running a script for it.

   The parent lets a person supply a context while a run is in progress. The
   activation is what tells them what to supply it for: which step, what that
   step was given, and what type of context it produces.

   It can be false while the parent holds, and it is today. A caller driving the
   core itself can already have a person perform any activation
   (``EVD_CORE_PARKS_FOR_A_PERSON``), which meets the parent word for word, and
   the scripted run - the one caller in this project that runs a workflow from
   its start to its end - refuses a workflow with such a step before it starts
   (``EVD_SCRIPTED_RUN_REFUSES_A_PERSON``).

.. feat_req:: A person's text becomes the step's output
   :id: FEAT_PERSON_TEXT_IS_THE_OUTPUT
   :derived_from: STKH_HUMAN_IN_RUN
   :ears_pattern: event
   :verification_method: test
   :statement: When a person's text is supplied for the activation a scripted run's record awaits, Agconflo shall continue that run from the record with a context holding that text as the activation's output.

   The parent's context, supplied into the run rather than beside it. From the
   record, because a person's answer arrives on no schedule the run keeps, and
   the process that handed the step over may have gone
   (``DEC_SCRIPTED_RUN_RETURNS_TO_AWAIT``).

   It can be false while the parent holds. A person's text taken in as the
   argument of a second run started for the rest of the workflow is a context
   a person supplied, and the two runs' lineages then meet nowhere - the
   fragmented provenance chain the parent's own body refuses. So can a person's
   text given to the process that handed the step over and to no other, which
   holds only for as long as that process does.

.. feat_req:: An answer for a step the run does not await is refused
   :id: FEAT_PERSON_ANSWER_ELSEWHERE_REFUSED
   :derived_from: STKH_PROVENANCE
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a person's text is supplied for an instance whose activation a run's record does not await, then Agconflo shall refuse it without continuing the run.

   The parent records which context each byte of a node's input came from, and
   the record says so by naming the instance that produced each context. A
   person's text recorded as another instance's output credits their bytes to a
   node that did not produce them.

   It can be false while the parent holds. A run that takes the text as the
   output of whatever activation it offers next records it faithfully, as that
   activation's output - of a script's step, perhaps, or of a person's step
   further on whose question this person never saw - and every lineage through
   it is recorded exactly and is false.

.. feat_req:: A run awaiting a person is not a stuck run
   :id: FEAT_PERSON_WAIT_NOT_STUCK
   :derived_from: STKH_STUCK_RUN
   :ears_pattern: state
   :verification_method: test
   :statement: While a scripted run awaits a person, Agconflo shall report it as awaiting that person's step rather than as a run in which no node can make further progress.

   The parent's own caveat, which it says decides whether it is correct at all:
   a run parked on a person is live, and the two stakeholder requirements have
   to agree about that.

   It can be false while the parent holds. A run that reports every stuck run,
   and reports a run awaiting a person as stuck as well, has reported each run
   the parent names, and one more. The one more is a run whose caller, told it
   can make no further progress, throws it away.

.. feat_arch:: A person in a run splits into the behaviour set and the script host
   :id: ARCH_PERSON
   :realises: FEAT_PERSON_STEP_HANDED_OVER, FEAT_PERSON_TEXT_IS_THE_OUTPUT, FEAT_PERSON_ANSWER_ELSEWHERE_REFUSED, FEAT_PERSON_WAIT_NOT_STUCK
   :uses: COMP_BEHAVIOUR_SET, COMP_SCRIPT_HOST
   :statement: Agconflo shall allocate a person's part in a run to the behaviour set and the script host.

   The two components ``ARCH_BEHAVIOUR`` named, each answerable for the same
   kind of thing it was there:

   - The behaviour set answers for what performs each node type a run
     instantiates, before the run starts. A person is one more answer to that
     question (``DEC_PERSON_NAMED_BY_CALLER``), and a node type given two is the
     fault it already refuses.
   - The script host answers for the loop performing activations: handing a
     person's step to the caller instead of running a script, and taking the
     person's text back into a resumed run.

   The workflow run and the run record are not among them. Measured, a run
   already waits for an activation's output for as long as nobody reports one,
   and a record already resumes a run offering again the activation it was
   waiting on (``EVD_CORE_PARKS_FOR_A_PERSON``), so every requirement of
   ``components/run`` and ``components/resume`` holds of a run a person takes
   part in, and none is added.

   ``FEAT_PERSON_WAIT_NOT_STUCK`` needs nothing of its own. The run reports
   quiescence only when no instance can activate, and a person's step is an
   instance that could: it is the one the run is offering. What the requirement
   rules out is a host that, handed that activation and unable to perform it,
   reports anything but the activation.

   The decisions this is built against are named here rather than linked:

   - ``DEC_PERSON_PERFORMS_AN_ACTIVATION``: a person performs a whole
     activation, never a call inside a script.
   - ``DEC_PERSON_NAMED_BY_CALLER``: which node types a person performs is given
     with the scripts.
   - ``DEC_SCRIPTED_RUN_RETURNS_TO_AWAIT``: the scripted run returns the
     activation rather than awaiting the caller.
   - ``DEC_PERSON_SUPPLIES_TEXT``: the person's output is text, made into a
     context of the declared type from the resumed source.
   - ``DEC_PERSON_HOLDS_THE_RUN``: nothing else is offered while a person's
     step is outstanding.
   - ``DEC_ANSWER_ONCE_BY_THE_KEEPER``: answering a record once is the caller's.
