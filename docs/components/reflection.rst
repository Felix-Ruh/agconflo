=====================================================
Components of the engine explaining itself to a model
=====================================================

The requirements ``ARCH_REFUSAL_ANSWERED`` allocates to the four components
it uses: the script host, defined in ``components/behaviour``, the workflow
run, defined in ``components/run``, the run record, defined in
``components/resume``, and the model roster, defined in
``components/models``. Each title is the grammatical subject of the
requirements allocated to it, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

.. comp_req:: A refused call is answered with why
   :id: CREQ_HOST_ANSWERS_REFUSAL
   :derived_from: FEAT_REFUSAL_GIVEN_TO_MODEL
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When it or the run refuses a call a model made, Script host shall answer that call with a text context of its prompt's type naming the node type called, the fault and the node types offered.

   ``DEC_REFUSED_CALL_ANSWERED``. The script host refuses a call it cannot
   make into one (``CREQ_HOST_REFUSES_MALFORMED_CALL``); the run refuses one
   it was reported (``CREQ_RUN_REFUSES_UNDECLARED_CALL``,
   ``CREQ_RUN_REFUSES_UNFILLED_CALL``). Either is answered alike, and the
   text is the same each time the same call is refused, so that a resumed
   activation's windows are the ones it sent before.

   Failure modes:

   - **The activation failed** where the call should be answered, as before.
   - **The answer a context of a type of the engine's own**, which no
     workflow declares.
   - **The fault or the node types offered left out**, so the model is told
     that it erred and not how.
   - **Text that differs between two refusals of one call**, such as an
     identifier, which a resumed activation's window would not match.

.. comp_req:: A window after a refusal holds the refusal where the output would be
   :id: CREQ_HOST_WINDOW_WITH_REFUSALS
   :derived_from: FEAT_REFUSAL_GIVEN_TO_MODEL
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a model's answer made a refused call and its other calls have outputs, Script host shall send the model a window composing the previous window, the answer, each call's contexts or arguments and each call's output or answer, in order.

   ``CREQ_HOST_NEXT_WINDOW`` says what follows an answer whose calls all
   have outputs; this says what follows one whose calls were not all made.
   A refused call stands where a performed one would, its arguments in
   place of its contexts and its answer in place of its output, so the
   refusal is part of what the model was sent, by reference, as any output
   is (``DEC_WINDOW_IS_A_CONTEXT``). The calls beside it are performed
   (``DEC_UNREFUSED_CALLS_PERFORMED``).

   Failure modes:

   - **The refusal sent to the provider and left out of the window**, so the
     record cannot say the model was given it.
   - **The calls beside a refused one not performed**, or performed out of
     the answer's order.
   - **The refused call left out of the turn sent back**, which a provider
     refuses as a result for a call nobody made.

.. comp_req:: A refused call is held with its exchange
   :id: CREQ_RUN_HOLDS_REFUSED_CALLS
   :derived_from: FEAT_REFUSAL_RESUMED
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: event
   :statement: When its caller reports an exchange holding refused calls, Workflow run shall hold each with that exchange, with the identifier and node type the model gave it, its arguments and its answer, in the order reported.

   ``DEC_RECORD_HOLDS_REFUSED_CALLS``. The arguments and the answer are
   contexts, brought into the run as the exchange's others are
   (``CREQ_RUN_HOLDS_EXCHANGES``), and refused like them when one shares an
   identifier with a different context the run holds
   (``CREQ_RUN_REFUSES_SHARED_CALL_IDENTIFIER``).

   Failure modes:

   - **A refused call dropped from its exchange**, so the record never sees
     it.
   - **Its contexts not brought into the run**, so a record written from it
     names contexts it does not hold.

.. comp_req:: The record writes each refused call
   :id: CREQ_RECORD_WRITES_REFUSED_CALLS
   :derived_from: FEAT_REFUSAL_RESUMED
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: ubiquitous
   :statement: Run record shall write each refused call an exchange holds with the identifier and node type the model gave it, its arguments and its answer.

   ``DEC_RECORD_HOLDS_REFUSED_CALLS``, beside ``CREQ_RECORD_HOLDS_EXCHANGES``.
   The record is of version 5 from here on.

   Failure modes:

   - **A refused call left out of the record**, and a resumed activation
     asks the model again from there.
   - **Its arguments written as a call's inputs**, which they may not be.

.. comp_req:: A resumed run holds each refused call its record holds
   :id: CREQ_RECORD_KEEPS_REFUSED_CALLS
   :derived_from: FEAT_REFUSAL_RESUMED
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: ubiquitous
   :statement: Run record shall resume a run holding each refused call its record holds, with its exchange and in the order recorded.

   The reading half of ``CREQ_RECORD_WRITES_REFUSED_CALLS``, beside
   ``CREQ_RECORD_KEEPS_EXCHANGES``.

   Failure modes:

   - **Refused calls read back in another order**, or with another
     exchange.
   - **A record of version 4 read as if it held none**, which
     ``CREQ_RECORD_KEEPS_EXCHANGES``'s version check refuses instead.

.. comp_req:: A resumed activation answers a refused call from its record
   :id: CREQ_HOST_REFUSAL_FROM_RECORD
   :derived_from: FEAT_REFUSAL_RESUMED
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a resumed activation reaches an exchange its record holds refused calls for, Script host shall answer each of those calls with the answer the record holds, without sending a request.

   ``DEC_SCRIPT_REPLAYED_FROM_ITS_RECORD``. The exchange itself is answered
   from the record already (``CREQ_HOST_ANSWERS_FROM_RECORD``); this gives
   its refused calls the answers they had, so the window after them is the
   one the model was sent.

   Failure modes:

   - **The refusal made again rather than read**, with a new identifier, so
     the record holds two contexts for one answer.
   - **A request sent** for an exchange the record holds.

.. comp_req:: A refused call is sent back as the model made it
   :id: CREQ_ROSTER_REFUSED_CALL_AS_MADE
   :derived_from: FEAT_REFUSAL_GIVEN_TO_MODEL
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: ubiquitous
   :statement: Model roster shall send a refused call given with a model's answer with the identifier, name and arguments the model sent, whatever the arguments' shape, and its answer as that call's result.

   ``EVD_GENAI_ECHOES_ANY_ARGUMENTS``: ``genai`` sends a call's arguments as
   given in either provider's format, and a call's result as plain text.
   ``CREQ_ROSTER_PARTS_AS_MESSAGES`` says each part goes as the message it
   is; this says a refused call goes as the model made it, which a call's
   contexts cannot hold.

   Failure modes:

   - **The arguments sent as an object of strings** they were not, or as
     nothing.
   - **The refused call left out of the turn**, leaving its result
     answering no call.
