===========================
A model yielding test cases
===========================

How each requirement in ``components/yield`` is to be checked, with cases
checking that the requirements every activation already had hold of a called
node type's, and two feature-level cases where the claim is about a whole run.
Results are never written here: they are imported from the test runner.

A case's id is the path of the Rust test that implements it, uppercased. The
core's cases live in ``agconflo-core``'s modules ``reader``, ``writer``,
``wiring``, ``run`` and ``record``; the script host's in ``agconflo-lua``'s
module ``scripted``, which drives a run; the roster's in its module ``models``
and the behaviour set's in ``behaviours``.

Every model here is the stub the other cases use, answering in OpenAI's or
Anthropic's format with replies scripted per request and recording each request
it receives. A call from the loaded local model was measured outside the tests
(``EVD_LOCAL_MODEL_YIELDS``). Where a case says "both formats", it runs once
against each.

Every failure mode listed in ``components/yield`` is named by the case that
catches it.

Reading and writing
===================

.. test_case:: An instance's calls are read in the order written
   :id: TEST_READER_CALLS_READ_IN_ORDER
   :verifies: CREQ_READER_CALLS
   :test_kind: positive
   :coverage: partial

   An instance declaring three calls, one of them twice, reads them in the
   order written, the repeat kept. An instance with no calls key declares none.
   A calls key on a node type document is kept as a key the reader does not
   name, and gives no instance a call. The same document's description of a
   node type - what a model it is offered to is shown - is read as written, and
   one without is read as empty.

   Catches: the key left unread; a calls key on a node type document read as
   calls.

.. test_case:: A calls key that is not a list of strings is a located fault
   :id: TEST_READER_MALFORMED_CALLS_LOCATED
   :verifies: CREQ_READER_CALLS
   :test_kind: error_path
   :coverage: partial

   A calls key holding a string, a list holding a number, and a table are each
   refused as a document that cannot be read, naming the document and the line
   and column of the key. Afterwards nothing has been read: there is no
   definition to run.

   A call to a node type no document declares reads without complaint, and the
   wiring defect is the validator's.

   Catches: a calls key that is not a list of strings read as no calls; the
   names resolved while reading.

.. test_case:: An instance's calls survive being written back
   :id: TEST_WRITER_CALLS_READ_BACK
   :verifies: CREQ_WRITER_WRITES
   :test_kind: property
   :coverage: partial

   For any definition whose instances declare calls, and any change to them -
   a call added, removed, reordered, an instance's calls emptied - writing it
   into the document it came from and reading that back gives the same calls in
   the same order. A document whose calls did not change is written back byte
   for byte.

Checking the wiring
===================

.. test_case:: A call to a missing node type is reported
   :id: TEST_WIRING_CALL_TO_MISSING_TYPE_IS_REPORTED
   :verifies: CREQ_VALIDATOR_CALL_RESOLVES
   :test_kind: error_path
   :coverage: full

   Two instances, one declaring two calls to node types the definition lacks
   and one declaring one, beside an unrelated binding defect. Four defects are
   reported: one per bad call, each naming its instance and node type as
   values, and the binding's. The run refuses to start, carrying all four, and
   nothing is offered.

   Controls that must pass: a call to a node type the definition carries, and
   an instance whose own node type is missing, which is reported once, as the
   binding defect it already was, with nothing added for calls.

   Catches: not checked; reported once for the workflow; the instance's own
   node type checked instead.

.. test_case:: A call to a node type whose name a provider refuses is reported
   :id: TEST_WIRING_UNPORTABLE_CALL_NAME_IS_REPORTED
   :verifies: CREQ_VALIDATOR_CALL_NAME
   :test_kind: error_path
   :coverage: partial

   Calls to node types named ``look up``, ``a.b``, ``é``, and a name of 65
   characters - each a node type the definition carries - are each reported as
   a defect naming the instance and the name, all in one refusal, and the run
   does not start. A 100-character name, which Anthropic's rule alone would
   pass, is among them.

   Controls that must pass: a name of exactly 64 characters, one using a
   letter, a digit, an underscore and a hyphen, and a node type with an illegal
   name that no instance calls.

   Catches: checked against one provider's rule; checked with a search rather
   than a whole match; every node type's name checked.

.. test_case:: A called name is reported exactly when it breaks the rule
   :id: TEST_WIRING_CALL_NAMES_MATCH_THE_RULE
   :verifies: CREQ_VALIDATOR_CALL_NAME
   :test_kind: property
   :coverage: partial

   For any string used as a called node type's name, the validator reports a
   defect for it exactly when the string does not match
   ``^[a-zA-Z0-9_-]{1,64}$``, with strings drawn to sit near the edges: 0, 1,
   64 and 65 characters, and one character outside the set among legal ones.

The run
=======

.. test_case:: A declared call is offered next, with its inputs by parameter
   :id: TEST_RUN_DECLARED_CALL_IS_OFFERED_NEXT
   :verifies: CREQ_RUN_CALL_OFFERED
   :test_kind: positive
   :coverage: full

   A workflow with two instances ready at once, the first declaring a call to a
   node type of two parameters. During the first's activation, a call giving
   the parameters in the reverse of their declared order is reported; the next
   activation offered is the called node type's, for the first instance, with
   its inputs in declared order, and the run has spent one activation more. Once
   the called output is accepted, the first instance's own output is accepted as
   its activation's, and only then is the second instance offered.

   Catches: another instance offered first; the calling activation dropped; not
   counted; its inputs in the order the call gave them.

.. test_case:: A called node type's output goes back to the caller alone
   :id: TEST_RUN_CALL_OUTPUT_GOES_BACK_TO_THE_CALLER
   :verifies: CREQ_RUN_CALL_OUTPUT_TO_CALLER
   :test_kind: positive
   :coverage: full

   The calling instance is the designated one and has a consumer. The called
   node type handing back a context it was called with is refused, since that
   would credit the caller's context to it too. When the
   called node type's output is accepted, the calling activation is outstanding
   again; the run has not completed, its consumer is not offered, and the
   calling instance's own output - a composition holding the called output - is
   then accepted and completes the run. Meanwhile the run gives the called
   output back by the call's identifier, and nothing for an identifier no call
   had.

   Catches: taken as the calling instance's output; the calling activation left
   outstanding nowhere; the output not held.

.. test_case:: A call its instance does not declare is refused
   :id: TEST_RUN_UNDECLARED_CALL_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNDECLARED_CALL
   :test_kind: error_path
   :coverage: full

   Refused, each naming the instance and the node type as values: a call to a
   node type the catalogue holds and the instance does not declare; a call
   reported during a called node type's activation, to a node type its caller
   declares; and, as the run's existing refusal, a call with nothing
   outstanding. After each, the budget spent is unchanged, the same activation
   is outstanding, and asking for the next step offers it again.

   Catches: checked against the catalogue; a call reported by a called node
   type's activation accepted; a call with nothing outstanding accepted;
   refused having spent the budget.

.. test_case:: A call that does not fill its node type's parameters is refused
   :id: TEST_RUN_UNFILLED_CALL_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNFILLED_CALL
   :test_kind: error_path
   :coverage: full

   One call giving a parameter the node type does not declare, leaving one it
   declares without a context, and giving another a context of the wrong type
   is refused once, naming all three parameters. Nothing is offered, the budget
   is unchanged, and the calling activation is outstanding.

   Control that must pass: a call filling every parameter with its declared
   type.

   Catches: a call as declared refused; the first fault reported; refused
   having offered the activation.

.. test_case:: A call or exchange bringing a second context under a held identifier is refused
   :id: TEST_RUN_CALL_SHARING_AN_IDENTIFIER_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_SHARED_CALL_IDENTIFIER
   :test_kind: error_path
   :coverage: full

   A call whose input is a different context under an identifier the run holds,
   made from a second source, is refused naming that identifier; so is an
   exchange whose answer is one. After each, nothing is held from the report
   and the activation is outstanding.

   Control that must pass: an exchange whose window composes the previous
   window, bringing the earlier window's parts again by reference.

   Catches: not checked; the same context brought twice refused.

.. test_case:: Exchanges are held with the activation that made them
   :id: TEST_RUN_EXCHANGES_HELD_WITH_THEIR_ACTIVATION
   :verifies: CREQ_RUN_HOLDS_EXCHANGES
   :test_kind: positive
   :coverage: full

   Two activations each report two exchanges, one of them with no calls; each
   activation holds its own two, in the order reported, each exchange holding
   its calls whole. A call's output is given back to the activation that made
   the call and to no other, though the other's call has the same identifier. An
   exchange reported with nothing outstanding is refused, and nothing is held.

   Catches: held for the run rather than for the activation; only exchanges
   that made calls held; an exchange reported with nothing outstanding
   accepted.

.. test_case:: A run with calls never spends more activations than its budget
   :id: TEST_RUN_ACTIVATIONS_WITH_CALLS_NEVER_EXCEED_BUDGET
   :verifies: CREQ_RUN_STOPS_AT_BUDGET
   :test_kind: property
   :coverage: partial

   For any budget and any sequence of calls reported during any activations,
   the activations offered, the called ones counted, never exceed the budget,
   and a call reported with the budget spent ends the run as over budget
   without offering the called node type's activation.

   The node type called twice in one run makes the per-instance count wrong
   where it was indistinguishable before, so this case catches
   ``CREQ_RUN_STOPS_AT_BUDGET``'s "counted per instance rather than per
   activation", which ``tests/run`` records as untestable while no instance
   activated twice.

.. test_case:: A called node type's output of an undeclared type is refused
   :id: TEST_RUN_CALLED_OUTPUT_OF_UNDECLARED_TYPE_IS_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNDECLARED_OUTPUT
   :test_kind: error_path
   :coverage: partial

   A called node type declaring ``note`` output, called from an instance whose
   node type declares ``draft``. A ``draft`` reported for the called activation
   is refused naming the calling instance, ``note`` and ``draft``, and the
   called activation stays outstanding; a ``note`` is then accepted.

Records and resuming
====================

.. test_case:: A record holds every exchange, with the provider's call identifiers as sent
   :id: TEST_RECORD_HOLDS_EXCHANGES
   :verifies: CREQ_RECORD_HOLDS_EXCHANGES
   :test_kind: positive
   :coverage: partial

   A run with an exchange that made no call and one that made two, with call
   identifiers ``call_7``, ``toolu_7`` and one of 32 characters, is written as a
   record of version 2 in which each exchange names its activation, its offer,
   window and answer by identifier, and each call its identifier as given. The
   window's parts are recorded, not its rendering.

   Catches: the window written as its rendering; the call's identifier made up;
   version 1 kept.

.. test_case:: A record of version 1 is refused
   :id: TEST_RECORD_VERSION_ONE_REFUSED
   :verifies: CREQ_RECORD_REFUSES_UNREADABLE
   :test_kind: error_path
   :coverage: partial

   A record written before exchanges existed is refused as unreadable, naming
   the version and where in the text it is, and no run is resumed - rather than
   one resumed as having made no model call.

.. test_case:: A resumed run holds its record's exchanges
   :id: TEST_RECORD_EXCHANGES_KEPT
   :verifies: CREQ_RECORD_KEEPS_EXCHANGES
   :test_kind: property
   :coverage: full

   For any run with calls and exchanges, resumed from every record it can be
   written as, the resumed run holds the same exchanges with the same
   activations in the same order, and written again gives the same text.

   Catches: exchanges of accepted outputs dropped as finished; the order lost;
   held under a different activation.

.. test_case:: A record holding a call the workflow would not accept is refused
   :id: TEST_RECORD_UNDECLARED_CALL_REFUSED
   :verifies: CREQ_RECORD_REFUSES_UNDECLARED_CALL
   :test_kind: error_path
   :coverage: full

   A record taken while a called node type was outstanding, resumed against the
   same workflow with the calling instance's calls key removed, is refused
   naming the call, and no run is resumed. The same record against the
   unchanged workflow resumes with the called activation outstanding.

   Catches: resumed, and refused at the call; the call dropped from the resumed
   run; named by its output.

The script host
===============

.. test_case:: A model is offered the declared node types and nothing else
   :id: TEST_SCRIPTED_OFFER_IS_THE_DECLARED_CALLS
   :verifies: CREQ_HOST_OFFERS_DECLARED
   :test_kind: positive
   :coverage: full

   An instance declaring ``lookup`` twice, in a workflow whose catalogue also
   holds ``search``. Its model call's request offers one tool, ``lookup``, whose
   name, description and parameters come from the node type's declaration; the
   offer is held by the run as contexts. ``lookup``'s own script's model call
   sends no tools at all, and neither does a node declaring no calls.

   Catches: every node type in the catalogue offered; a called node type's own
   model call offered its caller's calls; the offer built with no context
   behind it; a node type named twice offered twice.

.. test_case:: A model's calls are performed in order before it is asked again
   :id: TEST_SCRIPTED_CALLS_PERFORMED_IN_ORDER
   :verifies: CREQ_HOST_PERFORMS_CALLS
   :test_kind: positive
   :coverage: full

   An answer making two calls, to node types whose scripts each output which
   call they were. Both run, first then second, before the stub receives its
   next request, which carries both results. A called node type's script finds
   nothing its caller's script set in its own state.

   Catches: the calls performed in another order; the model asked again after
   the first call; the called node type run in the caller's state.

.. test_case:: The next window composes the last with the answer and the calls
   :id: TEST_SCRIPTED_NEXT_WINDOW_COMPOSES_THE_LAST
   :verifies: CREQ_HOST_NEXT_WINDOW
   :test_kind: positive
   :coverage: full

   After one call, the window the run holds for the continuation has as its
   parts the previous window, the answer, the call's contexts and the call's
   output - each the very context held, not a copy - an empty separator, and
   the prompt's declared type. Its rendering is the text the request carries.

   Catches: only the call's output sent; a new window built from copies; a
   separator between the parts.

.. test_case:: A malformed call fails its activation, and no call of its answer runs
   :id: TEST_SCRIPTED_MALFORMED_CALL_FAILS
   :verifies: CREQ_HOST_REFUSES_MALFORMED_CALL
   :test_kind: error_path
   :coverage: full

   Five answers, each with a well-formed call first and then one of: a call to
   a node type not offered; arguments that are a bare string; a parameter the
   node type does not declare; a required parameter missing; a number where a
   string belongs. Each fails the activation with the malformed-call failure,
   naming the node type and which of the five faults it was, as values. In each,
   the well-formed call's node type - whose script would fail the test if run -
   never runs, the run offers no called activation and spends nothing for one,
   and the stub receives no further request.

   Catches: a number or an object passed on as text; the calls before the
   malformed one performed; reported as a script error; the fault sent back to
   the model.

.. test_case:: A call the run refuses fails the activation with the refusal
   :id: TEST_SCRIPTED_REFUSED_CALL_FAILS_WITH_THE_REFUSAL
   :verifies: CREQ_HOST_CALL_REFUSAL_CARRIED
   :test_kind: error_path
   :coverage: partial

   The host's own check makes a call the run would refuse unreachable from a
   model, so the case makes one on purpose: the host's offer is built from a
   declaration the run is not given, the one way the two can disagree. The
   activation fails carrying the run's refusal as a value, and the stub
   receives no further request.

   Catches: the refusal swallowed; reported as a script error.

.. test_case:: Every answer is reported before its calls run
   :id: TEST_SCRIPTED_EXCHANGES_REPORTED_BEFORE_CALLS
   :verifies: CREQ_HOST_REPORTS_EXCHANGES
   :test_kind: positive
   :coverage: full

   A called node type whose script records the run's state when it starts: the
   exchange whose answer called it is already held, with its call. A script
   whose answer makes no call has its exchange held too, and so does each
   continuation.

   Catches: reported after the calls; only a script's own call reported; the
   answer's text reported without the calls.

.. test_case:: A record is handed over after every answer
   :id: TEST_SCRIPTED_RECORD_AFTER_EACH_ANSWER
   :verifies: CREQ_HOST_RECORD_AFTER_ANSWER
   :test_kind: positive
   :coverage: full

   A run whose one node yields once hands its caller records at the start,
   after the first answer, after the called output, after the second answer
   and after the node's output, and each record handed after an answer holds
   that answer.

   Catches: handed over before the answer is held; handed over only when an
   output is accepted.

.. test_case:: A run resumed from any record asks no model twice
   :id: TEST_SCRIPTED_RESUMED_MID_CALL_ASKS_NOTHING_TWICE
   :verifies: CREQ_HOST_ANSWERS_FROM_RECORD
   :test_kind: property
   :coverage: full

   For a run whose nodes yield, resumed in a fresh run from every record it
   hands over, the resumed run ends with the same output, and the requests the
   stub receives across both are each the first of their kind: no window the
   record answered is sent again, and no call whose output the record holds is
   performed again - the resumed run ends holding the events the uninterrupted
   one held. Among the runs is one whose called node type asks a model of its
   own, so that some records are taken while it is being performed, its answer
   held and its caller's continuation not yet sent. Resumed with a model call
   limit equal to the
   requests the record answered, the next new request fails at the limit, as
   it would have in the run that was recorded.

   Catches: the request sent anyway; not counted; answered by position alone.

.. test_case:: A resumed activation whose script now sends something else fails
   :id: TEST_SCRIPTED_REPLAY_THAT_DIVERGES_FAILS
   :verifies: CREQ_HOST_REPLAY_DIVERGED
   :test_kind: error_path
   :coverage: full

   A record taken after an answer, resumed with the script edited to send a
   different prompt, fails the activation as diverged, and the stub receives
   nothing. So does the same record resumed with the called node type's
   description edited, which changes the offer. The failure is its own kind,
   neither a script error nor a model's failure.

   Control that must pass: the same record resumed with the script unchanged,
   whose replayed prompt has new identifiers and the same type and content.

   Catches: compared by identifier; the difference sent to the provider as a
   new call; reported as a script error or as the provider's failure.

.. test_case:: A continuation counts against the model call limit
   :id: TEST_SCRIPTED_CONTINUATION_COUNTS_AGAINST_THE_LIMIT
   :verifies: CREQ_HOST_MODEL_CALL_LIMIT
   :test_kind: error_path
   :coverage: partial

   A node whose model yields, run with the default limit of one: the called node
   type runs and its output is held, and the continuation fails the activation
   as having exceeded its model call limit, with no second request sent.

.. test_case:: A called node type with nothing to perform it refuses the run
   :id: TEST_BEHAVIOURS_CALLEE_WITHOUT_BEHAVIOUR_REFUSED
   :verifies: CREQ_BEHAVIOURS_REFUSE_MISSING
   :test_kind: error_path
   :coverage: partial

   A node type an instance declares a call to, given no script and not named
   as performed by a person, refuses the run before it starts, naming that node
   type, alongside any other missing behaviour. Nothing runs.

.. test_case:: A person can perform a called node type
   :id: TEST_SCRIPTED_PERSON_AS_CALLEE
   :verifies: CREQ_HOST_HANDS_OVER_PERSON_STEP
   :test_kind: positive
   :coverage: partial

   A model calls a node type a person performs. The run returns that activation
   to its caller, for the calling instance, with the call's contexts as inputs.
   Answered from the last record in a fresh run, the calling script is run
   again, its first model call answered from the record, and the stub receives
   one request before and one continuation after, carrying the person's text as
   the call's result.

The model roster
================

.. test_case:: Each part of a window is sent as the message it is
   :id: TEST_MODELS_WINDOW_PARTS_AS_MESSAGES
   :verifies: CREQ_ROSTER_PARTS_AS_MESSAGES
   :test_kind: positive
   :coverage: full

   A window of a prompt, an answer that made a call, the call's contexts and its
   output, sent in both formats. The request holds a user message with the
   prompt, the model's turn with the call under the provider's identifier and
   the answer's text beside it, and the call's result paired with that
   identifier.

   Catches: sent as one user message; a call's result not paired with its call.

.. test_case:: What a continuation sends is its contexts and nothing else
   :id: TEST_MODELS_CONTINUATION_SENDS_CONTEXTS_WHOLE
   :verifies: CREQ_ROSTER_CONTEXTS_WHOLE
   :test_kind: property
   :coverage: partial

   For any prompt, answer, description and argument text - whitespace at either
   end, line endings of both kinds, characters outside ASCII, quotes and
   backslashes - every text a continuation's request carries in either format
   is the rendering of one of the window's or the offer's contexts, byte for
   byte, and every such rendering is carried.

.. test_case:: An offered node type is sent as a tool
   :id: TEST_MODELS_OFFER_SENT_AS_TOOLS
   :verifies: CREQ_ROSTER_OFFER_AS_TOOLS
   :test_kind: positive
   :coverage: full

   An offer naming two parameters, the first marked required and the second
   not, and an empty description is sent, in both formats, as a tool of its
   name with an empty description, two string parameters and only the first
   listed as required. A call offering nothing sends no tools field. The host
   marks every parameter it offers required (``DEC_EVERY_INPUT_REQUIRED``); the
   roster sends the marks it is given.

   Catches: a parameter not marked required listed as required; an empty description sent
   as absent, or absent sent as empty; an empty tool list sent when nothing is
   offered.

.. test_case:: Each call an answer makes comes back as sent
   :id: TEST_MODELS_CALLS_RETURNED_AS_SENT
   :verifies: CREQ_ROSTER_CALLS_RETURNED
   :test_kind: positive
   :coverage: full

   Answers in both formats with two calls, text beside them in Anthropic's, a
   name never offered and arguments that are a bare string. Both calls come
   back in order, with the provider's identifiers, the names as sent - the
   unoffered one included - and the arguments as given; the text comes back as
   the answer.

   Catches: a call dropped for naming nothing offered; the identifier made up;
   the text beside a call dropped.

The whole feature
=================

.. test_case:: A model yields, and the run goes on with what the call produced
   :id: TEST_SCRIPTED_MODEL_YIELDS
   :verifies: FEAT_YIELD_CALL_PERFORMED
   :test_kind: positive
   :coverage: partial

   In both formats, a node's model calls a declared node type, whose script
   outputs a note; the model is asked again with the note as the call's result
   and answers; the node outputs a composition of the answer, and the run
   completes with it. The record holds both exchanges, the call and its output.

.. test_case:: A run interrupted during a call is resumed without buying an answer twice
   :id: TEST_SCRIPTED_INTERRUPTED_CALL_RESUMES
   :verifies: FEAT_RESUME_REPEATS_NO_ANSWER
   :test_kind: positive
   :coverage: partial

   The measurement ``EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS`` made into a case
   the other way round: a script making two model calls, its run dropped while
   the second is held open, resumed from the last record handed over, makes
   only the second call, and the stub receives the first once in all.
