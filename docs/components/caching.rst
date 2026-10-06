=================================
Components of caching a next turn
=================================

The requirements ``ARCH_CONTINUATION_CACHED`` allocates to the model roster,
defined in ``components/models``. Its title is the grammatical subject of
both, and the gate in ``scripts/gates`` refuses a component requirement
whose subject is anything else.

What a mark is, and how a provider reads one, is
``DEC_CACHE_MARKED_BEFORE_A_CONTINUATION``'s. A mark changes no text a
message carries, which ``CREQ_ROSTER_CONTEXTS_WHOLE`` holds of every request
already.

.. comp_req:: A window offering node types is marked at its end and where the turn before ended
   :id: CREQ_ROSTER_MARKS_CONTINUATION
   :derived_from: FEAT_CONTINUATION_CACHED
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: event
   :statement: When a call offers node types, Model roster shall mark for the provider's cache the last message of its window and the message before the window's last answer, and no other.

   The message before the last answer is the one the turn before ended
   with: the prompt on the second turn, the last call's result after that.
   A first turn has no answer in its window, and only its last message is
   marked.

   Failure modes:

   - **Only the last message marked**, and a turn after an answer making ten
     calls or more reading nothing, its earlier entry beyond the 20 blocks
     Anthropic looks back (``EVD_ANTHROPIC_CACHE_RULES``).
   - **The answer marked** instead of what came before it, a place no
     earlier turn wrote an entry.
   - **A third message marked**, a write paid for that no later turn reads.
   - **A mark that does not reach the format that needs it**, after a change
     in ``genai``.

.. comp_req:: A window offering nothing is not marked
   :id: CREQ_ROSTER_UNOFFERED_UNMARKED
   :derived_from: FEAT_CONTINUATION_CACHED
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: event
   :statement: When a call offers no node types, Model roster shall mark none of its messages for the provider's cache.

   A call offering nothing is never continued, so a mark on it writes an
   entry no later turn of the activation reads, at 1.25 times the input
   price on Anthropic.

   Failure modes:

   - **A call offering nothing marked**, and every step that asks a model
     once paying for a write.
   - **The option set on the whole request** instead of on a message, which
     reaches providers that cache by themselves as a field of OpenAI's
     (``EVD_GENAI_CACHE_MARKS_BY_ADAPTER``).
