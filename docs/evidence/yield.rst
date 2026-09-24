===============================
Evidence about a model yielding
===============================

Measurements taken for ``STKH_MODEL_YIELDS`` and ``STKH_TOOLS_AS_NODES`` before
anything was specified for them: how ``genai`` represents a tool call, whether a
conversation waiting on one survives being written down, what the local model
does with one, what a Lua script can be held suspended through, and what the
workflow reader does with a declaration it does not know.

Each was taken in a scratch crate against ``agconflo-core`` at ``282acc2`` and
the dependency versions this workspace pins - ``genai`` 0.7.0-beta.24, ``mlua``
0.12.1 with Lua 5.5 - and each body says how to take it again. The stub is a
loopback HTTP server answering in OpenAI's chat-completions format or
Anthropic's messages format with replies scripted per request, as the tests'
own stub does. The local model is ``qwen3.8-27b-ridge`` on LM Studio, with a
context of 15,104 tokens, the one model it had loaded.

.. evd:: A tool call survives being written down mid-conversation
   :id: EVD_GENAI_TOOL_CALL_ROUND_TRIP
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: Through genai against a stub, a tool call came back as an id, a name and JSON arguments in both formats, and a conversation saved as JSON and read back sent a continuation identical to the live one in both.

   One tool, ``lookup``, with a required string parameter ``query``, offered with
   a user message. The stub answered with a call - OpenAI's ``tool_calls`` with
   arguments as a JSON string, Anthropic's ``tool_use`` block beside a text block
   "Let me look." - and ``ChatResponse::tool_calls`` gave
   ``{call_id, fn_name, fn_arguments}`` for each, the arguments parsed to JSON.
   ``into_assistant_message_for_tool_use`` kept Anthropic's text beside the call.

   The conversation - the user message, that assistant message and a
   ``ToolResponse`` carrying the call's id - was serialised with ``serde_json``
   (552 bytes for OpenAI's, 578 for Anthropic's), read back, and sent; the same
   conversation was sent live. The two request bodies the stub recorded were
   equal as JSON in each format, and re-serialising what was read back gave the
   same text.

   What genai adds to the request is protocol, not text: the tool as
   ``{"type": "function", ...}`` with ``"strict": false`` for OpenAI, as
   ``input_schema`` for Anthropic, an empty ``content`` on OpenAI's assistant
   turn, and Anthropic's ``max_tokens``. The serialised form is genai's own
   (``"role": "Assistant"``, ``{"ToolCall": ...}``), a pre-release crate's
   internal layout rather than either provider's.

   OpenAI's final answer came back trimmed and Anthropic's did not, as
   ``EVD_GENAI_OPENAI_TRIMS`` found for plain answers.

.. evd:: The local model yields and continues from a saved conversation
   :id: EVD_LOCAL_MODEL_YIELDS
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: qwen3.8-27b-ridge on LM Studio called a declared tool in 4.3 s and, continued from the conversation saved as text and read back, answered from the tool's result in 2.6 s.

   Asked to find what the codeword "amber-7" means with ``lookup``, it answered
   with one call, ``lookup({"query": "amber-7"})``, a 32-character call id of
   LM Studio's own, and its reasoning in a separate ``reasoning_content`` field
   of the response - whether genai carries that into the assistant message was
   not looked at. The tool's result, "amber-7 means: the
   harbour is closed until dawn.", was appended, the conversation serialised
   and read back as in ``EVD_GENAI_TOOL_CALL_ROUND_TRIP``, and sent. It answered
   "The codeword 'amber-7' means that the harbour is closed until dawn." with no
   further call.

.. evd:: A conversation sent flat makes the model call again
   :id: EVD_FLAT_WINDOW_RECALLS
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: After a tool call, qwen3.8-27b-ridge answered from the result 5 of 5 times given the provider's messages, and 1 of 5 given the same window as one user message, calling the tool again 4 times.

   Five trials each, over three codewords in turn, the tool still offered in
   both. Given the assistant turn holding the call and a tool message with the
   result, all five answered from the result, in 4.7 s together. Given one user
   message holding the prompt, the call as its name and arguments, and the
   result, joined by blank lines as a composition renders, one answered and four
   called ``lookup`` again with the same argument, in 20.0 s together.

   So the window after a call cannot be sent as a single rendering, as a prompt
   is now: each part has to go as the message it is. Five trials of one local
   model show the direction, not a rate; a model trained on the providers'
   formats is the expected case for it rather than the exception.

.. evd:: A script can be held suspended while a callee runs
   :id: EVD_LUA_SCRIPT_SUSPENDED_FOR_A_CALLEE
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: A Lua script awaiting a host function stayed suspended while a second script ran in a state of its own, each charged only its own instructions, 10000 against 40000. A callee over its limit reached the caller as an error.

   Both states built as the host builds one - named libraries, a memory limit,
   an instruction hook on the thread every thousand instructions - on a
   single-threaded tokio runtime. The first script called an asynchronous host
   function which created the second state, ran its script to the end and
   returned its result; the first then counted to 5,000 and returned. Its hook
   had counted about 10,000 instructions and the callee's about 40,000, so
   neither charged the other. A callee looping for ever was stopped by its own
   limit at about 51,000, and the caller's call raised a callback error carrying
   the callee's "instruction limit".

.. evd:: A script parked mid-call can be dropped
   :id: EVD_LUA_PARKED_SCRIPT_DROPPED
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: A script suspended in an async host function whose future was dropped, as when a run parks on a person mid-call, raised no panic, and its state held the same memory in each of three rounds.

   The host function awaited a future that never completes; the script's future
   was raced against a 20 ms timer and dropped when the timer won. The thread
   then reported itself resumable, the state held 31,468 bytes, and dropping the
   thread and the state raised nothing, in each of three rounds.

.. evd:: The workflow reader keeps a calls key it does not read
   :id: EVD_READER_KEEPS_UNKNOWN_CALLS_KEY
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: The workflow reader accepted a calls key on an instance and on a node type without a word, and the writer kept it on the instance when writing the definition back.

   ``read_workflow`` given ``calls = ["lookup"]`` on an entry instance returned
   the definition, and ``write_workflow`` into the document it came from left the
   key where it was, as ``CREQ_WRITER_KEEPS_UNREAD`` says of keys the model does
   not name. ``read_node_types`` given the same key on a node type returned the
   declarations. Declaring calls is therefore a change to the format either way,
   and a misspelt key declares nothing without a word.

.. evd:: Script order still differs between processes
   :id: EVD_LUA_ORDER_REMEASURED
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: Re-measured before building on it: pairs visited twelve string keys in a different order in each of three processes, and a table's address differed in each, as EVD_LUA_ORDER_PER_PROCESS found.

   A script filling a table from twelve names with ``ipairs``, then listing its
   keys with ``pairs`` and printing ``tostring`` of the table, run in three
   processes. A script that builds its prompt from either would send a different
   prompt when run again in another process, which is what replaying an
   activation from its record would do.

.. evd:: Tool names the providers accept
   :id: EVD_TOOL_NAME_RULES
   :evd_kind: vendor_doc
   :observed_on: 2026-09-24
   :observation: Anthropic documents a tool's name as matching ^[a-zA-Z0-9_-]{1,128}$ and OpenAI a function's as letters, digits, underscores and dashes up to 64 characters. Their intersection is ^[a-zA-Z0-9_-]{1,64}$.

   Anthropic's from its "Define tools" page on platform.claude.com, OpenAI's
   from the published ``openai/openai-openapi`` specification, whose
   function-name description reads "Must be a-z, A-Z, 0-9, or contain
   underscores and dashes, with a maximum length of 64". Neither was measured:
   there is no key for either provider here. A node type's name is a TOML key,
   which may hold any character, so a node type offered as a tool can be one
   either provider would refuse.

.. evd:: LM Studio does not check a tool's name
   :id: EVD_LM_STUDIO_TOOL_NAMES_UNCHECKED
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: LM Studio accepted a tool named "look up.v2", outside both providers' documented rules, and qwen3.8-27b-ridge called the tool by that name.

   So a workflow checked only against the local model would pass with a name a
   hosted provider refuses, and learn so on its first paid call.

.. evd:: genai passes malformed calls through
   :id: EVD_GENAI_PASSES_MALFORMED_CALLS
   :evd_kind: measurement
   :observed_on: 2026-09-24
   :observation: genai failed the whole response for tool arguments that were not JSON or were empty, and passed through a name never offered, a number for a string, a bare string for an object and two calls in one turn.

   Six OpenAI-format replies from the stub, each answering a request offering
   only ``lookup``. Arguments ``{query: amber`` and an empty string each failed
   ``exec_chat`` with a serde error and no response. A call to
   ``delete_everything``, ``{"query": 7}`` and ``"amber-7"`` each came back as a
   ``ToolCall`` as sent, and two calls in one turn as two. Whatever is checked
   about a call - that its name was offered, that its arguments fill the
   parameters - has to be checked by Agconflo.
