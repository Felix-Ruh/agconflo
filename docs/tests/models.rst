================
Model test cases
================

How each requirement in ``components/models`` is to be checked, with one
feature-level case where the claim is about a whole run. Results are never
written here: they are imported from the test runner.

No case reaches a provider. Each points the roster at a stub on the loopback
interface that answers in OpenAI's or Anthropic's format and records what it was
sent, which is how the measurements behind this feature were taken
(``EVD_GENAI_TWO_FORMATS_EXACT``). What reaches the stub is what would reach a
provider; that a provider answers alike is outside what a test here can show,
and no key is needed or held.

A case's id is the path of the Rust test that implements it, uppercased. The
roster's cases live in the module ``models``, which ``agconflo-core`` does not
have, so no id can repeat one of its.

Every failure mode listed in ``components/models`` is named by the case that
catches it. Two cases verify requirements of ``components/behaviour``: the
limits, which were measured not holding once a script had awaited a call
unless they are set where the script runs (``EVD_LUA_HOOK_PER_THREAD``).

.. test_case:: Each role reaches the model it is mapped to
   :id: TEST_MODELS_ROLES_REACH_THEIR_MODELS
   :verifies: CREQ_ROSTER_ROLE_TO_MODEL
   :test_kind: positive
   :coverage: partial

   A roster mapping two roles to two models of two providers. A call for each
   reaches its provider's path with its model's name, and a role that looks like
   a model name reaches the model it is mapped to rather than the one its name
   suggests.

   Catches: every role sent to one model; the role taken as a model name.

.. test_case:: A prompt is sent as one message and nothing else
   :id: TEST_MODELS_PROMPT_SENT_EXACTLY
   :verifies: CREQ_ROSTER_CONTEXTS_WHOLE
   :test_kind: property
   :coverage: partial

   For any prompt text - whitespace at either end, line endings of both kinds,
   characters outside ASCII - and for a prompt composed of several parts, the
   request holds exactly one message, from the user, whose text is the prompt's
   rendering byte for byte, and no system prompt.

   Catches: a system prompt added; the prompt trimmed, normalised or re-encoded.
   That the request holds one message is ``DEC_PROMPT_IS_A_CONTEXT``'s division
   of the text rather than the requirement's, and is asserted for the decision.

.. test_case:: A call to an unmapped role fails and reaches nothing
   :id: TEST_MODELS_UNMAPPED_ROLE_FAILS
   :verifies: CREQ_ROSTER_UNMAPPED_ROLE
   :test_kind: error_path
   :coverage: partial

   A call naming a role the roster does not map, including one spelled like a
   model the client would resolve on its own. The call fails naming the role,
   and the stub records no request.

   Catches: the role sent as a model name; a default model used; failed without
   the role.

.. test_case:: A provider's failure keeps its status and message
   :id: TEST_MODELS_PROVIDER_FAILURE_CARRIED
   :verifies: CREQ_ROSTER_PROVIDER_FAILURE
   :test_kind: error_path
   :coverage: partial

   Stubs answering 401 and 503. Each call fails carrying the role, the status as
   a value, and a message.

   Catches: the status folded into the message; the message dropped.

.. test_case:: A provider that cannot be reached gives no status
   :id: TEST_MODELS_UNREACHABLE_PROVIDER_HAS_NO_STATUS
   :verifies: CREQ_ROSTER_PROVIDER_FAILURE
   :test_kind: error_path
   :coverage: partial

   A roster pointed at a port nothing listens on. The call fails carrying the
   role and a message, and no status.

   Catches: a provider that could not be reached given a status.

.. test_case:: A model's answer comes back as a context of the type asked for
   :id: TEST_HOST_MODEL_ANSWER_IS_A_CONTEXT
   :verifies: CREQ_HOST_MODEL_ANSWER
   :test_kind: positive
   :coverage: partial

   A script calling a model twice, once naming a type for the answer and once
   not, and composing both answers into its output. The answers render exactly
   as the stub sent them, whitespace included, the first is of the type named
   and the second of the output's declared type, and the output is accepted.

   Catches: the answer returned as a string; its type chosen by the host; its
   text altered.

.. test_case:: A prompt that is not a context is refused before the call
   :id: TEST_HOST_PROMPT_MUST_BE_A_CONTEXT
   :verifies: CREQ_HOST_PROMPT_IS_A_CONTEXT
   :test_kind: error_path
   :coverage: partial

   A script passing a string as its prompt. The activation fails as a script
   error, and the stub records no request.

   Catches: a string accepted and sent; the call made and then failed.

.. test_case:: A script over its model call limit fails before the call
   :id: TEST_HOST_MODEL_CALL_LIMIT_HOLDS
   :verifies: CREQ_HOST_MODEL_CALL_LIMIT
   :test_kind: error_path
   :coverage: partial

   A limit of two calls. A script making three fails as having exceeded its
   model call limit, not as a script error, and the stub records two requests.
   A chain of two instances each making two calls completes, so the count is
   each activation's own.

   Catches: no limit; the call made and then counted; counted per run; reported
   as a script error.

.. test_case:: A failed model call ends the activation carrying the failure
   :id: TEST_HOST_MODEL_FAILURE_ENDS_THE_ACTIVATION
   :verifies: CREQ_HOST_MODEL_FAILURE
   :test_kind: error_path
   :coverage: partial

   A stub answering 503. The activation fails with the model failure - role,
   status and message as values - rather than as a script error, and nothing
   after it is performed.

   Catches: the failure handed to the script as a value it may ignore; reported
   as a script error.

.. test_case:: The instruction limit holds after a model call
   :id: TEST_HOST_INSTRUCTION_LIMIT_HOLDS_ACROSS_A_MODEL_CALL
   :verifies: CREQ_HOST_INSTRUCTION_LIMIT
   :test_kind: error_path
   :coverage: partial

   A script that calls a model and then loops for ever. It fails on the
   instruction limit within the time a test is allowed. Measured, a hook on the
   state would not have stopped it, and every earlier limit case would still
   have passed (``EVD_LUA_HOOK_PER_THREAD``).

.. test_case:: The memory limit holds after a model call
   :id: TEST_HOST_MEMORY_LIMIT_HOLDS_ACROSS_A_MODEL_CALL
   :verifies: CREQ_HOST_MEMORY_LIMIT
   :test_kind: error_path
   :coverage: partial

   A script that calls a model and then makes a string far larger than its
   memory limit. It fails on the memory limit.

.. test_case:: The same workflow reaches another provider by its caller's mapping alone
   :id: TEST_SCRIPTED_SAME_WORKFLOW_TWO_PROVIDERS
   :verifies: FEAT_MODEL_BY_ROLE
   :test_kind: positive
   :coverage: partial

   One workflow and one set of scripts calling the role ``drafting``, run twice:
   once with a roster mapping it to an OpenAI model and once to an Anthropic
   one. The first run's request reaches OpenAI's path and the second
   Anthropic's, and each result carries that provider's answer.

   The parent goal made concrete: nothing in the workflow changed.

.. test_case:: An answer keeps the whitespace the provider sent
   :id: TEST_MODELS_ANSWER_KEPT_AS_SENT
   :verifies: CREQ_ROSTER_ANSWER_AS_SENT
   :test_kind: error_path
   :coverage: partial

   A stub answering with whitespace at both ends, in OpenAI's format and in
   Anthropic's. Both calls return the answer exactly as sent.

   Catches: the answer taken from ``genai``'s reading, which is the measured
   defect for the OpenAI format. Not caught here, and recorded: several text
   blocks, and reasoning reported separately, since the stub sends neither.

.. test_case:: A scripted run hands its identifier source back advanced
   :id: TEST_SCRIPTED_SOURCE_HANDED_BACK_ADVANCED
   :verifies: FEAT_RUN_ONE_CONTEXT_PER_IDENTIFIER
   :test_kind: error_path
   :coverage: partial

   A scripted run lends the caller's identifier source to its scripts, because
   a function that awaits a model cannot borrow it. After the run, the next
   identifier the caller's source issues is none of those the run's contexts
   carry. Handed back reset, it would repeat the argument's and the run's own,
   and the caller's next run would hold two contexts under one identifier.
