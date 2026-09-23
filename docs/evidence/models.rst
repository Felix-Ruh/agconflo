==============================
Evidence about calling a model
==============================

Measurements the way a node reaches a language model rests on. All of them were
taken on the same day in a throwaway crate outside the repository, against the
newest release of everything involved: ``genai`` 0.7.0-beta.24, ``mlua`` 0.12.1
with Lua 5.5 and its ``async`` feature, and ``tokio`` 1.53.1, with
``agconflo-core`` by path at ``15d355a``.

No provider was reached. ``genai`` lets a caller decide where each model's
requests go, and the spike sent them to a stub on the loopback interface that
answered in OpenAI's chat-completions format at ``/v1/chat/completions`` and in
Anthropic's messages format at ``/v1/messages``, recording every request's path
and body. What reached the stub is what would have reached a provider; whether a
provider then answers the same way is not something these measurements can say.

.. evd:: genai spoke two providers' formats to a local stub, sending the prompt exactly
   :id: EVD_GENAI_TWO_FORMATS_EXACT
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: genai 0.7.0-beta.24 sent one user message to a local stub in OpenAI's format and in Anthropic's, each at its own path, and both carried the prompt byte for byte, trailing spaces, an accent, a dash and an emoji included.

   The client was given a resolver sending model names beginning ``openai::``
   to the stub as OpenAI and ``anthropic::`` as Anthropic, with a placeholder
   key. Both answers came back as text.

   What else went on the wire was the request's parameters and not content: the
   OpenAI request's keys were ``model``, ``stream`` and ``messages``, and the
   Anthropic request added ``max_tokens``, which ``genai`` fills in per model
   when the caller does not. So the only text a model was shown was the prompt.

.. evd:: A host blocking on a model call panicked under an async caller
   :id: EVD_BLOCK_ON_IN_ASYNC_PANICS
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: Blocking on a runtime of its own from inside a caller that was itself running on tokio panicked with Cannot start a runtime from within a runtime.

   The shape a synchronous host function would need: it cannot await, so it
   starts a runtime and blocks on the call. That works for a caller with no
   runtime at all, and panics for one that has one - which is every caller that
   is itself doing anything else asynchronous, a server or an editor among them.

.. evd:: The state's instruction hook did not reach an asynchronous call
   :id: EVD_LUA_HOOK_PER_THREAD
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: On Lua 5.5 an endless loop run through call_async was not stopped by an instruction hook set on the state and ran until killed after 10 s, while the same hook set on the Lua thread running it stopped the loop in 6.2 ms.

   Run with ``call``, the state-wide hook stopped the same loop in 6.0 ms, as
   measured before (``EVD_LUA_LIMITS_STOP``). ``call_async`` runs a chunk in a
   coroutine, and Lua's hooks belong to a thread, so the one set on the state
   never ran there: the loop was killed by the harness twice, once with a yield
   before it and once without.

   Setting the hook on the thread the chunk runs in - created by the caller
   rather than by ``call_async`` - stopped it, yield and all. The memory limit
   belongs to the state and held under ``call_async``: a one-gigabyte
   ``string.rep`` failed with a memory error.

.. evd:: A looping script made every model call it asked for
   :id: EVD_MODEL_CALLS_UNLIMITED
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: On Lua 5.5 under a one-million instruction limit a script calling a model 2000 times in a loop made all 2000 calls in 1.6 s, and the instruction limit was never reached.

   Each iteration costs a handful of instructions, and awaiting a model costs
   none, so the instruction limit - which exists for scripts that compute too
   long - does not bound how often a script calls out. Against a provider that
   bills per call, that is 2000 calls from one activation, with the run's step
   budget counting it as one.

.. evd:: A provider's failure kept its status
   :id: EVD_GENAI_ERROR_STATUS
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: A stub answering 401 made genai fail the call with a three-line message beginning Web call failed for model, and the error's status accessor returned 401.

   So which failure occurred is available as a value, where the message alone
   would make a caller parse prose to tell an expired key from an overloaded
   provider.

.. evd:: genai costs about half a minute to build
   :id: EVD_GENAI_BUILD_COST
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: A clean debug build of the dependencies of a crate depending on genai 0.7.0-beta.24 alone took 33.2 s and 33.7 s, over 201 crates including aws-lc-rs.

   Two builds from an empty target directory with the sources fetched.
   ``aws-lc-rs`` is ``rustls``'s default cryptography and builds C and assembly,
   as vendored Lua builds C; it built on this Windows machine with the tools Rust
   already needs. For comparison, the same measurement of ``mlua`` alone was
   under six seconds (``EVD_LUAU_BUILD_COST``).

.. evd:: A run can be held across an await on any runtime
   :id: EVD_RUN_IS_SEND
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: agconflo-core at 15d355a compiled a check that Run and Context are Send, so an asynchronous caller can hold a run across an await on a multi-threaded runtime.

   ``DEC_RUN_IS_DRIVEN`` rested on the argument that its caller is free to be
   asynchronous without the core knowing, and said that claim was the one most
   worth disproving early. This is the first measurement of it, and it holds.

   A Lua state is not ``Send`` with the features in use - the same check written
   for ``mlua::Lua`` failed to compile, on an ``Rc`` inside it - so a caller
   running scripts keeps them on one thread. That is the script host's
   constraint, not the run's.
