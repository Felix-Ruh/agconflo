======================
Evidence about caching
======================

Measurements the caching of a model call rests on, taken on 2026-10-06 in a
throwaway crate outside the repository against ``genai`` 0.7.0-beta.24, the
version the workspace pins, with the repository at ``7857afd``.

Two of them reached no provider. ``genai`` was given a resolver sending every
model's requests to a stub on the loopback interface, keeping the adapter it
chose from the model's name, with a placeholder key, and the stub recorded
each request's path and body - the technique of
``EVD_GENAI_TWO_FORMATS_EXACT``. What reached the stub is what would have
reached a provider. One reached OpenRouter, with the maintainer's leave for
that run, and one is what Anthropic's documentation says.

.. evd:: genai sent a cache mark on a message through two of its adapters and dropped it through the rest
   :id: EVD_GENAI_CACHE_MARKS_BY_ADAPTER
   :evd_kind: measurement
   :observed_on: 2026-10-06
   :observation: genai 0.7.0-beta.24 sent a cache mark set on a message through its Anthropic adapter and, for gpt-5.6, its OpenAI one, and dropped it through OpenRouter and for every other model named, with no error.

   Each model was sent three requests: one user message with nothing set
   about caching; two user messages, the first marked with ``CacheControl``
   ``Ephemeral``; and one user message with ``Ephemeral`` set on the whole
   request instead. What went out:

   - ``anthropic::claude-sonnet-5``: the mark as ``cache_control`` on the
     marked message's text block, the message otherwise as before. The
     option on the whole request sent nothing at all.
   - ``openai::gpt-5.6``: the mark as ``prompt_cache_breakpoint`` on the
     marked message's block, with ``prompt_cache_options`` in explicit mode
     and a lifetime of 30 minutes. With nothing set about caching,
     ``prompt_cache_options`` in explicit mode was sent all the same; with
     the option on the whole request, nothing was.
   - ``open_router::anthropic/claude-sonnet-5``, ``openai::gpt-5.5``,
     ``openai::qwen3.8-27b-ridge`` and
     ``open_router::deepseek/deepseek-v4.1-flash``: no mark, the two
     messages sent as plain text. The option on the whole request became
     ``prompt_cache_retention: in_memory``, a field of OpenAI's API.

   So a mark is an option of a whole message. It is set through
   ``MessageOptions``, and none of ``genai``'s content parts carries one,
   read from its source; Anthropic's adapter puts it on a message's last
   block. A request whose prompt is one message has nowhere inside the prompt
   to mark.

   Anthropic's adapter puts the option on the whole request on the last
   system block, or on the last tool when there is no system block, and the
   requests here had neither. Where OpenRouter would have sent a marked
   request on to Anthropic, the mark never left the machine.

.. evd:: A cache mark on a continuation's last result reached Anthropic's format and not OpenAI's
   :id: EVD_CACHE_MARK_ON_A_TOOL_RESULT
   :evd_kind: measurement
   :observed_on: 2026-10-06
   :observation: On a window of a prompt, an answer making two calls and the two results, genai sent a mark set on the last result through its Anthropic adapter and dropped it for gpt-5.6, keeping only prompt_cache_options.

   Through Anthropic's adapter each result went as a user message of its
   own holding a ``tool_result`` block, and the last block carried
   ``cache_control``. For ``gpt-5.6`` each result went as a ``tool`` message
   whose content is a string, and the OpenAI adapter places a mark on a block
   of a list of them, so there was nowhere to put it. The four other model
   names sent what they sent without a mark.

.. evd:: Anthropic caches an exact beginning up to a marked block, looking back 20 blocks from each mark
   :id: EVD_ANTHROPIC_CACHE_RULES
   :evd_kind: vendor_doc
   :observed_on: 2026-10-06
   :observation: Anthropic's documentation says a hit needs the prompt identical up to a marked block, allows 4 marks a request, looks back at most 20 blocks from each, and caches nothing under 512 to 4096 tokens, without an error.

   Read at ``platform.claude.com/docs/en/build-with-claude/prompt-caching``,
   which carries no date of its own. Beside the observation:

   - A prompt is matched in the order tools, system, messages, and a result
     of a call is a block like any other that can be marked.
   - Of the 20 blocks looked at, the mark itself is the first; if none
     matches, looking stops.
   - The minimum depends on the model: 512 tokens for the newest, 1024 for
     several, up to 4096 for some. A shorter request marked for caching "will
     be processed without caching, and no error is returned".
   - An entry lives 5 minutes by default, or an hour if asked, measured from
     the start of the request that writes or reads it, and every read renews
     it at no cost.
   - Writing costs 1.25 times the input price for 5 minutes and 2 times for
     an hour; reading costs 0.1 times, and less for a few models.
   - A field ``cache_control`` on the request itself has Anthropic mark the
     last block that can be cached, moving forward as a conversation grows.
     ``genai`` sends no such field: its option on the whole request is the one
     ``EVD_GENAI_CACHE_MARKS_BY_ADAPTER`` saw go nowhere.
   - What was read and written is reported in ``cache_read_input_tokens`` and
     ``cache_creation_input_tokens``.

.. evd:: DeepSeek served a repeated 16k-token beginning from its cache with nothing asked
   :id: EVD_REPEATED_PREFIX_SERVED_FROM_CACHE
   :evd_kind: measurement
   :observed_on: 2026-10-06
   :observation: Two calls to deepseek-v4.1-flash through OpenRouter, alike but for their last sentence, had 0 and then 16512 of 16749 prompt tokens served from the provider's cache, with no cache option sent.

   Sent as the runner reaches a role mapped with an endpoint: ``genai``'s
   OpenAI adapter at OpenRouter's endpoint, the key from
   ``OPEN_ROUTER_API_KEY`` and every other key variable unset. The prompt was
   ``README.md`` and ``AGENTS.md``, 69,068 bytes, followed by a sentence asking
   for one word, a different word the second time, the calls five seconds
   apart.

   OpenRouter sent both to DeepSeek. The first call's prompt cost $0.00251
   and the second's $0.000085, and both reported ``cache_write_tokens`` 0. A
   cache that matches a repeated beginning by itself served it with nothing
   in the request about caching, so a mark set for Anthropic's sake, which
   ``EVD_GENAI_CACHE_MARKS_BY_ADAPTER`` saw dropped on this path, is not what
   it needs.

   One pair of calls to one provider: it shows that the regime is there, not
   how often it serves.
