==========================================
Decisions about running from the documents
==========================================

How a person runs a workflow from the documents that describe it
(``STKH_RUN_FROM_DOCUMENTS``): where the code that hosts a run lives, how it is
told which documents make up a workflow and which models its roles are played
by, how it keeps a run's record, and what it tells the person when the run
stops.

Five rest on measurements recorded in ``evidence/runner``. The other nine
are judgements between the alternatives each names. One closes a question ``DEC_SCRIPTS_FROM_CALLER`` left
open on purpose: how a caller that reads files links a node type to its
script.

What they do not settle is named here. Nothing below decides a website or
any other interface beside the command line, which a goal of their own will
ask for; a second one is a crate depending on the runner as the command line
does. Nothing decides how a person is shown a step beyond printing it, or how
arguments other than text are given.

.. dec:: A run is hosted by a library of its own, and the command line is a crate beside it
   :id: DEC_RUNNER_CRATES
   :dec_status: accepted
   :decided_on: 2026-09-25
   :statement: Agconflo shall host a run from its documents in a library crate of its own, agconflo-runner, and give the command line a crate of its own, agconflo-cli, depending on it.

   Everything that hosting a run takes but the documents and the scripts is the
   same for every workflow (``EVD_HOST_IS_A_PROGRAM``), and it is the same for
   every way a person might ask for a run. Reading the documents, mapping the
   models, keeping the record and driving the scripted run belong to all of
   them; arguments, printing and exit statuses belong to a terminal.

   One package holding a library and a binary was the first alternative. A
   second interface would then depend on a crate whose other half is the
   command line, and what is shared and what is the terminal's would be kept
   apart by nothing but care. Putting the hosting into ``agconflo-lua`` was
   the second, and loses on that crate's own terms: its callers read files, and
   it reads none (``DEC_SCRIPTS_FROM_CALLER``).

   ``agconflo-runner`` is the first library in the project that opens a file.
   That is what it is for: it is the caller the libraries beneath it leave the
   files to.

.. dec:: A workflow is named to the runner by a manifest
   :id: DEC_RUN_FROM_A_MANIFEST
   :dec_status: accepted
   :decided_on: 2026-09-25
   :statement: Agconflo shall take the documents, scripts and person-performed node types of a run from a manifest that names each of them by a path relative to the manifest's own directory.

   ``DEC_SCRIPTS_FROM_CALLER`` left the link between a node type and its script
   to the caller, "decided when there is one". This is that caller. A manifest
   is a TOML document naming the workflow document, the node type documents,
   a script for each node type that has one, the node types a person performs
   (``DEC_PERSON_NAMED_BY_CALLER``), the run's budget, and optionally the
   script limits.

   A convention was the first alternative - the script for a node type as a
   file named after it, beside the document declaring it. A node type's name
   is any TOML key (``DEC_NAMES_AS_KEYS``), and a name holding ``/`` or ``:``
   names no file, so a convention would need an explicit form beside it
   anyway, and who performs a type cannot be a convention at all. Naming
   everything on the command line was the second: a run that cannot be
   repeated from a file cannot be reviewed or kept beside its workflow.

   Paths are relative to the manifest's directory, never to wherever the
   person happens to be, which answers the "relative to what" that made the
   library refuse to resolve paths at all.

.. dec:: A key the runner does not read is refused
   :id: DEC_UNKNOWN_KEYS_REFUSED
   :dec_status: accepted
   :decided_on: 2026-09-25
   :statement: Agconflo shall refuse a manifest or a model mapping that holds a key it does not read, at that key.

   A workflow document passes over the keys it does not name, because the
   writer keeps them for whoever wrote them (``DEC_TOPOLOGY_IN_TOML``).
   Nothing writes a manifest or a model mapping, so nothing is kept by passing
   over a key, and one passed over is usually a misspelt one. ``persns``
   passed over leaves a person's step with no script, which is refused later
   and elsewhere under a name that is not the misspelling.

.. dec:: A run's models are mapped in a file of their own
   :id: DEC_MODELS_IN_A_FILE_OF_THEIR_OWN
   :dec_status: accepted
   :decided_on: 2026-09-25
   :statement: Agconflo shall read which model plays each role of a run from a model mapping file named when the run is asked for, apart from its manifest.

   Which provider a machine can reach, and with which credentials, is the
   machine's; the workflow is the same workflow on each
   (``STKH_PROVIDER_CHOICE``). A mapping inside the manifest would make moving
   a workflow to another provider an edit to the workflow's own documents.

   Roles on the command line were the alternative. A role needs a model and
   may need an endpoint and a key variable, and three values per role on a
   command line is a file written badly.

.. dec:: A model is named with its provider
   :id: DEC_MODELS_NAMED_WITH_THEIR_PROVIDER
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_GENAI_UNKNOWN_NAME_TO_OLLAMA, EVD_GENAI_NAMESPACE_ROUTES
   :statement: Agconflo shall refuse a model mapping naming a model without the provider namespace genai reads, such as openai:: or anthropic::.

   A name ``genai`` does not recognise goes to Ollama, sent, with nothing in
   the answer to tell a misspelt name from a local model
   (``EVD_GENAI_UNKNOWN_NAME_TO_OLLAMA``); ``cluade-probe`` was sent there.
   With the namespace, the namespace alone picks the adapter and is taken off
   before sending (``EVD_GENAI_NAMESPACE_ROUTES``), so nothing is lost by
   requiring it.

   Passing the name to ``genai`` as written was the alternative, and a
   mapping to a cloud model mistyped once would then send its prompts to
   whatever answers on ``localhost:11434``. A local model is named with its
   provider as plainly as any other: ``ollama::`` for Ollama, and ``openai::``
   with an endpoint of its own for a server speaking OpenAI's format, as LM
   Studio does.

.. dec:: A key comes only from a variable the mapping names
   :id: DEC_KEYS_ONLY_BY_NAMED_VARIABLE
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_GENAI_KEY_CHECKED_AT_CALL, EVD_GENAI_EMPTY_KEY_SENT
   :statement: Agconflo shall send a model a key only from an environment variable its model mapping names, and an empty key for a role whose mapping names none.

   ``genai``'s default auth reads each adapter's own variable -
   ``ANTHROPIC_API_KEY``, ``OPENAI_API_KEY`` - from the environment of the
   process making the call, unasked (``EVD_GENAI_KEY_CHECKED_AT_CALL``). A key
   set for the user is then spent by every run that user starts, named
   nowhere a reader of the workflow or its mapping would look.

   The default auth was the alternative, and costs nothing to write. It loses
   because the mapping would stop being the whole account of where a run's
   calls go and whose account pays for them. Naming the variable costs one
   line per role that needs a key, and a local model needs none.

   A role naming none is sent an empty key rather than none: ``genai`` refuses
   a request with no key for both adapters measured, and sends an empty one
   as it is (``EVD_GENAI_EMPTY_KEY_SENT``).

   A variable named and not set is refused before the run: ``genai`` finds it
   only when the call is made, midway through a run that has already spent
   whatever came before.

.. dec:: A run's arguments are given as text
   :id: DEC_ARGUMENTS_AS_TEXT
   :dec_status: superseded
   :decided_on: 2026-09-25
   :statement: Agconflo shall take each argument of a run a person asks for as text for one entry instance's parameter, and make it a context of the type that parameter declares.

   A person has text to give, and a text context is what they can give without
   writing anything that builds a context (``DEC_PERSON_SUPPLIES_TEXT`` makes
   the same choice for a person's answer). The text is kept exactly as given,
   read from the command line or from a file, a file's line endings included.

   A composition as an argument was the alternative, and it needs a way to
   write down parts and identifiers that nothing yet asks for. It stays open.

.. dec:: A run's arguments are given as text for any instance's parameter
   :id: DEC_ARGUMENTS_AS_TEXT_PER_PARAMETER
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supersedes: DEC_ARGUMENTS_AS_TEXT
   :statement: Agconflo shall take each argument of a run a person asks for as text for one instance's parameter, and make it a context of the type that parameter declares.

   What ``DEC_ARGUMENTS_AS_TEXT`` decided stands, for any instance rather than
   for entry instances, which there are none of any more
   (``STKH_RUN_FROM_ANY_PARAMETER``). A person has text to give, it is kept
   exactly as given, from the command line or from a file, and the context
   made of it is of the type the parameter declares.

   The runner looks only for the declaration, which it needs to make the
   context. Whether a declared parameter is one the run takes - nothing binds
   it, and nothing else is given for it - is the run's question, and the run
   refuses the rest before any node runs
   (``DEC_RUN_REFUSED_UNLESS_EVERY_INPUT_GIVEN``).

.. dec:: A record file is replaced by renaming a new file over it
   :id: DEC_RECORD_REPLACED_BY_RENAME
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_RENAME_REPLACES_RECORD
   :statement: Agconflo shall keep a run's latest record by writing it to a new file beside the record file and renaming that file over it.

   Measured on Windows, a rename either replaces the record or fails and
   leaves the previous one whole (``EVD_RENAME_REPLACES_RECORD``). Writing the
   record file in place was the alternative, and a process ending mid-write
   leaves a file that is neither record, where the record was the only way to
   resume the run.

   A rename can fail while another program holds the file without delete
   sharing. The previous record is then still whole and still describes the
   run up to the step before, which is the failure ``DEC_RECORD_NOT_KEPT_TOLD``
   reports.

.. dec:: A record file is held by one run at a time through a lock beside it
   :id: DEC_RECORD_FILE_LOCKED
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_FILE_LOCK_DIES_WITH_ITS_PROCESS, EVD_CREATE_NEW_ONE_WINNER
   :statement: Agconflo shall hold a record file for a run by taking an exclusive lock on a lock file beside it, which the operating system releases when the process holding it ends.

   ``DEC_ANSWER_ONCE_BY_THE_KEEPER`` leaves answering a record once to
   whoever keeps it, and the runner keeps it. Replacing the record after an
   answer covers one person answering twice in turn; two answering at once
   both read the record before either replaces it, and give the two runs
   sharing identifiers that decision's evidence measured.

   The lock is refused to another process and to another handle in the same
   one, and is free again as soon as the process holding it is killed
   (``EVD_FILE_LOCK_DIES_WITH_ITS_PROCESS``). A run interrupted by its process
   ending - the interruption a record is kept for - therefore leaves nothing
   that stops it being resumed. The lock file stays on disk, held by nobody.

   Creating the lock file exclusively and removing it when the run stops was
   the first form of this decision, and exclusive creation does let exactly
   one of many racers win (``EVD_CREATE_NEW_ONE_WINNER``). It lost before any
   code was written: a process killed mid-run leaves the file behind, and every
   resume after a crash would be refused until a person deleted it by hand.

   A lock on the record file itself was the other alternative. The record is
   replaced by renaming over it (``DEC_RECORD_REPLACED_BY_RENAME``), and a
   rename over a file held open without delete sharing fails
   (``EVD_RENAME_REPLACES_RECORD``), so the lock would stop the very write it
   exists to protect.

   The standard library's file lock is from Rust 1.89, so ``agconflo-runner``
   declares that as its own ``rust-version``, as ``agconflo-lua`` declares
   what ``mlua`` needs.

.. dec:: A record not kept is told once the run has stopped
   :id: DEC_RECORD_NOT_KEPT_TOLD
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_KEEP_CANNOT_STOP_A_RUN
   :statement: Agconflo shall report a record it could not write once the run has stopped, with the last record its file holds, rather than stop the run.

   A scripted run gives its caller no way to stop it from the function that
   receives each record (``EVD_KEEP_CANNOT_STOP_A_RUN``). The runner notes the
   first failure and reports it with the run's ending, which it reports
   whatever that ending is: a run that completed without its record being kept
   still completed.

   Changing ``agconflo-lua`` so that failing to keep a record stops the run was
   the alternative. It would change that crate's interface for a failure that
   loses resumability and nothing else, and it stays open until a run is
   measured spending much after its record stopped being kept.

.. dec:: The command line exits with a status for each way a run stops
   :id: DEC_EXIT_STATUS_PER_ENDING
   :dec_status: accepted
   :decided_on: 2026-09-25
   :statement: Agconflo's command line shall exit with a status of its own for each way a run it was asked for stops.

   0 completed, 2 a command line it cannot read, 3 awaiting a person, 4
   refused before anything ran, 5 a node failed, 6 the budget ran out, 7 stuck
   with nothing more to do, 8 the record not kept. A run that completed but
   whose record was not kept exits 8, since the completion is printed and the
   status is the one thing a script driving the command line checks.

   One status for every failure was the alternative, and
   ``STKH_TYPED_FAILURE`` wants the failure told apart; a script retrying a
   run on a failed node and not on a budget run out needs the two to differ
   without reading prose. 1 is left unused, as the status most tools give a
   failure of any kind, so none of these is mistaken for one.

.. dec:: The command line prints what the run produced, and nothing else, on its standard output
   :id: DEC_RESULT_ON_STANDARD_OUTPUT
   :dec_status: accepted
   :decided_on: 2026-09-25
   :statement: Agconflo's command line shall print a completed run's result, or the step a run awaits, to standard output and everything else it says to standard error.

   The result is what a person running a workflow wants to keep, and what a
   script driving the command line reads; a run's refusals and failures are
   for the person reading the terminal. The step a run awaits is printed where
   the result would be because it is what the person has to act on: which
   instance, what it was given, rendered, and the type of context it produces.

   A machine-readable form was the alternative and stays open. Another
   interface reads the runner's own values, not the terminal's text.

.. dec:: The command line is parsed with clap
   :id: DEC_COMMAND_LINE_THROUGH_CLAP
   :dec_status: accepted
   :decided_on: 2026-09-25
   :statement: Agconflo's command line shall read its arguments with clap.

   Four commands with their own flags, read by a person who types them, is
   what the crate is for: usage, help and the message for a misspelt flag
   come with it. Reading ``std::env::args`` by hand, as ``junit-to-needs``
   does, was the alternative; it suits one flag, and every command added to it
   would be parsing written and tested here.

.. dec:: The runner's entry points are futures run on one thread
   :id: DEC_RUNNER_ON_ONE_THREAD
   :dec_status: accepted
   :decided_on: 2026-09-25
   :statement: Agconflo shall give the runner's entry points as futures that are not Send, to be polled on the thread that runs them.

   A scripted run's future is not ``Send``: a Lua state is not
   (``DEC_BEHAVIOUR_ASYNC``). The runner awaits one, so its own futures are not
   either, and saying so is the decision - hiding it behind a blocking call
   would suit the command line and leave the next interface, which will want
   to await a run beside other work, with a blocking call to move off its
   threads.

   The command line polls them on a current-thread runtime.
