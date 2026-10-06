=======================
Decisions about caching
=======================

What Agconflo does so that a provider can serve a model call from what it
cached for an earlier one (``STKH_PROMPT_CACHE_REUSED``), and what it leaves
undone, on the evidence in ``evidence/caching``.

.. dec:: A call that may be continued is marked for the cache where it ends and where the call before it ended
   :id: DEC_CACHE_MARKED_BEFORE_A_CONTINUATION
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supersedes: DEC_PREFIX_STORE
   :supported_by: EVD_GENAI_CACHE_MARKS_BY_ADAPTER, EVD_CACHE_MARK_ON_A_TOOL_RESULT, EVD_ANTHROPIC_CACHE_RULES, EVD_REPEATED_PREFIX_SERVED_FROM_CACHE
   :statement: Agconflo shall mark for the provider's cache the last message of a model call that offers node types to call and the last message of the call it continues, and mark no other message.

   While a model calls node types and goes on, each turn is sent the window
   of the turn before with the answer, the calls and their outputs after it
   (``CREQ_HOST_NEXT_WINDOW``). Every turn but the first begins with the
   whole of the one before, which is exactly what a cache serves. A provider
   that caches by itself serves it with nothing asked: 16,512 of 16,749
   tokens on a second call (``EVD_REPEATED_PREFIX_SERVED_FROM_CACHE``).
   Anthropic, and OpenAI from ``gpt-5.6``, cache only what a request marks
   (``EVD_ANTHROPIC_CACHE_RULES``), and the marks are what this decides.

   Two marks, because a provider finds an earlier entry by looking back a
   fixed number of blocks from a mark: 20 on Anthropic. An answer making ten
   calls adds 21 blocks - its text, the ten calls and their ten results - so
   a mark at the end alone would look back past where the turn before was
   written, and read nothing. A mark where the turn before ended matches its
   entry exactly however many calls came between. Two is half of the four a
   request may carry.

   On a call that offers node types, because such a call may be continued
   and one offering none never is. A mark writes the cache, at 1.25 times
   the input price on Anthropic, so marking every call would have each step
   that asks a model once, offering it nothing, pay for an entry nothing
   reads. The last turn of a continued activation still writes one nothing
   reads: which turn is last is the model's to say, after the request has
   gone.

   The engine marks, and a workflow writes nothing about caching. A script
   asking for a mark would carry one provider's way of caching into the
   workflow, which ``STKH_PROVIDER_CHOICE`` keeps at the engine.

   What it gives each provider, measured
   (``EVD_GENAI_CACHE_MARKS_BY_ADAPTER``, ``EVD_CACHE_MARK_ON_A_TOOL_RESULT``):

   - Anthropic through ``genai``'s own adapter: both marks.
   - OpenAI from ``gpt-5.6``: a mark on the user's prompt, which the second
     turn carries, and none on a call's result, which ``genai`` drops. From
     the third turn on both marks fall on results, and what OpenAI then reads
     is not known here.
   - Through OpenRouter: no mark, dropped by ``genai``.
     ``DEC_CACHE_MARKS_THROUGH_GENAI`` says what that leaves a person to do.
   - A provider caching by itself: no change in what is sent.

   Not adopted:

   - **Reordering a call's contexts so that its beginning repeats an
     earlier call's**, which ``DEC_PREFIX_STORE`` decided. It has the engine
     change the order of a window, and order is meaning
     (``FEAT_CONTEXT_PARTS``) and the node's to choose. It moved only contexts
     a node type declares order-free, which no node type can declare. And a
     beginning made to repeat is still not read from a cache that needs a
     mark without one.
   - **Marking where a prompt's shared part ends** - a node type's
     instructions, ahead of its inputs - so that the next activation of the
     node type reads it. That is the rest of ``STKH_PROMPT_CACHE_REUSED``,
     left undone. ``genai`` sets a mark on a whole message, and a prompt is
     one message, so there is no place inside it to mark. Sending a prompt's
     parts as messages of their own would give each a place, and in OpenAI's
     format would frame each with its role, text the window does not hold
     (``FEAT_MODEL_WINDOW_IS_THE_PROMPT``). Marks on a message's parts, in
     ``genai``, are what would let it be done; a provider that caches by
     itself serves it already.
   - **The option ``genai`` sets on a whole request**: measured to reach
     Anthropic as nothing for a window with no system block and no tool, and
     other providers as a field of OpenAI's. Anthropic's own field on the
     request, which marks the last block and moves forward with the
     conversation, would do what the mark at the end does; ``genai`` sends no
     such field.

   A mark lives as long as ``genai``'s ``Ephemeral`` says: five minutes on
   Anthropic, renewed at every read. A turn follows the one before in the
   time a call takes. An hour costs twice the input price to write and still
   covers no call that waits on a person, which can wait days. Below a
   model's minimum length nothing is cached and nothing fails, so no length
   is checked before marking.

.. dec:: Cache marks reach a provider through genai's own adapters
   :id: DEC_CACHE_MARKS_THROUGH_GENAI
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supported_by: EVD_GENAI_CACHE_MARKS_BY_ADAPTER, EVD_GENAI_TWO_FORMATS_EXACT
   :statement: Agconflo shall set a cache mark through genai's message options rather than write any part of a request's body itself.

   Through OpenRouter ``genai`` drops a mark. Its ``extra_body`` could put
   one back, by writing the request's messages over what the adapter built,
   and then what reaches a provider would be Agconflo's writing rather than
   ``genai``'s: what was measured of exactly what is sent
   (``EVD_GENAI_TWO_FORMATS_EXACT``) would be about a request nobody sends.

   So a model whose provider caches only what is marked is mapped through
   that provider's own adapter, ``anthropic::claude-...``, as the README
   says. Mapped through OpenRouter it runs, uncached.

   A warning from ``agconflo check`` for such a mapping was the other way. It
   would guess a provider's way of caching from a model's name, which changes
   with releases: ``genai``'s own list counts OpenAI's models as marking from
   ``gpt-5.6`` on.
