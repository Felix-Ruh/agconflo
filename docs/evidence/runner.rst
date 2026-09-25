===========================
Evidence about a run's host
===========================

Measurements taken for ``STKH_RUN_FROM_DOCUMENTS`` before anything was
specified for it: what hosting a run takes today, how ``genai`` routes a model
named in a file and where its key comes from, how a record file can be
replaced and locked on Windows, and what a scripted run does when its caller
cannot keep a record.

Each was taken in a scratch crate against ``agconflo-core`` and
``agconflo-lua`` at ``62bf607``, on Windows 10 (19045) with Rust 1.96.0 and
the dependency versions this workspace pins - ``genai`` 0.7.0-beta.24 - and
each body says how to take it again. The stub is a loopback HTTP server
answering every request in OpenAI's chat-completions format, recording the
request line, the body's ``model`` field, and whether an ``authorization`` or
``x-api-key`` header came and how long its value was, never the value.

.. evd:: A run's host is a program written against the libraries
   :id: EVD_HOST_IS_A_PROGRAM
   :evd_kind: measurement
   :observed_on: 2026-09-25
   :observation: The workspace's only binary target was junit-to-needs, and running one yielding workflow against LM Studio and resuming it took a program of 107 lines, 86 of them code.

   ``cargo metadata --no-deps``: ``agconflo-core`` and ``agconflo-lua`` build
   a library each and nothing else; ``junit-to-needs`` builds a binary, the
   importer of test results.

   The program is the scratch crate the yield feature was run live with
   (``EVD_LOCAL_MODEL_YIELDS``). Its 86 lines of code hold the node type and
   workflow documents and the two scripts as string constants, read them,
   build a ``genai`` client whose resolver sends every model to LM Studio with
   a key from the environment, map one role, start the run keeping every
   record in memory, and resume from one of them. Every part of it but the
   documents and the scripts would be the same for another workflow.

.. evd:: A namespaced model name picks the adapter and is sent without it
   :id: EVD_GENAI_NAMESPACE_ROUTES
   :evd_kind: measurement
   :observed_on: 2026-09-25
   :observation: Through genai against the stub, openai:: and anthropic:: names chose their adapters, reached the resolver's endpoint, were sent without the namespace, and carried the key from a named variable in each adapter's own header.

   Each client was built with a ``ServiceTargetResolver`` replacing the
   endpoint with the stub's and, where a variable was named, the auth with
   ``AuthData::from_env(name)``, the variable holding 19 characters.

   ``openai::probe-model`` resolved to adapter ``OpenAI`` with default endpoint
   ``https://api.openai.com/v1/``, and the stub received
   ``POST /v1/chat/completions`` with ``"model": "probe-model"`` and an
   ``authorization`` header of 19 characters after ``Bearer``.
   ``anthropic::claude-probe`` resolved to ``Anthropic``, default endpoint
   ``https://api.anthropic.com/v1/``, and the stub received ``POST /v1/messages``
   with ``"model": "claude-probe"``, no ``authorization`` and an ``x-api-key``
   of 19 characters. The resolver saw the name with its namespace and the
   request carried it without.

.. evd:: A model name genai does not recognise goes to Ollama
   :id: EVD_GENAI_UNKNOWN_NAME_TO_OLLAMA
   :evd_kind: measurement
   :observed_on: 2026-09-25
   :observation: Through genai against the stub, probe-model and the misspelt cluade-probe, neither namespaced, each resolved to the Ollama adapter at http://localhost:11434/ and was sent, with no key.

   The stub received ``POST /v1/api/chat`` for each, Ollama's path under the
   endpoint the resolver set, with the name as given and neither an
   ``authorization`` nor an ``x-api-key`` header. Without the resolver the
   request would have gone to ``localhost:11434``.

   ``AdapterKind::from_model`` says why: a namespace decides the adapter when
   there is one, then a list of prefixes (``gpt``, ``claude``, ``gemini`` and
   others), and anything else falls back to Ollama, which its own comment
   says "will never fail". Nothing in the answer tells a misspelt name from a
   local model.

.. evd:: A missing key is found when the call is made
   :id: EVD_GENAI_KEY_CHECKED_AT_CALL
   :evd_kind: measurement
   :observed_on: 2026-09-25
   :observation: Through genai against the stub, a key named in an unset variable, and Anthropic's default key unset, each built a client without complaint and was refused only at the call, with a resolver error and nothing sent.

   ``openai::probe-model`` with the auth read from ``PROBE_UNSET_KEY`` failed
   with ``Resolver error for model 'openai::probe-model (adapter: OpenAI)'``,
   caused by ``ApiKeyEnvNotFound``; the stub received nothing.
   ``anthropic::claude-probe`` with the default auth failed the same way, run
   in a process whose environment had ``ANTHROPIC_API_KEY`` removed.

   The default auth reads the adapter's own variable - ``ANTHROPIC_API_KEY``
   and ``OPENAI_API_KEY`` among them - from the environment of whatever
   process makes the call, so a key set for the user is used by every process
   that user starts without being named anywhere. A first run of this probe,
   with ``ANTHROPIC_API_KEY`` present, sent it to the stub in the default-auth
   case where this run refused.

.. evd:: A record file is replaced by renaming over it, unless something holds it
   :id: EVD_RENAME_REPLACES_RECORD
   :evd_kind: measurement
   :observed_on: 2026-09-25
   :observation: On Windows, renaming over a closed file or one open through std::fs::File replaced it, and over one open without delete sharing or read-only failed with Access is denied, the old file left whole.

   Each case wrote a temporary file beside the destination in the same
   directory and renamed it over the destination, then read the destination
   back.

   Closed: ``Ok``, the new text. Held open by ``std::fs::File::open``, which
   shares reading, writing and deletion: ``Ok``, the new text. Held open with
   ``share_mode`` sharing reading alone, as a program that does not allow
   deletion opens a file: ``PermissionDenied`` (OS error 5), the destination
   still the previous text once closed, the temporary file still there.
   Destination marked read-only: the same error and the same result.

   In none of the cases did the destination hold anything but one whole
   version.

.. evd:: Exclusive creation lets exactly one of many racers win
   :id: EVD_CREATE_NEW_ONE_WINNER
   :evd_kind: measurement
   :observed_on: 2026-09-25
   :observation: On Windows, 16 threads released together to create the same file with OpenOptions::create_new succeeded exactly once in each of 50 rounds, and creating a file already there failed with AlreadyExists.

   Each round used a new path in the temporary directory and a barrier to
   release all 16 threads at once; the count of ``Ok`` results was 1 in every
   round, the lowest and the highest alike.

   Threads in one process ask the operating system for the file as separate
   processes would, but separate processes were not raced.

.. evd:: A caller that cannot keep a record cannot stop the run
   :id: EVD_KEEP_CANNOT_STOP_A_RUN
   :evd_kind: measurement
   :observed_on: 2026-09-25
   :observation: A scripted run of three script nodes, whose keep failed to write each record it was handed, completed with the designated output after handing over four records, all four writes failing.

   The workflow is a chain - an entry node writing ``a``, then two nodes each
   appending ``+`` to its input - completing with ``a++``. ``keep`` wrote each
   record to a path in a directory that does not exist, and counted the
   failures.

   ``keep`` is ``impl FnMut(String)``: it returns nothing, so a caller has no
   way to tell the run a record was not kept. It can note the failure and
   report it once the run has returned, by which time the run has done
   everything it was going to do.

.. evd:: A request with no key is refused, and an empty key is sent
   :id: EVD_GENAI_EMPTY_KEY_SENT
   :evd_kind: measurement
   :observed_on: 2026-09-25
   :observation: Through genai against the stub, AuthData::None was refused at the call for both openai:: and anthropic:: with nothing sent, and an empty key was sent as an empty Bearer value and an empty x-api-key.

   Each client's resolver replaced the auth with ``AuthData::None`` or with
   ``AuthData::from_single("")``, in a process with neither
   ``ANTHROPIC_API_KEY`` nor ``OPENAI_API_KEY`` set. ``None`` failed with a
   resolver error for each adapter and the stub received nothing. The empty
   key reached the stub as ``authorization: Bearer`` with nothing after it
   for OpenAI and as an ``x-api-key`` of no characters for Anthropic.

   Read in its source (``adapter_shared.rs``), ``genai``'s OpenAI adapter
   goes without the header only for the OpenAI-compatible adapters that
   declare they allow it, and the plain
   ``openai::`` adapter is not one of them: a local server reached as
   ``openai::`` with an endpoint of its own is sent some key, and an empty one
   is the one that names nothing.

.. evd:: A file lock is exclusive across processes and dies with its holder
   :id: EVD_FILE_LOCK_DIES_WITH_ITS_PROCESS
   :evd_kind: measurement
   :observed_on: 2026-09-25
   :observation: On Windows, a file locked with std::fs::File::lock refused try_lock from another process and from another handle in the same process, and was locked by another process 3 ms after the holder was killed, in each of 3 runs.

   One process opened a lock file, took ``lock`` on it, printed that it held
   it and slept; a second process and a second handle in the driving process
   each took ``try_lock``, and both got ``WouldBlock``. The holder was then
   killed, and a new process's ``try_lock`` succeeded at once. The lock file
   itself stayed on disk: what a killed process leaves is a file nobody
   holds.

   ``File::lock`` and ``File::try_lock`` are in the standard library from Rust
   1.89. Only Windows was measured; the tests that rest on this run on Linux
   in continuous integration as well.
