===============================
Decisions about calling a model
===============================

How a node reaches a language model, what it may send, and what bounds it.
``DEC_RUN_IS_DRIVEN`` left open how a context reaches a provider and whether
behaviour is ever asynchronous; these settle both for the Lua host, and neither
reaches into the core.

Six rest on measurements recorded in ``evidence/models``, the last taken while
implementing. The first is superseded by
``DEC_DECISIONS_THROUGH_THEIR_ENDPOINT``, which keeps it for chat models and
adds decisions models beside them. Two are judgements, and say so: where the
code lives, and that a script names a role rather than a model. The second
is argued from ``STKH_PROVIDER_CHOICE`` itself, which is the strongest footing
a judgement can have and still not a measurement.

What they do not settle is named here. Nothing below decides whether a call
carries a system prompt or several messages, whether a script may use tools,
how long a call may take, or whether the pair of prompt and answer is recorded
anywhere but in the contexts themselves - that last is the run log's, which does
not exist yet.

.. dec:: A model is reached through genai
   :id: DEC_MODELS_THROUGH_GENAI
   :dec_status: superseded
   :decided_on: 2026-09-23
   :supported_by: EVD_GENAI_TWO_FORMATS_EXACT, EVD_GENAI_ERROR_STATUS, EVD_GENAI_BUILD_COST
   :statement: Agconflo shall reach language models through genai.

   Providers differ in their wire formats, and measured, the difference is real
   rather than cosmetic: OpenAI's requests went to one path with three keys,
   Anthropic's to another with a fourth. ``genai`` put the same prompt into
   both, byte for byte (``EVD_GENAI_TWO_FORMATS_EXACT``), and handed back a
   failure's status as a value (``EVD_GENAI_ERROR_STATUS``).

   Writing an adapter per provider was the first alternative, and it is the
   work ``genai`` exists to do, done again and kept current by hand. A framework
   with an agent loop of its own was the second, and loses on the argument this
   project is built on: the loop - what goes into a call and what comes of it -
   is the layer Agconflo exists to own, and a framework's loop is one to be
   fought rather than used.

   The cost is measured and accepted: about half a minute of clean build and 201
   crates (``EVD_GENAI_BUILD_COST``). Pinned exactly, at the newest release,
   which is a pre-release.

.. dec:: Model calls live in the Lua host's crate
   :id: DEC_MODELS_IN_LUA_CRATE
   :dec_status: accepted
   :decided_on: 2026-09-23
   :statement: Agconflo shall keep model calls in agconflo-lua rather than in a crate of their own.

   A judgement. The script host is the only thing that calls a model, and a
   crate boundary with one consumer on each side decides nothing a module
   boundary does not. The core gains no dependency either way
   (``DEC_BEHAVIOUR_OWN_CRATE``); the day a second kind of caller wants models,
   the module moves.

.. dec:: A script names a role and the caller names the model
   :id: DEC_MODELS_BY_ROLE
   :dec_status: accepted
   :decided_on: 2026-09-23
   :statement: Agconflo shall let a script call a model by a role that the caller maps to a provider's model when starting the run.

   A judgement, argued from the stakeholder requirement it serves.
   ``STKH_PROVIDER_CHOICE`` asks that the same workflow run against different
   providers, and a script is part of the workflow. A script naming
   ``gpt-something`` is a workflow that has chosen its vendor, and changing
   vendor would mean editing every script that calls one.

   So a script says what the call is for - ``drafting``, ``review`` - and the
   caller starting the run says which model plays that part this time. The same
   scripts then run against another provider by changing one mapping, which is
   the stakeholder requirement stated as a mechanism.

   Naming the model in the node type's document was the alternative, and has the
   same defect one level up: the workflow still carries its vendor.

.. dec:: Node behaviour runs asynchronously
   :id: DEC_BEHAVIOUR_ASYNC
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_BLOCK_ON_IN_ASYNC_PANICS, EVD_RUN_IS_SEND
   :statement: Agconflo shall run node behaviour asynchronously so that a model call is awaited rather than blocked on.

   A model call is a network call, and something has to wait for it. A
   synchronous host blocking on a runtime of its own was measured panicking the
   moment its caller was itself asynchronous (``EVD_BLOCK_ON_IN_ASYNC_PANICS``),
   which is the caller this project most expects - an editor, a server, an
   agent's tool.

   A thread per call, blocking there instead, was the other alternative. It
   survives an asynchronous caller, and it blocks one of that caller's threads
   for the length of every call, which for a model is seconds.

   Asynchronous behaviour costs the core nothing: the run was measured ``Send``
   (``EVD_RUN_IS_SEND``), so a caller can hold it across an await, as
   ``DEC_RUN_IS_DRIVEN`` argued it could. The Lua state is not, so a scripted run
   stays on one thread; that is the host's to say, and it says it.

.. dec:: A script's instruction limit is set on its thread
   :id: DEC_HOOK_ON_THE_THREAD
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_LUA_HOOK_PER_THREAD
   :statement: Agconflo shall hold a script to its instruction limit through a hook on the Lua thread that runs it rather than on its state.

   Measured, and the finding is the kind that would have shipped: a hook on the
   state never ran in the coroutine an asynchronous call uses, and an endless
   loop ran until the harness killed it (``EVD_LUA_HOOK_PER_THREAD``). Every
   existing limit test used the synchronous call and would still have passed.

   A hook on the thread the script runs in stopped the same loop in six
   milliseconds, across a yield. The memory limit belongs to the state, and held
   either way.

.. dec:: A script's model calls are limited per activation
   :id: DEC_MODEL_CALLS_COUNTED
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_MODEL_CALLS_UNLIMITED
   :statement: Agconflo shall limit the number of model calls one activation's script may make.

   Awaiting a model costs a script no instructions, so the instruction limit
   does not bound calls: a loop made 2000 of them in 1.6 s under a limit it
   never reached (``EVD_MODEL_CALLS_UNLIMITED``). The run's step budget counted
   that as one activation. ``STKH_STEP_BUDGET`` exists because a runaway spends
   real money, and 2000 calls is the runaway it was written for, inside one step.

   Counted per activation, like the other limits, so that the budget and this
   limit together bound a run's calls: at most one limit's worth per activation
   the budget allows. A call over the limit fails the activation without being
   made, since a call once made is spent.

.. dec:: A prompt is a context and nothing is added to it
   :id: DEC_PROMPT_IS_A_CONTEXT
   :dec_status: superseded
   :decided_on: 2026-09-23
   :supported_by: EVD_GENAI_TWO_FORMATS_EXACT
   :statement: Agconflo shall take a model call's prompt as a context and send that context's rendering as the call's only content.

   What was in a call's window is the question this project exists to answer
   exactly. Taking the prompt as a context makes the answer a context the run
   can name: its rendering is what was sent, and its lineage says where each part
   came from. Measured, nothing else went into the window - ``genai`` added
   parameters and no text (``EVD_GENAI_TWO_FORMATS_EXACT``).

   A prompt passed as a Lua string was the alternative. It would be text the run
   never held, assembled in a state that is discarded when the activation ends,
   and the window would be recorded nowhere.

   The answer comes back as a context of its own for the same reason: a script
   composing it into its output keeps the model's bytes addressable as the
   model's, where a string would splice them in indistinguishably from the
   script's own.

   Superseded by ``DEC_WINDOW_IS_A_CONTEXT``, which keeps the window a
   context and sends its parts as the messages they are, once a call has put
   more than a prompt in it.

.. dec:: A model's answer is the text the provider sent
   :id: DEC_ANSWER_AS_SENT
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_GENAI_OPENAI_TRIMS
   :statement: Agconflo shall read a model's answer from the response the provider sent rather than from genai's reading of it, for the response formats it has measured.

   Taken after the others, while implementing: ``genai``'s OpenAI adapter
   trimmed an answer's surrounding whitespace and its Anthropic adapter did not
   (``EVD_GENAI_OPENAI_TRIMS``). The same workflow would then get different bytes
   from two providers saying the same thing, which is the difference
   ``STKH_PROVIDER_CHOICE`` wants to stop at the engine.

   Accepting the trim was the alternative, and it is the one defect this project
   is least willing to carry: text is held byte for byte everywhere else
   (``CREQ_VALUE_TEXT_EXACT``), and a model's bytes are the ones whose origin
   matters most.

   ``genai`` captures the response body on request, and the answer is read from
   it for the two formats measured. For any other format it falls back to
   ``genai``'s reading, trimmed or not, and that is the stated limit rather than
   a guess about formats nobody has tried.
