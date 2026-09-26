===============================
Components of a person in a run
===============================

The requirements ``ARCH_PERSON`` allocates to the behaviour set and the script
host, both defined in ``components/behaviour``. No component is added. Each
title is the grammatical subject of the requirements allocated to it, and the
gate in ``scripts/gates`` refuses a component requirement whose subject is
anything else.

Two requirements of ``components/behaviour`` are narrowed in the same change,
since each said something of every node type or every activation that is now
true only of those given a script: the refusal of a node type with no script,
and how an activation is performed.

.. comp_req:: A node type a person performs has no script
   :id: CREQ_BEHAVIOURS_PERSON_OR_SCRIPT
   :derived_from: FEAT_PERSON_STEP_HANDED_OVER
   :allocated_to: COMP_BEHAVIOUR_SET
   :ears_pattern: unwanted
   :statement: If a node type named by an instance of a workflow is both named as performed by a person and given a script, then Behaviour set shall refuse to start the run naming that node type and each document supplying a script for it.

   Two answers to what performs one node type, which is the fault two scripts
   are (``CREQ_BEHAVIOURS_REFUSE_TWICE``) and gets the same answer for the same
   reason: running either would be a choice nobody made.

   Failure modes:

   - **The script run.** The person is never asked, and the caller that named
     the type never learns why.
   - **The person asked.** A script the caller supplied is never run, and
     nothing says so.
   - **Refused for a type no instance names.** A caller sharing one set of
     behaviours across workflows is refused for a type this run never
     activates, as a missing script would not be.

   Must pass unreported: a type named twice as performed by a person, which is
   one answer given twice rather than two answers; and a type so named that no
   instance names.

.. comp_req:: A person's step is handed to the caller, with nothing run for it
   :id: CREQ_HOST_HANDS_OVER_PERSON_STEP
   :derived_from: FEAT_PERSON_STEP_HANDED_OVER
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When the run offers an activation of a node type named as performed by a person, Script host shall return that activation to the scripted run's caller having run no script and reported nothing to the run for it.

   Returned rather than awaited (``DEC_SCRIPTED_RUN_RETURNS_TO_AWAIT``), so the
   call driving the run ends, and the record the caller was handed last is what
   the answer is later given with.

   Failure modes:

   - **An activation other than the one offered.** Inputs gathered afresh, or
     the instance's inputs from the definition rather than from the run: the
     person answers a question the run did not ask.
   - **A failure reported for it.** The run ends on a node's failure where it
     should wait, and a person's step reads as a node that broke.
   - **The run asked for its next step and told it ended.** A host that reports
     whatever the run says once the activation is set aside reports
     quiescence on a run that is only waiting (``FEAT_PERSON_WAIT_NOT_STUCK``).
   - **A panic.** The loop looks the type up among the scripts, finds none,
     and takes the check before the run to have ruled that out.

.. comp_req:: A person's text is the output of the step it was supplied for
   :id: CREQ_HOST_TAKES_PERSON_TEXT
   :derived_from: FEAT_PERSON_TEXT_IS_THE_OUTPUT
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a person's text is supplied for the activation a record's run awaits, Script host shall report as that activation's output a context of its declared type holding exactly the text from the source resumed with the record.

   The run then goes on as any scripted run does, handing its caller a record
   once the output is accepted (``CREQ_HOST_HANDS_RECORDS``) and returning at
   the next person's step or at its ending.

   Failure modes:

   - **A context from a fresh source.** Its identifier is one the run holds,
     and the run refuses it (``CREQ_RUN_REFUSES_HELD_IDENTIFIER``).
   - **The text changed on the way.** Trimmed, or its line endings converted:
     the run holds bytes the person did not write.
   - **An exhausted source met with a panic, or the answer dropped.** A record
     can carry a source that has issued every identifier, and the text then
     has none to be issued under. The step fails carrying that, as a script's
     does when the functions it calls cannot issue one.
   - **A record not handed over after it.** An interruption after the person
     answered and before the next output loses the answer.

.. comp_req:: An answer for a step the run does not await is refused, with nothing run
   :id: CREQ_HOST_REFUSES_ANSWER_ELSEWHERE
   :derived_from: FEAT_PERSON_ANSWER_ELSEWHERE_REFUSED
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a person's text is supplied for an instance whose activation a record's run does not await, then Script host shall refuse it naming that instance and any instance the run offers next, having run nothing.

   Asked after everything a start or a resume would be refused for, so a
   record that is not a record of this workflow, and scripts that cannot run,
   are reported as themselves rather than as an answer in the wrong place.

   Failure modes:

   - **Taken for a script's step.** The run offers a step a script performs,
     and the person's text is recorded as that script's output.
   - **Scripts run until the instance comes up.** The record is an earlier
     one, the steps run meanwhile spend activations and model calls, and the
     person's text is taken for inputs they were never shown.
   - **Dropped for a run that has ended.** The run's ending is returned and the
     person's text goes nowhere, without a word.
   - **Refused without naming what the run awaits.** A caller holding the
     wrong record for the right person cannot tell which record to use.

.. comp_req:: A person's answer to a router's step carries its route
   :id: CREQ_HOST_TAKES_PERSON_ROUTE
   :derived_from: FEAT_PERSON_ROUTES
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a person's answer to the router's step a record's run awaits names instances, Script host shall report the answer's text as that step's output with those names as its route.

   ``DEC_PERSON_ROUTE_IN_THE_ANSWER``. The output is made as any person's
   answer is (``CREQ_HOST_TAKES_PERSON_TEXT``); the names are reported with
   it in the order given, none meaning nowhere, as a script's are
   (``CREQ_HOST_ROUTE_NAMED``).

   Failure modes:

   - **The names dropped**, and the router's output refused by the run as
     unrouted, which ends the run.
   - **The names reported in another order than given.**

.. comp_req:: A person's answer with a route that does not fit its step is refused
   :id: CREQ_HOST_REFUSES_BAD_PERSON_ROUTE
   :derived_from: FEAT_PERSON_ROUTES
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a person's answer gives no route for a router's step or gives one for any other step or names an instance no edge out of the router enters, then Script host shall refuse it having run nothing.

   ``DEC_PERSON_ROUTE_IN_THE_ANSWER``: refused as an answer for another step
   is (``CREQ_HOST_REFUSES_ANSWER_ELSEWHERE``), so the person answers again.
   A step that is not a router's own includes a call's activation of a node
   type that routes: a call's output goes back to its caller alone
   (``CREQ_RUN_CALL_OUTPUT_TO_CALLER``) and has nowhere to be walked.

   Failure modes:

   - **A missing route read as every edge**, and a branch nobody chose.
   - **A route on a step that is not a router's own ignored**, and a person
     believing they chose a branch that is not there.
   - **A misspelt name failing the run**, where the person could have
     answered again.

.. comp_req:: The runner carries a person's route with the answer
   :id: CREQ_RUNNER_ANSWERS_WITH_ROUTE
   :derived_from: FEAT_PERSON_ROUTES
   :allocated_to: COMP_RUNNER
   :ears_pattern: event
   :statement: When text and a route are given for the step of an instance, Runner shall answer the record its file holds with that text and that route for that instance.

   What ``CREQ_RUNNER_ANSWERS`` does with the text, with the names beside it.

   Failure modes:

   - **The route dropped between the person and the scripted run.**

.. comp_req:: A router's awaited step is handed back with the instances it may name
   :id: CREQ_RUNNER_TELLS_ROUTES
   :derived_from: FEAT_PERSON_ROUTES
   :allocated_to: COMP_RUNNER
   :ears_pattern: event
   :statement: When a run stops awaiting a person at a router's step, Runner shall hand back with the step every instance an edge out of that router enters.

   A person cannot name a branch they cannot see, and the names are the
   workflow's, not the activation's.

   Failure modes:

   - **Nothing handed back**, and the person reading the workflow's document
     to learn what to type.
   - **Instances handed back that no edge from the router enters**, which the
     run refuses when named.

.. comp_req:: The command line reads a person's route and shows a router's choices
   :id: CREQ_COMMAND_READS_ROUTE
   :derived_from: FEAT_PERSON_ROUTES
   :allocated_to: COMP_COMMAND_LINE
   :ears_pattern: event
   :statement: When a person answers a step with route names or a run awaits a router's step, Command line shall hand the runner those names with the answer or print the instances it may name with the step.

   ``--route`` once for each instance, and ``--route ""`` for none.

   Failure modes:

   - **An empty name taken as an instance**, and ``--route ""`` refused as a
     name no edge enters.
   - **The choices printed to standard error**, where a script reading the
     step from standard output does not see them.
