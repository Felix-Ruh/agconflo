======================================================
Evidence about the engine explaining itself to a model
======================================================

Measurements taken for ``STKH_REFLECTION`` before anything was specified for
it: how a call the engine refuses could be answered to the model, and how
often a weak model makes such a call.

The genai measurement was taken in a scratch crate on the version this
workspace pins, ``genai`` 0.7.0-beta.24, against a loopback HTTP server that
records each request and answers in OpenAI's chat-completions format or
Anthropic's messages format. The model measurements went to
``deepseek/deepseek-v4.1-flash`` through OpenRouter, with a key the
maintainer allowed for these runs.

.. evd:: genai sends a call's arguments back as given, and its result as text
   :id: EVD_GENAI_ECHOES_ANY_ARGUMENTS
   :evd_kind: measurement
   :observed_on: 2026-10-08
   :observation: genai sent a model's turn back with each call's arguments as given, a string, a number, an array or an object, in OpenAI's format and Anthropic's, and each call's result as plain text with no error flag in either.

   One request per adapter and shape, ``openai::`` and ``anthropic::``: a
   user message, then the model's turn holding text and two calls, ``bash``
   with the arguments under test and ``read_file`` with ``{"path":
   "a.txt"}``, then a ``ToolResponse`` for each. In OpenAI's format the
   arguments went out as their JSON text - ``"arguments":"\"ls -la\""``,
   ``"42"``, ``"[\"ls\",\"-la\"]"``, ``"{\"path\":7}"`` - and each result as
   a ``tool`` message carrying ``tool_call_id``. In Anthropic's they went
   out as the ``input`` value itself - ``"ls -la"``, ``42``, ``["ls","-la"]``,
   ``{"path":7}`` - and each result as a ``tool_result`` block in a user
   message carrying ``tool_use_id``. Neither result carried ``is_error`` or
   any other mark.

   So a turn holding a call the engine refused is sent back as it came, and
   what the call is answered with reaches the model the way any call's
   output does. Whether Anthropic accepts an ``input`` that is not an object
   was not measured; its own models send an object, and a call whose
   arguments are not one comes from the OpenAI format
   (``EVD_GENAI_PASSES_MALFORMED_CALLS``).

.. evd:: Offered two tools, the weak model called no other in 101 calls
   :id: EVD_WEAK_MODEL_KEPT_TO_ITS_TOOLS
   :evd_kind: measurement
   :observed_on: 2026-10-08
   :observation: Offered read_file and write_file and asked to fix a project and check its tests pass, deepseek-v4.1-flash called no tool it was not offered in 20 first turns and in 10 conversations of up to 5 turns, 101 calls in all.

   The task named a project of two Python files and the command running
   its tests, which neither tool can run. The 20 first turns each read both
   files, 40 calls. The 10 conversations answered each call from a script -
   the two files with a bug in ``add``, ``written`` for a write - and went
   on until the model stopped calling or had taken 5 turns: 43 reads and 18
   writes. One conversation ended saying it could not run ``pytest`` with
   the tools it had; none reached for one it lacked, and none wrote a call
   out as text.

   It does not reproduce what version 3 of the record-goals workflow
   measured for #71, where the same model called ``bash`` 43 activations
   into a run, after reading two files at paths that do not exist, from a
   prompt many times this size. A call to a tool not offered is rare on a
   task this small, and when it comes, today it fails the activation and the
   run with it (``DEC_MALFORMED_CALL_FAILS``).
