============================================
Decisions about the engine explaining itself
============================================

How Agconflo tells a model what it did with what the model asked of it
(``STKH_REFLECTION``). The first is what a model is told when a call it made
is refused, which used to end its activation instead.

.. dec:: A refused call is answered with why, and the activation goes on
   :id: DEC_REFUSED_CALL_ANSWERED
   :dec_status: accepted
   :decided_on: 2026-10-08
   :supersedes: DEC_MALFORMED_CALL_FAILS
   :supported_by: EVD_GENAI_ECHOES_ANY_ARGUMENTS, EVD_WEAK_MODEL_KEPT_TO_ITS_TOOLS
   :statement: Agconflo shall answer a call it refuses with a context naming the node type called and the fault, in place of that call's output, and go on with the model's activation.

   Every refusal of a call alike, whoever finds it: the script host for a
   node type not offered, arguments that are not an object, a parameter not
   declared, a required one missing or a value that is not a string, and the
   run for what it refuses of a reported call. The answer is a text context
   of the type of the prompt the model was sent, made in the activation as
   its window and offer are (``DEC_WINDOW_IS_A_CONTEXT``,
   ``DEC_TOOLS_OFFERED_AS_CONTEXTS``), and it is part of the next window as
   a call's output is. So the refusal has a place in the run, which is what
   ``DEC_MALFORMED_CALL_FAILS`` wanted of it before sending one back. Its
   text names the node type called, the fault, and the node types the step
   may call, and nothing else.

   The model's turn goes back as it came, the refused call included, and
   the refusal reaches the model as any output does: ``genai`` sends both
   unchanged in either provider's format (``EVD_GENAI_ECHOES_ANY_ARGUMENTS``).
   A model that keeps making a refused call is stopped where any model is,
   at the activation's model call limit.

   Failing the activation was the decision this supersedes. A refused call
   is rare (``EVD_WEAK_MODEL_KEPT_TO_ITS_TOOLS``), and when one came it
   ended the run, the work of every activation before it included. A type
   of context of the engine's own for a refusal was the other shape: a type
   Agconflo ships and a workflow's types are not. A provider's mark for an
   error was a third, and ``genai`` sends none.

.. dec:: The calls an answer makes that are not refused are performed
   :id: DEC_UNREFUSED_CALLS_PERFORMED
   :dec_status: accepted
   :decided_on: 2026-10-08
   :statement: Agconflo shall perform each call a model's answer makes that it does not refuse, whichever other calls of that answer it refuses.

   An answer may make several calls, and one of them may be refused. The
   others are performed in the order the answer gives them, each answered
   with its output, and the refused one with its refusal
   (``DEC_REFUSED_CALL_ANSWERED``).

   Performing none of them when one is refused was the alternative: every
   call checked before any is performed, as ``DEC_MALFORMED_CALL_FAILS``
   had it. Its reason was that a failed activation left the calls it had
   performed paid for and unused. An activation that goes on uses them, and
   a refusal the run makes comes after the calls before it were performed
   in any case.

.. dec:: A record holds each refused call as the model made it
   :id: DEC_RECORD_HOLDS_REFUSED_CALLS
   :dec_status: accepted
   :decided_on: 2026-10-08
   :supported_by: EVD_GENAI_ECHOES_ANY_ARGUMENTS
   :statement: Agconflo shall record each call an exchange's answer makes that the script host refuses, with the node type called and the arguments as the model sent them, and the context it was answered with.

   A resumed activation answers each request its record holds from the
   record (``DEC_SCRIPT_REPLAYED_FROM_ITS_RECORD``), and sends the model the
   turns before it again. A refused call is in those turns, and its
   arguments may not be an object of strings, so they cannot be made into a
   call's contexts: they are kept as a text context holding the JSON the
   model sent, beside the node type it named and the refusal. A call the run
   refuses was reported whole and is refused again the same way when it is
   reported again, so it needs nothing more.

   The record's version moves from 4 to 5, and a record of version 4 is
   refused naming its version, as a record of any other version is. Working
   out a refusal again on resume was the alternative, and cannot be done
   without the arguments, which the record did not hold.
