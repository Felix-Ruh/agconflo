=============================
Components of calling a model
=============================

The component ``ARCH_MODELS`` adds, and the requirements allocated to it and to
the script host for calling a model. Each title is the grammatical subject of
the requirements allocated to it, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

.. comp:: Model roster
   :id: COMP_MODEL_ROSTER
   :crate: agconflo-lua

   The caller's mapping from role to model, and the client that reaches them.
   It turns one call - a role and a prompt - into a request to the right
   provider, and the provider's answer or failure into a value.

   Nothing in it knows about scripts or activations. That is what lets its
   requirements be tested with no script at all, against a stub.

.. comp_req:: A role reaches the model it is mapped to
   :id: CREQ_ROSTER_ROLE_TO_MODEL
   :derived_from: FEAT_MODEL_BY_ROLE
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: ubiquitous
   :statement: Model roster shall send a call for a role to the model that role is mapped to.

   Failure modes:

   - **Every role sent to one model.** A roster mapping two roles to two
     providers then reaches one, and a test with a single mapping passes.
   - **The role taken as a model name.** A script's role then reaches whichever
     provider ``genai`` guesses from the name, which is the vendor choice the
     role exists to keep out of the workflow.
   - **The mapping read once and kept.** A caller changing the mapping between
     runs, which is the whole of provider choice, changes nothing.

.. comp_req:: A call sends its prompt and nothing else
   :id: CREQ_ROSTER_ONE_MESSAGE
   :derived_from: FEAT_MODEL_WINDOW_IS_THE_PROMPT
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: ubiquitous
   :statement: Model roster shall send a call's prompt as one user message holding the prompt's rendering and no other content.

   Measured, ``genai`` adds parameters and no text
   (``EVD_GENAI_TWO_FORMATS_EXACT``), so this is a requirement on the roster's
   own restraint.

   Failure modes:

   - **A system prompt added.** The window then holds text no wire brought.
   - **The prompt trimmed, normalised or re-encoded.** A trailing space or a
     line ending changed is a different window, and a model can answer it
     differently.
   - **The prompt's parts sent as separate messages.** A composition renders as
     one text, and splitting it into messages changes what the model sees at
     the seams.

.. comp_req:: A call to an unmapped role fails without reaching a provider
   :id: CREQ_ROSTER_UNMAPPED_ROLE
   :derived_from: FEAT_MODEL_FAILURE_CARRIED
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: unwanted
   :statement: If a call names a role that is mapped to no model, then Model roster shall fail that call naming the role without reaching any provider.

   Failure modes:

   - **The role sent as a model name.** The call reaches whatever provider the
     name resembles, is billed there, and fails or - worse - succeeds.
   - **A default model used.** A misspelt role then quietly answers from a
     model nobody chose.
   - **Failed without the role.** A caller mapping several cannot tell which to
     add.

.. comp_req:: A provider's failure is carried as values
   :id: CREQ_ROSTER_PROVIDER_FAILURE
   :derived_from: FEAT_MODEL_FAILURE_CARRIED
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: unwanted
   :statement: If a provider fails a call, then Model roster shall fail that call carrying the role, the provider's status when it gave one, and the failure's message.

   The status is a value ``genai`` already holds (``EVD_GENAI_ERROR_STATUS``),
   and a provider that could not be reached gave none.

   Failure modes:

   - **The status folded into the message.** A caller telling an expired key
     from an overloaded provider parses prose.
   - **A provider that could not be reached given a status.** There was no
     answer, and a made-up status is a claim about one.
   - **The message dropped.** The status says which class of failure and the
     message says which instance of it.

.. comp_req:: A model's answer comes back as a new context
   :id: CREQ_HOST_MODEL_ANSWER
   :derived_from: FEAT_MODEL_ANSWER_IS_A_CONTEXT
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: ubiquitous
   :statement: Script host shall give a script a model's answer as a new context of the type the script names for it, or of its output's declared type when it names none.

   Issued from the run's one identifier source, like every context a script
   makes, so it is new to the run (``CREQ_RUN_REFUSES_SHARED_OUTPUT_IDENTIFIER``).

   Failure modes:

   - **The answer returned as a string.** The model's bytes enter the output as
     the script's.
   - **The answer's type chosen by the host.** A script composing answers of two
     kinds cannot tell them apart by type.
   - **The answer's text altered.** Trimmed or normalised, it is no longer what
     the model said.

.. comp_req:: A prompt that is not a context is refused
   :id: CREQ_HOST_PROMPT_IS_A_CONTEXT
   :derived_from: FEAT_MODEL_WINDOW_IS_THE_PROMPT
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a script passes a model call a prompt that is not a context, then Script host shall fail that activation as a script error without making the call.

   Failure modes:

   - **A string accepted and sent.** The window is then text the run never held
     (``DEC_PROMPT_IS_A_CONTEXT``).
   - **The call made and then failed.** A call once made is spent.

.. comp_req:: A script over its model call limit fails before the call
   :id: CREQ_HOST_MODEL_CALL_LIMIT
   :derived_from: FEAT_MODEL_CALLS_LIMITED
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a script makes more model calls than its limit, then Script host shall fail that activation as having exceeded its model call limit without making that call.

   Failure modes:

   - **No limit.** The measured shape: 2000 calls in one activation.
   - **The call made and then counted.** One call over the limit on every
     activation that reaches it, each one billed.
   - **Counted per run.** One node's calls exhaust what the rest were given.
   - **Reported as a script error.** A caller cannot tell a script that called
     too often from one that broke.

.. comp_req:: A failed model call ends the activation with its failure
   :id: CREQ_HOST_MODEL_FAILURE
   :derived_from: FEAT_MODEL_FAILURE_CARRIED
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a model call fails, then Script host shall fail that activation carrying the call's failure.

   A script cannot catch the failure (``CREQ_HOST_NO_CATCHING``), so it ends the
   activation like any other error.

   Failure modes:

   - **The failure handed to the script as a value it may ignore.** The script
     goes on with no answer, and its output says nothing of the failure.
   - **Reported as a script error.** The status and the role, which the roster
     carried as values, become prose.
