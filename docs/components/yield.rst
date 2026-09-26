==============================
Components of a model yielding
==============================

The requirements ``ARCH_YIELD`` allocates to the wiring validator, the workflow
run, the run record, the script host and the model roster, and three that
earlier features need of calls: reading an instance's calls
(``FEAT_TOPOLOGY_READS``), refusing a second context under a held identifier
from a call or an exchange (``FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER``), and
refusing a record whose calls the workflow would not accept
(``FEAT_RESUME_REFUSES_ANOTHER_RUN``). Each is a new child of its feature, which
changes nothing the feature says. No component is added. Each title is the grammatical subject of the
requirements allocated to it, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

The core knows a call and an exchange without knowing a model. An exchange is
what an activation's performer reports having sent out and got back - the
offer and window, the answer, the calls the answer made - and a call is an
activation of a declared node type for the instance that made it. The script
host is what turns a model's answer into both.

What the budget, the declared output type, a missing behaviour, a person, the
model call limit and the resume already require of every activation holds of
a called node type's as written, and gains test cases in ``tests/yield``
rather than requirements here.

Reading
=======

.. comp_req:: An instance's calls are read in the order written
   :id: CREQ_READER_CALLS
   :derived_from: FEAT_TOPOLOGY_READS
   :allocated_to: COMP_TOPOLOGY_READER
   :ears_pattern: ubiquitous
   :statement: Topology reader shall read the node types an instance's calls key names into that instance, in the order the key lists them.

   The calls are part of the workflow definition
   (``DEC_CALLS_DECLARED_ON_THE_INSTANCE``), so reading a workflow reads them.
   Measured, the reader kept the key without reading it
   (``EVD_READER_KEEPS_UNKNOWN_CALLS_KEY``).

   Failure modes:

   - **The key left unread.** The measured state: the workflow reads, and no
     model is ever offered a tool.
   - **A calls key that is not a list of strings read as no calls.** A typo of
     shape declares nothing without a word; it is a fault, located like any
     other (``CREQ_READER_FAULT_LOCATED``).
   - **The names resolved while reading.** A call to a node type no document
     declares is a wiring defect to report with the others
     (``CREQ_READER_NAMES_UNRESOLVED``), not a reason to refuse the document.
   - **A calls key on a node type document read as calls.** Calls belong to
     the instance; on a node type the key is one the reader does not name, and
     is kept as such.

   Must pass unreported: an instance with no calls key, which declares none;
   and one naming the same node type twice, which is read as written.

Checking the wiring
===================

.. comp_req:: A call to a node type the definition lacks is a defect
   :id: CREQ_VALIDATOR_CALL_RESOLVES
   :derived_from: FEAT_WIRING_CALL_RESOLVES
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If an instance declares a call to a node type the definition does not carry, then Wiring validator shall report a defect naming that instance and that node type.

   A wiring defect like the others, so the run refuses it with them
   (``CREQ_RUN_REFUSES_DEFECTS``) and every one is reported
   (``CREQ_VALIDATOR_EVERY_DEFECT``).

   Failure modes:

   - **Not checked.** The measured state for any key the reader does not read,
     and a model offered a node type that cannot be performed.
   - **Reported once for the workflow.** An instance with two bad calls, or two
     instances with one each, is fixed one round trip at a time.
   - **The instance's own node type checked instead.** That is
     ``CREQ_VALIDATOR_BINDING_RESOLVES``'s defect, and a call to a missing node
     type goes unreported beside it.

.. comp_req:: A call to a node type whose name a provider refuses is a defect
   :id: CREQ_VALIDATOR_CALL_NAME
   :derived_from: FEAT_WIRING_CALL_NAME_PORTABLE
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If an instance declares a call to a node type whose name is not one to sixty-four characters that are each an ASCII letter or digit or an underscore or hyphen, then Wiring validator shall report a defect naming that instance and that name.

   That is ``^[a-zA-Z0-9_-]{1,64}$``, the two providers' documented rules taken
   together (``DEC_TOOL_NAMES_PORTABLE``), written out in words because the
   pattern's comma cannot stand in the trigger of an ``unwanted`` statement.

   Failure modes:

   - **Checked against one provider's rule.** Anthropic's allows 128
     characters, and a name of 100 passes and is refused by OpenAI.
   - **Checked with a search rather than a whole match.** A name holding one
     legal character passes, a space and a dot beside it.
   - **Every node type's name checked.** A node type no instance calls is never
     offered as a tool, and a workflow is refused for a name nothing sends.

   Must pass unreported: a name of exactly 64 characters, and one using each of
   the four kinds the pattern allows.

The run
=======

.. comp_req:: A declared call is offered as the next activation
   :id: CREQ_RUN_CALL_OFFERED
   :derived_from: FEAT_YIELD_CALL_PERFORMED
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: event
   :statement: When its caller reports a call to a node type the outstanding activation's instance declares a call to, Workflow run shall offer next an activation of that node type for that instance, given the call's contexts as its inputs.

   The call is an activation of the run (``DEC_CALL_IS_AN_ACTIVATION``), so it
   is offered the way every activation is, and counted when it is
   (``CREQ_RUN_STOPS_AT_BUDGET``). The calling activation stays outstanding
   beneath it.

   Failure modes:

   - **Another instance offered first.** The scheduler's next instance runs
     while the model waits, and its output is taken for the call's.
   - **The calling activation dropped.** Its script's own output is then
     refused as reported with nothing outstanding.
   - **Not counted.** A model calling in a loop spends nothing of the budget,
     which is the runaway ``STKH_STEP_BUDGET`` stops.
   - **Its inputs in the order the call gave them.** A node type reads its
     inputs by parameter, in the order it declares them
     (``CREQ_SCHEDULER_ACTIVATION_CARRIES``), and a model's argument order is
     its own.

.. comp_req:: A called node type's output goes back to its caller alone
   :id: CREQ_RUN_CALL_OUTPUT_TO_CALLER
   :derived_from: FEAT_YIELD_CALL_PERFORMED
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: event
   :statement: When the run accepts the output of an activation it offered for a call, Workflow run shall make the calling activation outstanding again and take that output as no instance's output.

   The model continues with the call's output; nothing else in the workflow
   receives it (``DEC_CALL_IS_AN_ACTIVATION``).

   Failure modes:

   - **Taken as the calling instance's output.** The activation is for that
     instance, and filing the output under it makes the instance produced: its
     consumers are offered the callee's output as its, the run completes on it
     if the instance is designated, and the instance's own output is refused
     later as a second.
   - **The calling activation left outstanding nowhere.** Its script's output
     is refused, and the run cannot go on.
   - **The output not held.** A later output composing it is refused for an
     identifier the run does not hold, and the record loses it.
   - **Given back to a later activation of the same instance.** An instance
     runs again when the edges into it carry something new
     (``CREQ_SCHEDULER_OFFERS_AGAIN``), and a provider may name its calls alike
     each time; the output of an earlier pass's call is not the calling
     activation's, and a resumed pass answered with it asks nothing it should.

.. comp_req:: A call its instance does not declare is refused
   :id: CREQ_RUN_REFUSES_UNDECLARED_CALL
   :derived_from: FEAT_YIELD_UNDECLARED_REFUSED
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If its caller reports a call to a node type the outstanding activation's instance does not declare a call to, then Workflow run shall refuse that call naming the instance and that node type.

   The last line: the script host checks every call before reporting any
   (``CREQ_HOST_REFUSES_MALFORMED_CALL``), and the run checks what it is told
   whoever tells it, as it does outputs.

   Failure modes:

   - **Checked against the catalogue.** Any node type the workflow knows can be
     called from anywhere.
   - **A call reported by a called node type's activation accepted.** That
     activation has no instance of its own, so it declares nothing, and calls
     go one deep.
   - **A call with nothing outstanding accepted.** There is no instance to
     check it against.
   - **Refused having spent the budget.** A refused call is no activation.

.. comp_req:: A call that does not fill its node type's parameters is refused
   :id: CREQ_RUN_REFUSES_UNFILLED_CALL
   :derived_from: FEAT_YIELD_UNDECLARED_REFUSED
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If a reported call gives a context for a parameter its node type does not declare or of another type than declared or gives none for a required one, then Workflow run shall refuse that call naming each such parameter.

   What an activation is given is what its node type declares it needs; a call
   is held to the declaration a binding is held to by the validator.

   Failure modes:

   - **The first fault reported.** A caller fixing its call learns one at a
     time.
   - **Refused having offered the activation.** The node type runs without its
     inputs before the refusal arrives.

.. comp_req:: A call or exchange bringing a second context under a held identifier is refused
   :id: CREQ_RUN_REFUSES_SHARED_CALL_IDENTIFIER
   :derived_from: FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If a context a reported call or exchange brings into the run shares its identifier with a different context the run holds, then Workflow run shall refuse that report naming the identifier.

   Calls and exchanges bring contexts into the run as outputs do, so the same
   rule holds of them (``CREQ_RUN_REFUSES_SHARED_OUTPUT_IDENTIFIER``).

   Failure modes:

   - **Not checked.** A second identifier source reaches the run through a call
     where it could not through an output, and a record then holds two contexts
     under one identifier.
   - **The same context brought twice refused.** A window composing the one
     before it brings the earlier window's parts again, by reference, and each
     is the very context held.

.. comp_req:: An exchange is held with the activation that made it
   :id: CREQ_RUN_HOLDS_EXCHANGES
   :derived_from: FEAT_MODEL_EXCHANGE_RECORDED
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: event
   :statement: When its caller reports an exchange for the outstanding activation, Workflow run shall hold the exchange's offer, window, answer and calls with that activation, in the order reported.

   An exchange is what the activation sent out and what came back, with the
   calls the answer made; the run holds it so that its record can
   (``DEC_RECORD_HOLDS_EXCHANGES``).

   Failure modes:

   - **Held for the run rather than for the activation.** A resumed activation
     cannot tell its own exchanges from another's.
   - **Only exchanges that made calls held.** An answer with no call in it is
     paid for too, and bought again on resume.
   - **An exchange reported with nothing outstanding accepted.** It belongs to
     no activation.

The record
==========

.. comp_req:: A record holds every exchange
   :id: CREQ_RECORD_HOLDS_EXCHANGES
   :derived_from: FEAT_MODEL_EXCHANGE_RECORDED
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: ubiquitous
   :statement: Run record shall write every exchange its run holds with the activation it was made in, its offer, window and answer, and each call it made with the identifier its caller gave the call.

   Each context once, by identifier, as every other context of a record is
   (``DEC_RECORD_IN_TOML``), and the format's version is 2
   (``DEC_RECORD_HOLDS_EXCHANGES``).

   Failure modes:

   - **The window written as its rendering.** Its parts, and which of them was
     an answer, are what a resume sends again; a rendering is one flat text.
   - **The call's identifier made up.** The provider pairs a result with its
     call by it.
   - **Version 1 kept.** A reader of version 1 takes the record as having made
     no model calls, and pays for all of them again.

.. comp_req:: A resumed run holds its record's exchanges
   :id: CREQ_RECORD_KEEPS_EXCHANGES
   :derived_from: FEAT_RESUME_REPEATS_NO_ANSWER
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: ubiquitous
   :statement: Run record shall resume a run holding every exchange its record holds, with the activation it was made in and in the order recorded.

   What a resumed activation's script is answered from
   (``DEC_SCRIPT_REPLAYED_FROM_ITS_RECORD``).

   Failure modes:

   - **Exchanges of accepted outputs dropped as finished.** Harmless to the
     run, and the record written after the resume has lost what the one before
     it held.
   - **The order lost.** A script's second call is answered with its first
     answer.
   - **Held under a different activation.** A node type called twice has its
     calls answered from each other's exchanges.

.. comp_req:: A record holding a call the workflow would not accept is refused
   :id: CREQ_RECORD_REFUSES_UNDECLARED_CALL
   :derived_from: FEAT_RESUME_REFUSES_ANOTHER_RUN
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: unwanted
   :statement: If a record holds a call the workflow it is resumed against would not accept from the recorded activation with the recorded contexts, then Run record shall refuse to resume it naming the first such call.

   A record describes a run of its workflow only if the run would have
   accepted what it records (``DEC_RESUME_REPLAYS_CALLS``).
   ``CREQ_RECORD_REFUSES_DIVERGENCE`` refuses a recorded output the workflow
   would not have offered, which catches a call whose output was recorded; this
   is the call whose output was not, because its node type was still being
   performed - by a person, say - when the record was taken.

   Failure modes:

   - **Resumed, and refused at the call.** The run then fails halfway through
     a replay, where the record could have been refused whole.
   - **The call dropped from the resumed run.** The calling activation waits
     for an output nothing will offer.
   - **Named by its output.** It has none.

The script host
===============

.. comp_req:: A model is offered the node types its instance declares
   :id: CREQ_HOST_OFFERS_DECLARED
   :derived_from: FEAT_YIELD_OFFERS_DECLARED
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: ubiquitous
   :statement: Script host shall offer each model call a script makes the node types its activation's instance declares calls to, each once, as contexts made from each node type's declaration, and no other tool.

   Made once for the activation, from the declaration as it stands when the
   activation starts (``DEC_TOOLS_OFFERED_AS_CONTEXTS``).

   Failure modes:

   - **Every node type in the catalogue offered.** The model can call what the
     node was never given.
   - **A called node type's own model call offered its caller's calls.** Calls
     go one deep, and a called node type declares nothing.
   - **The offer built with no context behind it.** Its text is held by nothing,
     and the window is more than its contexts.
   - **A node type named twice offered twice.** The model is shown one tool
     twice; whether a provider accepts two tools of one name was not measured.

.. comp_req:: A model's calls are performed in order before it is asked again
   :id: CREQ_HOST_PERFORMS_CALLS
   :derived_from: FEAT_YIELD_CALL_PERFORMED
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a model's answer makes calls, Script host shall report each to the run and perform the activation the run offers for it, in the order the answer gives them, before sending the model another request.

   Each call is an activation of its own (``DEC_CALLS_IN_ORDER``), performed
   as any activation the run offers is: by the called node type's script in a
   state of its own, or handed over when a person performs it.

   Failure modes:

   - **The calls performed in another order.** A model that looks something up
     and then acts on it is answered in the wrong order.
   - **The model asked again after the first call.** It is shown one result of
     the several it asked for, and the other calls stand in its window with
     none.
   - **The called node type run in the caller's state.** It reads what the
     caller's script left, and the limits are shared.

.. comp_req:: The next window composes the last with the answer and the calls
   :id: CREQ_HOST_NEXT_WINDOW
   :derived_from: FEAT_YIELD_CALL_PERFORMED
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a model's calls all have outputs, Script host shall send the model a window composing the previous window, the answer, each call's contexts and each call's output, in that order.

   A composition by reference with an empty separator, of the prompt's type
   (``DEC_WINDOW_IS_A_CONTEXT``).

   Failure modes:

   - **Only the call's output sent.** The model is shown a result with no call
     and no question, and cannot tell what it answers.
   - **A new window built from copies.** The window's parts are no longer the
     contexts the record holds as answers and outputs, so none of them is sent
     as the message it is.
   - **A separator between the parts.** The window's rendering holds text that
     was never sent.

.. comp_req:: A malformed call fails its activation before any call is reported
   :id: CREQ_HOST_REFUSES_MALFORMED_CALL
   :derived_from: FEAT_YIELD_UNDECLARED_REFUSED
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a model's call is to a node type not offered or has arguments that are not an object of strings filling its parameters, then Script host shall fail the activation naming the node type and the fault, reporting none of that answer's calls.

   ``genai`` passes each of these through (``EVD_GENAI_PASSES_MALFORMED_CALLS``);
   the faults are the five ``DEC_MALFORMED_CALL_FAILS`` names.

   Failure modes:

   - **A number or an object passed on as text.** It has no text to be a
     context of, and ``genai``'s rendering of it is not what the model wrote.
   - **The calls before the malformed one performed.** Paid for, and then
     thrown away with the activation.
   - **Reported as a script error.** A caller cannot tell a model's bad call
     from a script that broke.
   - **The fault sent back to the model.** Text the host wrote, in the window,
     with no context behind it.

.. comp_req:: A call the run refuses fails the activation
   :id: CREQ_HOST_CALL_REFUSAL_CARRIED
   :derived_from: FEAT_YIELD_UNDECLARED_REFUSED
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If the run refuses a call the script host reports, then Script host shall fail that activation carrying the run's refusal.

   As an output the run refuses does (``CREQ_HOST_OUTPUT_REFUSAL_CARRIED``).

   Failure modes:

   - **The refusal swallowed.** The model is asked again with a call that has no
     result.
   - **Reported as a script error.** The run's refusal, a value, becomes prose.

.. comp_req:: Every answer is reported to the run before its calls
   :id: CREQ_HOST_REPORTS_EXCHANGES
   :derived_from: FEAT_MODEL_EXCHANGE_RECORDED
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a model call a script makes is answered, Script host shall report to the run the offer and window the call was sent, the answer, and each call the answer makes, before performing any of those calls.

   Failure modes:

   - **Reported after the calls.** An interruption during a called node type
     leaves a record whose calls have no answer they came from.
   - **Only a script's own call reported.** The continuations, each paid for,
     are missing from the record.
   - **The answer's text reported without the calls.** The next window names
     calls the record does not hold.

.. comp_req:: A record follows every answer
   :id: CREQ_HOST_RECORD_AFTER_ANSWER
   :derived_from: FEAT_RESUME_REPEATS_NO_ANSWER
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a model call a script makes is answered, Script host shall hand the scripted run's caller a record holding that answer.

   (``DEC_RECORD_AFTER_EACH_ANSWER``.)

   Failure modes:

   - **Handed over before the answer is held.** The record lacks exactly the
     answer that prompted it.
   - **Handed over only when an output is accepted.** An interruption before
     then pays for every answer of the activation again.

.. comp_req:: A resumed activation's model calls are answered from its record
   :id: CREQ_HOST_ANSWERS_FROM_RECORD
   :derived_from: FEAT_RESUME_REPEATS_NO_ANSWER
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a resumed activation's script sends the window and offer its record holds an answer for, Script host shall answer it from the record, counting it against the model call limit, without sending a request.

   The script is run again (``DEC_SCRIPT_REPLAYED_FROM_ITS_RECORD``); its model
   calls are not. A recorded call whose output the record holds is not
   performed again either: the run holds that output
   (``FEAT_RESUME_REPEATS_NO_OUTPUT``).

   Failure modes:

   - **The request sent anyway.** The measured state
     (``EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS``).
   - **Not counted.** An activation interrupted and resumed n times gets n
     limits.
   - **Answered by position alone.** A script that now sends something else is
     answered with what was said to something it no longer asks.

.. comp_req:: A resumed activation that sends something else fails
   :id: CREQ_HOST_REPLAY_DIVERGED
   :derived_from: FEAT_RESUME_REPEATS_NO_ANSWER
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a resumed activation's script sends a window or offer that differs in type or content from the one its record holds at that point, then Script host shall fail that activation as diverged without sending a request.

   Compared by type and content, since the identifiers a replay makes are new
   (``DEC_SCRIPT_REPLAYED_FROM_ITS_RECORD``).

   Failure modes:

   - **Compared by identifier.** Every replay diverges.
   - **The difference sent to the provider as a new call.** The run goes on
     with a record whose earlier answers were to other questions.
   - **Reported as a script error or as the provider's failure.** A caller
     cannot tell a script that changed from one that broke.

The model roster
================

.. comp_req:: Each part of a window is sent as the message it is
   :id: CREQ_ROSTER_PARTS_AS_MESSAGES
   :derived_from: FEAT_YIELD_CALL_PERFORMED
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: ubiquitous
   :statement: Model roster shall send each part of a window given as a model's answer as the model's turn with the calls given as its, each part given as a call's output as that call's result, and every other part as the user's.

   What each part is comes from the script host, which knows it from what the
   run holds (``DEC_WINDOW_IS_A_CONTEXT``). Measured, a window sent flat is
   answered as if the call had not been made (``EVD_FLAT_WINDOW_RECALLS``).

   Failure modes:

   - **Sent as one user message.** The measured recall.
   - **A call's result not paired with its call.** Both formats pair a result
     with its call by the provider's identifier
     (``EVD_GENAI_TOOL_CALL_ROUND_TRIP``), and a result paired with nothing is
     the result of no call.

.. comp_req:: An offered node type is sent as a tool
   :id: CREQ_ROSTER_OFFER_AS_TOOLS
   :derived_from: FEAT_YIELD_OFFERS_DECLARED
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: ubiquitous
   :statement: Model roster shall send each node type a call offers as a tool named by its name's rendering, described by its description's rendering, and taking one string for each parameter the offer names, marking each required parameter required.

   Every text sent is a rendering of the offer's contexts
   (``CREQ_ROSTER_CONTEXTS_WHOLE``), and the rest is the format's
   (``DEC_TOOLS_OFFERED_AS_CONTEXTS``).

   Failure modes:

   - **A parameter not marked required listed as required.** The model fills
     it with something rather than leave it out. The host marks every parameter
     it offers required (``DEC_EVERY_INPUT_REQUIRED``); the roster sends the
     marks it is given.
   - **An empty description sent as absent, or absent sent as empty.** Either
     is a different tool from the one offered.
   - **An empty tool list sent when nothing is offered.** Every model call a
     node makes without declaring calls then sends a different request from
     the one it sends today.

.. comp_req:: Each call an answer makes comes back as the provider sent it
   :id: CREQ_ROSTER_CALLS_RETURNED
   :derived_from: FEAT_YIELD_CALL_PERFORMED
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: ubiquitous
   :statement: Model roster shall return each call a model's answer makes, in the order the answer gives them, with the provider's identifier as sent, the name called, and its arguments as the provider gave them.

   The roster does not judge a call; the script host does
   (``CREQ_HOST_REFUSES_MALFORMED_CALL``), so a malformed one comes back
   malformed rather than dropped.

   Failure modes:

   - **A call dropped for naming nothing offered.** The model's mistake
     disappears, and the activation goes on as if it had answered.
   - **The identifier made up.** The result pairs with no call.
   - **The text beside a call dropped.** Anthropic's format carries both, and
     the answer is the text (``CREQ_ROSTER_ANSWER_AS_SENT``).
