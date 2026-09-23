===============
Calling a model
===============

A node's script calling a language model: which model answers, what it is
shown, what comes back, and what bounds it. Every requirement here derives from
a goal in ``stakeholder/execution`` or ``stakeholder/context``, and each is
written against the decisions in ``decisions/models``.

What a call does not do here is what keeps this a slice. It sends one message
and no system prompt, offers no tools, streams nothing, and is not recorded
anywhere but in the contexts it takes and gives. A call is something a script
does during its activation, so everything already true of an activation - the
fresh state, the environment, the limits, the run's refusal of its output -
holds unchanged.

Each requirement was checked by hand against the question no rule can ask:
could this be false while its parent is true? The body of each says how.
Three statements are ``ubiquitous`` - which model answers, what it is shown and
what comes back are true of every call - and two ``unwanted``.

The feature's architecture closes the file. It realises all five requirements
and names the components they are divided between, which are defined in
``components/models``.

.. feat_req:: A script's call goes to the model its caller chose for the role
   :id: FEAT_MODEL_BY_ROLE
   :derived_from: STKH_PROVIDER_CHOICE
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall send each model call a script makes to the model its caller mapped the call's role to when starting the run.

   The parent asks that one workflow run against different providers. A script
   is part of the workflow, so what a script names cannot be a provider's model
   (``DEC_MODELS_BY_ROLE``): it names a role, and the caller says which model
   plays it.

   It can be false while the parent holds. A workflow whose scripts name
   ``openai::gpt-something`` does run against a different provider - once every
   script naming it has been edited - and the parent, read loosely, is met. What
   this adds is that the change happens where the run starts, and nowhere in
   the workflow.

   The test is the parent's claim made concrete: the same workflow and the same
   scripts, run twice with two mappings, reach two providers' formats.

.. feat_req:: A model is shown exactly the prompt it was given
   :id: FEAT_MODEL_WINDOW_IS_THE_PROMPT
   :derived_from: STKH_EXPLICIT_CONTEXT
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall send a model the rendering of the prompt context a script passes it as the call's only content.

   The parent gives a node exactly the contexts wired to it. A model call is the
   moment those contexts, assembled, become a window, and this says the window
   is exactly the assembly and nothing the engine chose to add.

   It can be false while the parent holds. A node given exactly what was wired to
   it can still have its call prefixed with a system prompt the engine wrote,
   decorated with instructions, or trimmed to fit - and the question "what was in
   this call's window?" then has an answer nobody wired. Measured, ``genai``
   adds parameters and no text (``EVD_GENAI_TWO_FORMATS_EXACT``), so what
   remains to hold is the engine's own restraint.

   Taking the prompt as a context rather than a string is what makes the window
   nameable afterwards (``DEC_PROMPT_IS_A_CONTEXT``).

.. feat_req:: A model's answer is a context of its own
   :id: FEAT_MODEL_ANSWER_IS_A_CONTEXT
   :derived_from: STKH_PROVENANCE
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall give a script a model's answer as a new context.

   The parent records which context each byte of a node's input came from. A
   model's answer is the first bytes in this project that no node wrote, and
   the ones whose origin matters most.

   It can be false while the parent holds. A host returning the answer as a Lua
   string meets the parent for every node downstream - they receive contexts -
   while the node that called the model splices the answer into its output
   indistinguishably from its own text. As a context the answer has an
   identifier, and a script composing it into its output holds it by reference,
   so the model's bytes stay the model's (``DEC_PROMPT_IS_A_CONTEXT``).

.. feat_req:: A script's model calls are bounded
   :id: FEAT_MODEL_CALLS_LIMITED
   :derived_from: STKH_STEP_BUDGET
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a node's script makes more model calls than its limit allows, then Agconflo shall fail that activation.

   The parent stops a run that exceeds its step budget, because a runaway loop
   through a model spends real money. A step is an activation, and one
   activation was measured making 2000 calls (``EVD_MODEL_CALLS_UNLIMITED``).

   It can be false while the parent holds, exactly as measured: the run stopped
   at its budget every time, having counted 2000 calls as one step. With calls
   limited per activation (``DEC_MODEL_CALLS_COUNTED``), the budget and the
   limit together bound what a run can spend.

.. feat_req:: A failed model call says which failure it was
   :id: FEAT_MODEL_FAILURE_CARRIED
   :derived_from: STKH_TYPED_FAILURE
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a model call fails, then Agconflo shall fail that activation carrying the call's role and what the provider answered.

   The parent reports which failure occurred when a node fails, and a model
   call fails for reasons its caller acts on differently: a key that has
   expired, a provider that is overloaded, a role nobody mapped.

   It can be false while the parent holds. A failure reported as the script's
   own error, carrying ``genai``'s message, has said a failure occurred and made
   its caller read prose to learn which - where the status was a value all
   along (``EVD_GENAI_ERROR_STATUS``). And a failure that does not name the role
   leaves a caller mapping several not knowing which mapping to fix.

   A call to a role mapped to nothing is among these, and fails without reaching
   any provider.

.. feat_arch:: Calling a model splits into a model roster and the script host
   :id: ARCH_MODELS
   :realises: FEAT_MODEL_BY_ROLE, FEAT_MODEL_WINDOW_IS_THE_PROMPT, FEAT_MODEL_ANSWER_IS_A_CONTEXT, FEAT_MODEL_CALLS_LIMITED, FEAT_MODEL_FAILURE_CARRIED
   :uses: COMP_MODEL_ROSTER, COMP_SCRIPT_HOST
   :statement: Agconflo shall allocate calling a model to the model roster and the script host.

   Two components, each answerable for what the other cannot guarantee:

   - The model roster answers for a call: which model a role reaches, what goes
     on the wire, and what a provider's failure becomes. It knows nothing of
     scripts or activations, and a call made through it is the same call
     whoever made it.
   - The script host answers for what a script may do with calls: that its
     prompt is a context, that its answer comes back as one, how many calls an
     activation may make, and that a failed call ends the activation. Each of
     those is a statement about one activation, which is the host's unit.

   ``FEAT_MODEL_FAILURE_CARRIED`` is the requirement split across both, and the
   split is the division: the roster says what failed and how, and the host says
   that a failure ends the activation.

   The decisions this is built against are named here rather than linked:

   - ``DEC_MODELS_THROUGH_GENAI``: the roster calls through ``genai``.
   - ``DEC_MODELS_IN_LUA_CRATE``: both live in ``agconflo-lua``.
   - ``DEC_MODELS_BY_ROLE``: a script names a role; the caller's roster maps it.
   - ``DEC_BEHAVIOUR_ASYNC``: a call is awaited, so the host runs scripts
     asynchronously.
   - ``DEC_HOOK_ON_THE_THREAD``: the instruction limit is set on the thread a
     script runs in, since the state's hook does not reach it there.
   - ``DEC_MODEL_CALLS_COUNTED``: calls are counted per activation.
   - ``DEC_PROMPT_IS_A_CONTEXT``: a prompt is a context and its rendering is all
     that is sent.
