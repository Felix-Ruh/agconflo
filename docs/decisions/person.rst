=================================
Decisions about a person in a run
=================================

How a person takes part in a run: what they perform, how the engine is told
which steps are theirs, how a run waits for them, and what they hand back.
``DEC_RUN_IS_DRIVEN`` argued that a run a person takes part in would need no
redesign of the run, and ``DEC_RUN_ENDS_ONE_WAY`` that a run awaiting a person
is none of its four endings; these are the choices about the caller that makes
both true.

Four rest on measurements recorded in ``evidence/person``. Two are judgements
and say so: where a person's steps are named, and that a scripted run returns
to its caller to wait.

What they do not settle is named here. Nothing below decides how a person is
shown what they are asked, how they are reached, who they are, or whether a
person may decline a step rather than answer it - a caller that will not
continue a run simply does not, and a run nobody continues has not ended.
Nothing decides that a person's step counts differently against the budget:
it is an activation, and ``DEC_BUDGET_COUNTS_ACTIVATIONS`` counts those.

.. dec:: A person performs a whole activation
   :id: DEC_PERSON_PERFORMS_AN_ACTIVATION
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS, EVD_CORE_PARKS_FOR_A_PERSON
   :statement: Agconflo shall have a person take part in a run by performing an activation whole rather than by answering a call made from inside a script.

   A record is taken between activations, so an activation is the unit a run
   survives an interruption in. Performed whole by a person, the step waiting on
   them is the run's outstanding activation, which a record already re-derives
   and a resumed run already offers again (``EVD_CORE_PARKS_FOR_A_PERSON``).

   A function a script calls to ask a person, in the shape of the one it calls
   to ask a model, was the alternative, and it lost on a measurement. Work done
   inside an interrupted activation was done again when the run resumed
   (``EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS``), and a wait of hours is where an
   interruption is likeliest. Everything the script had done before asking
   would be done again, and the answer would arrive for an activation that
   was dropped with the process.

   Taking a person's context in as an argument of a second run was the other,
   and ``STKH_HUMAN_IN_RUN`` rules it out in its own body: splitting a workflow at
   every human touchpoint fragments the provenance chain.

   The cost is accepted: a step that needs a model's draft and a person's
   correction of it is two node instances, not one script.

.. dec:: A person's steps are named by the caller
   :id: DEC_PERSON_NAMED_BY_CALLER
   :dec_status: accepted
   :decided_on: 2026-09-24
   :statement: Agconflo shall take which node types a person performs from the caller starting a scripted run, beside the scripts it supplies, rather than from a node type's declaration.

   A judgement, and two decisions already taken make it. What performs a node
   type is the caller's to supply (``DEC_SCRIPTS_FROM_CALLER``), and a node type
   document holds declarations that are validated without running anything
   (``STKH_TOPOLOGY_AS_DATA``). A person is one more answer to "what performs
   this type", so it goes where the other answer goes.

   A node type Agconflo ships for people to perform - a built-in review step -
   was the alternative, and ``STKH_NO_PRIVILEGED_TYPES`` refuses it: it would be
   a type with a capability no user-defined type has. Named by the caller, any
   node type can be performed by a person in one run and by a script in the
   next, and the workflow is the same workflow in both.

   The engine cannot tell whether whoever answers is a person, and does not
   try. A step named this way is one the caller performs; that the caller
   hands it to a person is the caller's business.

.. dec:: A scripted run returns to its caller to await a person
   :id: DEC_SCRIPTED_RUN_RETURNS_TO_AWAIT
   :dec_status: accepted
   :decided_on: 2026-09-24
   :statement: Agconflo shall hand a scripted run's caller an activation a person performs by returning from the call that drives the run rather than by awaiting a function the caller supplies.

   A judgement, and it is ``DEC_RUN_IS_DRIVEN`` applied one level up: the run
   hands its caller an activation and waits for nothing, and a scripted run does
   the same with the activations no script performs. The caller answers from
   the run's record, whenever and wherever the answer arrives.

   A function the caller supplies, awaited for each such activation, was the
   alternative. It holds a process, and a pending future inside it, for as
   long as a person takes - hours or days by ``STKH_RESUMABLE_RUN``'s own
   reckoning - and a caller serving requests, where the person answers in a
   later request than the one that started the run, would have to keep that
   future alive between them.

   Either can be built from the other, and only one direction is clean. A
   caller wanting to wait in place calls again with the answer in a loop. A
   caller wanting to return from an awaited function has to drop the run's
   future to do it, which is an interruption, and then find out from the
   record what was being asked.

.. dec:: A person's output is text the scripted run makes into a context
   :id: DEC_PERSON_SUPPLIES_TEXT
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_RUN_ARGUMENTS_SHARE_IDENTIFIER
   :statement: Agconflo shall take a person's output as text and make it into a context of the activation's declared output type from the identifier source resumed with the run.

   The source a context must come from is the one that comes back with the
   resumed run, and it comes back inside the call that resumes it - after the
   caller has handed over the answer. A context the caller made itself would
   come from some other source, and two sources were measured repeating each
   other's identifiers (``EVD_RUN_ARGUMENTS_SHARE_IDENTIFIER``). The run would
   refuse it, as it refuses such an output from a script
   (``TEST_SCRIPTED_ARGUMENTS_FROM_ANOTHER_SOURCE_FAIL``).

   Taking a context the caller makes, through a function handed the resumed
   source, was the alternative. It would let a person compose, passing on a
   context they were given with a note of their own. Nothing asks for that yet,
   and the wiring already does it: a node that needs both the draft and the
   person's verdict binds both, and receives the draft as the context it was.

   The type is the one the activation declares, so it cannot be wrong, and the
   text is kept exactly as supplied.

.. dec:: A person's step holds the run while it waits
   :id: DEC_PERSON_HOLDS_THE_RUN
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_PARKED_ACTIVATION_HOLDS_OTHER_BRANCHES, EVD_RUN_STATE_DERIVABLE
   :statement: Agconflo shall keep a run to its one outstanding activation while a person performs it rather than offering another instance meanwhile.

   A boundary rather than an ambition, in the shape of
   ``DEC_ACTIVATION_ONCE_PER_RUN``. Measured, a run with one activation
   outstanding offers no other (``EVD_PARKED_ACTIVATION_HOLDS_OTHER_BRANCHES``),
   so an instance on another branch that a script could run at once waits for
   the person.

   Several activations outstanding at once was the alternative, and it is a
   redesign of the run rather than an addition to it. The run hands out one
   activation at a time (``DEC_RUN_IS_DRIVEN``), and a record rests on there
   being at most one outstanding: the activations spent are the outputs, or
   one more (``EVD_RUN_STATE_DERIVABLE``). Both would have to change, and the
   resume with them, for a gain in when a run ends rather than in what it ends
   with.

   The cost is time, and it is accepted until it is measured mattering. Which
   ready instance is offered first follows the order a definition is written
   in, so of two instances ready at once, one written before a person's step
   runs before it and one written after waits for the person.

.. dec:: A record is answered once by whoever keeps it
   :id: DEC_ANSWER_ONCE_BY_THE_KEEPER
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_ONE_RECORD_ANSWERED_TWICE
   :statement: Agconflo shall leave answering a record once to the caller that keeps it rather than detect a second answer to the same record.

   Measured, one record answered twice gave two runs, each sound, that shared
   an identifier (``EVD_ONE_RECORD_ANSWERED_TWICE``). Nothing in either run could
   see the other.

   Detecting it was the alternative, and it needs somewhere to remember that a
   record was answered, which is storage the core has decided not to have
   (``DEC_RECORD_IN_CORE``). The caller already keeps the records, and replacing
   the one it answered with the one that comes back is what keeping them means;
   ``IdSource`` leaves preventing two sources to whatever owns a run on the
   same ground.

   Resuming one record twice was already possible before a person took part,
   and gives the same two runs. What a person adds is a reason to do it by
   accident: a second person answering a question already answered.

.. dec:: A person routes a router's step by naming instances with the answer
   :id: DEC_PERSON_ROUTE_IN_THE_ANSWER
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall take the instances a person's answer to a router's step names as that router's route, given beside the answer's text, and refuse an answer that names none for a router's step or names some for any other.

   A router's output is its decision and the instances it names are where
   the run goes (``DEC_ROUTER_OUTPUT_IS_ITS_DECISION``); a person performing
   the router gives both, as a script does through ``host.route``. The text
   stays the output, so what ``DEC_PERSON_SUPPLIES_TEXT`` decided holds, and
   the names travel beside it: on the command line, ``--route`` once for each
   instance, and ``--route ""`` for none.

   Reading the route out of the text was the alternative, and is the branch
   read out of prose that ``FEAT_ROUTE_CHOSEN_BY_MODEL`` exists to end. A
   menu to choose from was the other, and makes an answer something a script
   cannot give. A name the run refuses - one no edge out of the router enters
   - refuses the answer rather than failing the run, as an answer for a step
   the run does not await is refused: a typo costs an answer, not the run.
