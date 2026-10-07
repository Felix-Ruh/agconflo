==================================================
The engine explaining itself to a model test cases
==================================================

How each requirement in ``components/reflection`` and ``features/reflection``
is to be checked. Results are never written here: they are imported from the
test runner.

A case's id is the path of the Rust test that implements it, uppercased. The
cases live in ``agconflo-lua``'s modules ``scripted`` and ``models``, and in
``agconflo-core``'s modules ``run`` and ``record``. No case reaches a
provider, a container or the network: a model is the loopback stub.

.. test_case:: A call to a node type not offered is answered, and the model goes on
   :id: TEST_SCRIPTED_REFUSED_CALL_ANSWERED
   :verifies: CREQ_HOST_ANSWERS_REFUSAL, CREQ_HOST_WINDOW_WITH_REFUSALS, FEAT_REFUSAL_GIVEN_TO_MODEL
   :test_kind: positive
   :coverage: full

   The stub's model first calls ``bash``, which the instance does not
   declare, then the node type it does, then answers without calling. The
   run completes; the callee was performed once; the second request carried
   the refused call in the model's turn and, as its result, a text naming
   ``bash``, the fault and the node type offered; the window sent second
   composes the first, the answer, the refused call's arguments and the
   refusal, and the refusal is of the prompt's type.

   Catches: the activation failed, the refusal left out of the window, the
   answer of an engine-owned type, the offered node types left out.

.. test_case:: Each fault of a call is answered naming it, and nothing is performed for it
   :id: TEST_SCRIPTED_EACH_FAULT_ANSWERED
   :verifies: CREQ_HOST_ANSWERS_REFUSAL, CREQ_HOST_REFUSES_MALFORMED_CALL
   :test_kind: error_path
   :coverage: full

   One run per fault: a node type not offered, arguments that are not an
   object, a parameter the node type does not declare, a required one
   missing, a value that is not a string. Each refusal names that fault and
   the node type called; the callee's script ran no times; each run
   completes after the model's next answer. Refusing the same call twice
   gives the same text, the control that it holds nothing that changes
   between refusals.

   Catches: a fault answered as another, a refused call performed anyway,
   text that differs between two refusals of one call.

.. test_case:: The calls beside a refused one are performed in order
   :id: TEST_SCRIPTED_UNREFUSED_CALLS_PERFORMED
   :verifies: CREQ_HOST_WINDOW_WITH_REFUSALS
   :test_kind: error_path
   :coverage: full

   One answer makes three calls, the second to a node type not offered. The
   first and third are performed, in that order; the window after them
   composes the one before and the answer, the two performed calls'
   contexts, the refused call's arguments, the two outputs, and the refused
   call's answer - the same order whether composed live or from a record,
   which holds performed and refused calls apart.

   Catches: no call performed when one is refused, as before; the calls
   reordered; the refusal left out of the window.

.. test_case:: A model that keeps making refused calls stops at its model call limit
   :id: TEST_SCRIPTED_REFUSALS_END_AT_THE_LIMIT
   :verifies: CREQ_HOST_ANSWERS_REFUSAL
   :test_kind: error_path
   :coverage: full

   The stub's model calls ``bash`` on every turn, under a limit of three
   model calls. The activation fails as having exceeded its model call
   limit after three requests, and the record holds the three exchanges,
   each with its refused call.

   Catches: a refused call that loops for ever, or that ends the activation
   with another failure.

.. test_case:: A call the run refuses is answered as the host's are
   :id: TEST_SCRIPTED_RUN_REFUSAL_ANSWERED
   :verifies: CREQ_HOST_ANSWERS_REFUSAL, CREQ_HOST_CALL_REFUSAL_CARRIED
   :test_kind: error_path
   :coverage: partial

   The host's own check makes a call the run would refuse unreachable from a
   model, so the case makes one on purpose: the host's offer is built from a
   declaration the run is not given, the one way the two can disagree. The
   model is answered with a text naming the run's refusal and what may be
   called, is asked once more, and the activation completes.

   Catches: the run's refusal ending the activation, as before; the refusal
   swallowed.

.. test_case:: A resumed activation is given the refusal its record holds, asking nothing
   :id: TEST_SCRIPTED_REFUSAL_REPLAYED
   :verifies: CREQ_HOST_REFUSAL_FROM_RECORD, FEAT_REFUSAL_RESUMED
   :test_kind: positive
   :coverage: full

   A run whose model made a refused call is interrupted at a person's step
   after it, written down, and resumed from its record against a stub that
   counts requests. The resumed run sends no request for the exchanges the
   record holds, the refusal it gives is the recorded context, identifier
   included, and the run completes as the uninterrupted one did.

   Catches: the refusal made again with a new identifier, a request sent for
   a recorded exchange, a resume that diverges at the refused call.

.. test_case:: A run holds each refused call with its exchange
   :id: TEST_RUN_REFUSED_CALLS_HELD
   :verifies: CREQ_RUN_HOLDS_REFUSED_CALLS
   :test_kind: positive
   :coverage: full

   An exchange reported with a refused call beside a call is held with both,
   its refused call's arguments and answer among the contexts the run holds;
   an exchange whose refused call brings a context sharing an identifier
   with a different one the run holds is refused naming it.

   Catches: a refused call dropped, its contexts not brought into the run.

.. test_case:: A record writes and reads back every refused call as it was
   :id: TEST_RECORD_REFUSED_CALLS_KEPT
   :verifies: CREQ_RECORD_WRITES_REFUSED_CALLS, CREQ_RECORD_KEEPS_REFUSED_CALLS
   :test_kind: positive
   :coverage: full

   A property over runs whose exchanges hold any number of refused calls,
   with arguments of any JSON shape and any text: written and read back,
   the run holds the same exchanges, each refused call with the same
   identifier, node type, arguments and answer, in the same order. A record
   of version 4 is refused naming its version.

   Catches: a refused call left out of the record, read back in another
   order or with another exchange, arguments written as a call's inputs.

.. test_case:: A refused call is sent back as the model made it, with its answer as its result
   :id: TEST_MODELS_REFUSED_CALL_SENT_AS_MADE
   :verifies: CREQ_ROSTER_REFUSED_CALL_AS_MADE
   :test_kind: positive
   :coverage: full

   A window holding a model's turn with a refused call whose arguments are
   a string, a number, an array or an object, in OpenAI's format and in
   Anthropic's, is sent with that call's identifier, name and arguments as
   given, and its answer as that call's result, as the stub records.

   Catches: the arguments sent as an object of strings, the refused call
   left out of the turn.
