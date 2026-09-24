================================
Decisions about a model yielding
================================

The choices ``STKH_MODEL_YIELDS`` and ``STKH_TOOLS_AS_NODES`` are specified
against, each resting on the measurements in ``evidence/yield``. Four of them
replace an earlier decision whose statement closed a list the yield extends -
what a call sends, what a record holds, what a resume replays, when a record is
handed over. Each replaced decision is kept where it was, marked superseded.

Two things are left out on purpose. A workflow cannot be called yet: nothing
invokes a workflow, and its signature is not settled
(``DEC_ARGUMENTS_PER_ENTRY``), so the node types a model may call are node types
until ``STKH_WORKFLOW_AS_NODE`` has its invocation. And nothing here reaches a
tool outside the engine: a Lua script cannot, by design, so a node type wrapping
an MCP server needs a behaviour written in Rust, which is the next slice.

.. dec:: What a node may call is declared on its instance
   :id: DEC_CALLS_DECLARED_ON_THE_INSTANCE
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_READER_KEEPS_UNKNOWN_CALLS_KEY
   :statement: Agconflo shall take the node types a node's model may call from a list its instance declares in the workflow document.

   ``STKH_MODEL_YIELDS`` makes what may be called something declared for the
   node, and only that. The instance is where it is declared, as
   ``calls = ["lookup"]``, because the workflow is where wiring lives: one node
   type used in two workflows may call different things in each, and a list on
   the node type would give both the same callees from a document every
   workflow shares (``DEC_TYPES_IN_OWN_DOCUMENTS``).

   The list names node types rather than instances. An instance activates once
   in a run (``DEC_ACTIVATION_ONCE_PER_RUN``) and its output feeds its
   bindings, so calling one would activate it out of order and hand its output
   to two places.

   Measured, the reader keeps a ``calls`` key it does not read, on an instance
   or a node type, without a word (``EVD_READER_KEEPS_UNKNOWN_CALLS_KEY``). So
   declaring calls changes the format whichever document holds them, and a
   misspelt key declares nothing: unknown keys are kept on purpose
   (``CREQ_WRITER_KEEPS_UNREAD``), so the reader cannot tell a typo from a key
   another tool wrote. The model is then offered no tool, which its first run
   shows.

.. dec:: A call is an activation of the run, for the instance that made it
   :id: DEC_CALL_IS_AN_ACTIVATION
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_LUA_SCRIPT_SUSPENDED_FOR_A_CALLEE, EVD_LUA_PARKED_SCRIPT_DROPPED
   :statement: Agconflo shall perform a model's call as an activation of the called node type that the run offers for the calling instance, counts against its budget, and gives back only to the calling activation.

   ``STKH_MODEL_YIELDS`` has the thing called run as a step of the run. So the
   core offers it, as it offers every activation: it counts against the budget
   (``CREQ_RUN_STOPS_AT_BUDGET``), its output is checked and held as any output
   is, and the record holds it. Performing calls in the Lua crate alone was the
   alternative, and it is the tool loop inside a node that the stakeholder
   requirement turned down: the budget, the record and the identifier checks
   would never see a call.

   The calling activation stays outstanding while its callee is performed. The
   run still hands its caller one activation at a time
   (``DEC_RUN_IS_DRIVEN``) - the callee's - and the caller's goes on when the
   callee's output is accepted. That output goes back to the calling activation
   and to no binding, so ``DEC_ACTIVATION_ONCE_PER_RUN`` holds as it stands: a
   call activates a node type, not an instance, and nothing downstream can see
   two passes of one instance. A callee has no instance to declare calls of its
   own, so calls go one deep.

   Measured, a script can wait suspended while its callee's script runs in a
   state of its own, each held to its own limits
   (``EVD_LUA_SCRIPT_SUSPENDED_FOR_A_CALLEE``), and a script parked in the
   middle of a call can be dropped without harm
   (``EVD_LUA_PARKED_SCRIPT_DROPPED``). The second is what a callee performed by
   a person needs: it is handed over like any step a person performs, whole
   (``DEC_PERSON_PERFORMS_AN_ACTIVATION``), and the calling script is run again
   when the answer comes (``DEC_SCRIPT_REPLAYED_FROM_ITS_RECORD``).

.. dec:: A model call's window is a context, sent part by part as its provenance says
   :id: DEC_WINDOW_IS_A_CONTEXT
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_FLAT_WINDOW_RECALLS, EVD_GENAI_TWO_FORMATS_EXACT
   :supersedes: DEC_PROMPT_IS_A_CONTEXT
   :statement: Agconflo shall send each part of a model call's window, one context composed of the prompt a script passes and every answer, call and call output since, as the message its provenance makes it.

   What ``DEC_PROMPT_IS_A_CONTEXT`` decided stands: a call is made of contexts,
   so what was in its window has an exact answer. Its last clause cannot: "send
   that context's rendering as the call's only content". After a call, the
   window holds the prompt, the model's answer, the call and its output. Sent
   as one rendering, as a composition renders, the model called the same tool
   again in four of five trials; sent as the provider's messages, it answered
   from the result in five of five (``EVD_FLAT_WINDOW_RECALLS``).

   So the window is one context, composed by reference
   (``DEC_COMPOSITION_BY_REFERENCE``). The first is the prompt the script
   passes. Each later one composes the previous window, the model's answer, the
   contexts each of its calls was made with, and each call's output, in that
   order. Each part goes as the message its provenance makes it: a part the
   activation holds as a model's answer is the model's turn, carrying the calls
   made after it; a part it holds as a call's output is that call's result;
   every other part is the user's. A window with no answer in it is one user
   message holding its rendering, which is what is sent today.

   Writing a role on each context was the alternative. It puts a provider's
   vocabulary into every context, and a label is a claim nothing checks, where
   provenance is what the run itself accepted. Roles by context type would
   reserve type names, which a user's own types could collide with.

   A window takes the declared type of the prompt the script passed, so the
   engine names no type of its own. Its separator is empty, so nothing is sent
   between its parts that its rendering does not hold
   (``CREQ_ROSTER_CONTEXTS_WHOLE``).

.. dec:: A call is carried as the strings it was made with
   :id: DEC_CALL_CARRIES_STRING_VALUES
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_GENAI_REWRITES_CALL_JSON, EVD_LOCAL_MODEL_WRITES_COMPACT_ARGUMENTS
   :statement: Agconflo shall take a model's call as one text context per parameter it fills, holding the string the model gave that parameter, with the JSON around those strings left to the provider's format.

   A call's arguments are JSON, and ``genai`` sends them back as its own
   rendering of what it parsed: spaces, escapes and the form of a number are
   lost, and only the string values come back exactly
   (``EVD_GENAI_REWRITES_CALL_JSON``). Holding the arguments as the text the
   model wrote would have every later window claim bytes that were never sent,
   which ``CREQ_ROSTER_CONTEXTS_WHOLE`` forbids. Holding them as ``genai``'s
   rendering would record as the model's a text it did not write.

   Each string is a text context of the type the called node type declares for
   that parameter: the callee's input, exactly as a bound context is an
   instance's. The keys are parameter names, which the offer holds
   (``DEC_TOOLS_OFFERED_AS_CONTEXTS``); braces, quotes and escapes are the
   format's, as a message's role is. A value that is not a string has no text
   to be a context of, and fails the call (``DEC_MALFORMED_CALL_FAILS``).

   The local model writes its arguments compactly, so for it the two never
   differed (``EVD_LOCAL_MODEL_WRITES_COMPACT_ARGUMENTS``); a hosted provider's
   models were not measured.

.. dec:: The tools a model is offered are contexts made from the declaration
   :id: DEC_TOOLS_OFFERED_AS_CONTEXTS
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_GENAI_TOOL_CALL_ROUND_TRIP, EVD_GENAI_REWRITES_CALL_JSON
   :statement: Agconflo shall offer a model each node type its instance declares a call to as a context holding that node type's name, its description and its parameters' names, made from the node type's declaration when the activation starts.

   A tool is text a model reads: a name, what it does, what to fill in.
   ``FEAT_MODEL_WINDOW_IS_THE_PROMPT`` sends a model exactly the contexts a call
   is made with, so the offer is contexts too. It is also what keeps the record
   exact: the offer is recorded as it was sent, so a description edited while a
   run waits is a difference a resumed run can see.

   Each node type offered is one composition, with an empty separator, of text
   contexts: its name; its description, from an optional ``description`` key of
   its node type document and empty without one; and the name of each
   parameter, required ones first, in the order the node type declares them.
   Each takes the declared type of the prompt, as the window does.

   What ``genai`` adds is the format's own words: ``"type": "object"``,
   ``"string"`` for each parameter, the list of required ones, and per provider
   ``"type": "function"`` with ``"strict": false`` or ``input_schema``
   (``EVD_GENAI_TOOL_CALL_ROUND_TRIP``). Those say how a tool is written down,
   not what this one is, as a role does. Building each tool from the declaration
   with no context behind it was the alternative: its text would be held by no
   context, and ``FEAT_MODEL_WINDOW_IS_THE_PROMPT`` false on every call that
   offers one.

   Only the node types the instance declares are offered, and nothing else
   (``STKH_TOOLS_AS_NODES``). A callee's own model calls are offered nothing,
   since it has no instance to declare calls.

.. dec:: A called node type's name is one every provider accepts
   :id: DEC_TOOL_NAMES_PORTABLE
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_TOOL_NAME_RULES, EVD_LM_STUDIO_TOOL_NAMES_UNCHECKED
   :statement: Agconflo shall refuse to start a run whose workflow declares a call to a node type whose name does not match ^[a-zA-Z0-9_-]{1,64}$.

   The node type's name is the tool's name, because the model calls by it and a
   second name would be a mapping that can disagree with the first. Anthropic
   documents ``^[a-zA-Z0-9_-]{1,128}$`` and OpenAI the same characters up to 64
   (``EVD_TOOL_NAME_RULES``), and the rule is the two taken together. LM Studio
   checks neither (``EVD_LM_STUDIO_TOOL_NAMES_UNCHECKED``), so a workflow tried
   only on a local model would learn of a bad name on its first paid call.
   Checked with the wiring, it is refused before anything runs.

   Escaping a name the providers refuse was the alternative. The model would
   then call a name that is not the node type's, and two escaped names could
   meet. Neither provider's rule was measured, since there is no key for either
   here; the rule follows their documentation and moves with it.

.. dec:: A malformed call fails its activation
   :id: DEC_MALFORMED_CALL_FAILS
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_GENAI_PASSES_MALFORMED_CALLS
   :statement: Agconflo shall fail an activation whose model's answer makes a call that is undeclared or whose arguments are not an object of strings filling the called node type's parameters, performing none of that answer's calls.

   ``genai`` passes through a name never offered, a number for a string and a
   bare string for an object (``EVD_GENAI_PASSES_MALFORMED_CALLS``), so what a
   call is checked against is Agconflo's to check. The failure names the node
   type and which fault it was: not declared, arguments not an object, a
   parameter the node type does not declare, a required one missing, a value
   that is not a string. Every call of an answer is checked before any is
   performed, so a bad third call does not leave two performed and paid for.

   Sending the fault back to the model to correct was the alternative, and the
   one most frameworks take. It puts text the host wrote into the window with
   no context behind it, which is exactly what this project refuses to leave
   unaccounted for. It is worth having again under ``STKH_REFLECTION``, where
   the engine explaining itself to a model is the point - as a context,
   recorded like any other.

   Arguments that are not JSON at all fail inside ``genai`` before a call
   exists (``EVD_GENAI_PASSES_MALFORMED_CALLS``). That is a model call that
   failed, and is carried as one (``FEAT_MODEL_FAILURE_CARRIED``).

.. dec:: Several calls in one answer are performed in order
   :id: DEC_CALLS_IN_ORDER
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_GENAI_PASSES_MALFORMED_CALLS
   :statement: Agconflo shall perform the calls one answer makes in the order the answer gives them, each as an activation of its own, and call the model again once all of them have outputs.

   Models make several calls in one turn, and ``genai`` hands them over as
   several (``EVD_GENAI_PASSES_MALFORMED_CALLS``), so refusing them would refuse
   ordinary behaviour. Performing them at once was the other alternative, and
   the run hands its caller one activation at a time (``DEC_RUN_IS_DRIVEN``).
   The next window carries every call of the answer and every output, each
   paired with its call by the provider's identifier.

.. dec:: Every request to a model counts against the model call limit
   :id: DEC_EVERY_TURN_COUNTED
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_MODEL_CALLS_UNLIMITED
   :statement: Agconflo shall count every request a script's model call sends to a provider against the activation's model call limit, the requests continuing after a call included.

   ``DEC_MODEL_CALLS_COUNTED`` limits an activation's model calls because each
   is paid for, and a continuation after a call is a request like any other.
   Counting only a script's own calls would let one of them send as many
   requests as the model chooses to make calls, which is the unbounded shape
   measured (``EVD_MODEL_CALLS_UNLIMITED``).

   The default limit stays one. A node that yields has to be given a higher one
   by its caller, and without it the first continuation fails the activation at
   the limit, reported as the limit.

.. dec:: A run is recorded with every model call it made
   :id: DEC_RECORD_HOLDS_EXCHANGES
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_RUN_STATE_DERIVABLE, EVD_REPLAY_BY_NAME_ACCEPTS_REWIRING, EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS, EVD_GENAI_TOOL_CALL_ROUND_TRIP
   :supersedes: DEC_RECORD_IS_OUTPUTS
   :statement: Agconflo shall record a run as its budget, the activations it spent, its arguments, each output it accepted together with the inputs that output's activation was given, and each model call's window, answer and calls.

   Everything ``DEC_RECORD_IS_OUTPUTS`` recorded, for the reasons it gives, and
   one thing more. Its record held all of a run's state between activations
   (``EVD_RUN_STATE_DERIVABLE``) and nothing inside one: a script's answered
   model calls left no trace, and resumed they were made again
   (``EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS``). A call is a point the run can be
   written down at (``STKH_MODEL_YIELDS``), and it is inside an activation.

   So the record holds each exchange: the window a model call sent, the answer
   that came back, and each call the answer made - the provider's identifier as
   sent, the node type called, the contexts it was made with, and the callee's
   output once accepted - with the offer the call was made with. All by
   identifier, in the table that holds each context once
   (``DEC_RECORD_IN_TOML``). For every model call, not only those that yield: an
   answer with no call in it is still paid for, and a record without it is a
   resume that pays again.

   The provider's call identifier is kept as sent. A provider pairs a result
   with its call by it, and whether one accepts an identifier it did not issue
   was not measured; nothing is gained by making up another.

   ``genai``'s serialised conversation was the alternative, and continues
   byte for byte (``EVD_GENAI_TOOL_CALL_ROUND_TRIP``). It records a
   pre-release crate's internal layout, and hides each turn from provenance: the
   window would be a blob beside the contexts rather than made of them.

   The record's format version moves to 2, and a record of version 1 is refused
   as unreadable rather than read as having made no model calls.

.. dec:: A run is resumed by replaying its record's calls and outputs through the run
   :id: DEC_RESUME_REPLAYS_CALLS
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_REPLAY_BY_NAME_ACCEPTS_REWIRING
   :supersedes: DEC_RESUME_BY_REPLAY
   :statement: Agconflo shall resume a run by starting it from its recorded arguments and reporting each recorded call and output to it in order, refusing a record whose calls or outputs the run would not have been offered.

   ``DEC_RESUME_BY_REPLAY``'s reason, carried to calls: every check a run makes
   applies to a resumed run because it is the same code meeting the same
   values. A recorded call is reported to the run as its caller first reported
   it, so a record calling a node type the workflow no longer declares is
   refused by the run's own refusal of an undeclared call, and a callee's output
   is checked against its declaration as it was the first time.

.. dec:: An activation resumed mid-call runs its script again, answered from its record
   :id: DEC_SCRIPT_REPLAYED_FROM_ITS_RECORD
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_LUA_ORDER_REMEASURED, EVD_LUA_PARKED_SCRIPT_DROPPED, EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS
   :statement: Agconflo shall continue an activation resumed mid-call by running its script again and answering each model call its record holds from the record, failing it if a window or offer differs from the recorded one.

   A Lua state does not outlive its process, so an activation interrupted
   mid-call can only be continued by running its script again. Answering its
   model calls from the record is what makes that affordable: the script's own
   work is repeated, which costs instructions, and no answered call is paid for
   twice (``EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS``).

   A script run again need not send what it sent before. One that builds its
   prompt with ``pairs`` sends a different one in another process
   (``EVD_LUA_ORDER_REMEASURED``), and so does one edited since. Answering a
   different window with the recorded answer would answer a question the
   script did not ask. So each replayed window and offer is compared with the
   recorded one - by type and content, since identifiers made during the replay
   are new - and a difference fails the activation with a failure of its own,
   before anything is sent. When they agree, the activation goes on with the
   recorded contexts, so the record's identifiers stay the record's.

   Keeping the Lua state alive was the alternative, and dies with the process;
   dropping a parked one is safe (``EVD_LUA_PARKED_SCRIPT_DROPPED``), which is
   what an interrupted activation leaves behind.

.. dec:: A scripted run hands its caller a record after every output and every answer
   :id: DEC_RECORD_AFTER_EACH_ANSWER
   :dec_status: accepted
   :decided_on: 2026-09-24
   :supported_by: EVD_INTERRUPTED_RUN_REPEATS_CALLS, EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS
   :supersedes: DEC_SCRIPTED_RUN_HANDS_RECORDS
   :statement: Agconflo shall hand the caller of a scripted run the run's record when the run starts, each time the run accepts an output and each time a model call is answered.

   ``DEC_SCRIPTED_RUN_HANDS_RECORDS`` left the activation in progress to be
   lost, and what that cost was its answered model calls
   (``EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS``). With each answer recorded
   (``DEC_RECORD_HOLDS_EXCHANGES``), a record handed over after it loses nothing
   that was paid for. A callee's output is an output the run accepts, so a
   record follows each of those already.
