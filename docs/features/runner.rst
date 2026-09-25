=====================================
Running a workflow from its documents
=====================================

A person running a workflow from the documents that describe it: starting the
run with its arguments, keeping its record in a file, resuming it from that
file, answering the step it awaits, being told how it stopped, and having the
documents checked without running anything. Every requirement here derives
from a goal in ``stakeholder/execution``, ``stakeholder/authoring`` or
``stakeholder/context``, and each is written against the decisions in
``decisions/runner``.

What this does not do is what keeps it a slice. The core and the scripted run
are not changed: every behaviour a run has is theirs, and what is added is the
host that asks for it from documents and a file. There is one way to ask - the
command line - and nothing else a person could use (``DEC_RUNNER_CRATES``). A
run's arguments are text (``DEC_ARGUMENTS_AS_TEXT``). Nothing here decides how
a website or a model would ask for a run.

Several of these meet requirements already written, because a run hosted from
documents has the behaviours a run hosted by any caller has: resuming one asks
no model again for an answer its record holds
(``FEAT_RESUME_REPEATS_NO_OUTPUT``), and a person's text becomes the step's
output (``FEAT_PERSON_TEXT_IS_THE_OUTPUT``). What these add is the person
reaching them without writing the caller, and the file the record is kept in
between.

Each requirement was checked by hand against the question no rule can ask:
could this be false while its parents are true? The body of each says how. Six
statements are ``event``, four are ``unwanted`` and one is ``state``: a run's
record being kept holds for as long as the run is in progress.

The feature's architecture closes the file. It realises all eleven
requirements and names the components they are divided between, which are
defined in ``components/runner`` with the requirements allocated to them.

.. feat_req:: A run is started from a workflow's documents
   :id: FEAT_RUNNER_STARTS_FROM_DOCUMENTS
   :derived_from: STKH_RUN_FROM_DOCUMENTS
   :ears_pattern: event
   :verification_method: test
   :statement: When a person asks for a run of a workflow from the documents that describe it, Agconflo shall perform that run with the arguments that person gave until it ends or awaits a person.

   The parent's run, and the first thing it asks: that the documents and the
   arguments are all a person gives, and the run is what comes of them.

   It can be false while the parent holds. A host that starts the run and
   stops at the first activation it cannot perform in-process - a model's
   call, say - lets a person run a workflow from its documents in the letter
   of the parent, and runs every workflow that does anything only as far as
   its first step.

.. feat_req:: A run's record is kept in the file a person named
   :id: FEAT_RUNNER_KEEPS_THE_RECORD
   :derived_from: STKH_RESUMABLE_RUN, STKH_RUN_FROM_DOCUMENTS
   :ears_pattern: state
   :verification_method: test
   :statement: While a run a person asked for is in progress, Agconflo shall keep its latest record in the file that person named.

   ``STKH_RESUMABLE_RUN`` resumes a run that was interrupted, and a record that
   outlives the process is what it is resumed from. ``STKH_RUN_FROM_DOCUMENTS``
   counts resuming as part of running a workflow, so the record has to be kept
   by the host the person did not write, somewhere the person can name again.

   It can be false while both hold. A host keeping every record in memory runs
   a workflow from its documents, and a run is still resumable by a caller
   that kept its own record - just not by the person, whose run is gone with
   the process.

.. feat_req:: A new run does not overwrite a kept record
   :id: FEAT_RUNNER_RECORD_NOT_OVERWRITTEN
   :derived_from: STKH_RESUMABLE_RUN
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a person asks for a new run with a record file that already exists, then Agconflo shall refuse to start it and leave that file as it was.

   A record file already holding a record is the record of a run, perhaps one
   awaiting a person for days (the parent's own reason). Starting another run
   into it replaces the only thing that run can be resumed from.

   It can be false while the parent holds. Every run stays resumable from its
   latest record, and a person who names the wrong file loses a run that was
   resumable a moment before - the interruption the parent exists for, caused
   by the host.

.. feat_req:: A run is resumed from the file its record is kept in
   :id: FEAT_RUNNER_RESUMES_FROM_THE_FILE
   :derived_from: STKH_RESUMABLE_RUN, STKH_RUN_FROM_DOCUMENTS
   :ears_pattern: event
   :verification_method: test
   :statement: When a person asks to resume a run from the file its record is kept in, Agconflo shall continue that run from the record the file holds.

   ``STKH_RESUMABLE_RUN``'s resumption, asked for by the person rather than by
   a program of theirs, which ``STKH_RUN_FROM_DOCUMENTS`` says running a
   workflow includes.

   It can be false while both hold. A host that keeps the record and resumes
   nothing leaves resuming to a caller the person writes, which meets
   ``STKH_RESUMABLE_RUN`` and is exactly the program the other parent rules
   out.

.. feat_req:: A person answers the step a kept record awaits
   :id: FEAT_RUNNER_TAKES_THE_ANSWER
   :derived_from: STKH_HUMAN_IN_RUN, STKH_RUN_FROM_DOCUMENTS
   :ears_pattern: event
   :verification_method: test
   :statement: When a person gives text for the step the record in a file awaits, Agconflo shall continue that run with the text as the step's output.

   ``STKH_HUMAN_IN_RUN`` has a person supply a context while a run is in
   progress. Hosted from documents, the run the person answers is the one a
   file holds, and the answer is given to the host rather than to a caller.

   It can be false while both hold. A run started from documents that hands a
   person's step to a caller the person must write meets ``STKH_HUMAN_IN_RUN``
   through that caller, and stops being runnable from its documents at the
   first step a person performs.

.. feat_req:: A record file is continued by one run at a time
   :id: FEAT_RUNNER_ONE_RUN_PER_FILE
   :derived_from: STKH_PROVENANCE
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a run is asked for with a record file that a run in progress holds, then Agconflo shall refuse it without reading the record.

   The parent records which context each byte of a node's input came from, by
   identifier. Two runs continued from one record issue the same identifiers
   for different contexts (``EVD_ONE_RECORD_ANSWERED_TWICE``), so an identifier
   stops saying where a byte came from.

   It can be false while the parent holds. Each of the two runs records its
   own provenance exactly, and the two records then name different contexts
   by the same identifiers - which is provenance that holds for each run and
   says nothing once both exist, the case two people answering one step at
   once produces.

.. feat_req:: A run whose documents cannot be read is refused before it starts
   :id: FEAT_RUNNER_REFUSES_BEFORE_RUNNING
   :derived_from: STKH_WIRING_CHECKED, STKH_RUN_FROM_DOCUMENTS
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a document or model mapping a run is asked for with cannot be read or names a file or variable that does not exist, then Agconflo shall refuse the run before any node in it runs and say where the fault is.

   ``STKH_WIRING_CHECKED`` rejects an invalid workflow before any node runs,
   and a workflow whose documents cannot be read, or whose model mapping names
   a key nobody set, is one no run of can succeed. ``STKH_RUN_FROM_DOCUMENTS``
   has the person find the fault without a program, which is what saying where
   it is means.

   It can be false while both hold. A host that reads each script or key only
   when a node first needs it refuses nothing the wiring check would, and
   finds a missing key halfway through a run that has already paid for every
   step before it.

.. feat_req:: A workflow's documents are checked without running anything
   :id: FEAT_RUNNER_CHECKS_WITHOUT_RUNNING
   :derived_from: STKH_TOPOLOGY_AS_DATA
   :ears_pattern: event
   :verification_method: test
   :statement: When a person asks for a workflow's documents to be checked, Agconflo shall report whether a run of them would be refused and why, without running any node.

   The parent stores topology as data that is validated without executing it,
   and its body names the first thing that follows: a graph can be checked
   before anything runs. A person with documents and no program can check them
   only if the host offers it.

   It can be false while the parent holds. A host whose only way to check the
   documents is to run them validates them before running, as every run does,
   and a person learns that a workflow is sound by spending a run on it.

.. feat_req:: A person is told how a run stopped
   :id: FEAT_RUNNER_TELLS_HOW_IT_STOPPED
   :derived_from: STKH_TYPED_FAILURE, STKH_RUN_FROM_DOCUMENTS
   :ears_pattern: event
   :verification_method: test
   :statement: When a run a person asked for stops, Agconflo shall tell that person whether it completed, awaits a person, was refused or failed, and which failure it was.

   ``STKH_TYPED_FAILURE`` reports which failure occurred when a node fails, and
   the run's ending carries it. Hosted from documents, the one it has to
   reach is the person, with the other ways a run stops beside it: a run
   awaiting a person has not failed, and one refused never started.

   It can be false while both hold. A host that reports every failure with the
   same words and the same status carries the typed failure in the value it
   was handed and loses it at the last step, where the person is.

.. feat_req:: A record not kept is told to the person
   :id: FEAT_RUNNER_RECORD_NOT_KEPT_TOLD
   :derived_from: STKH_RESUMABLE_RUN
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a record of a run a person asked for cannot be kept in its file, then Agconflo shall tell that person once the run stops which record the file still holds.

   The parent resumes an interrupted run, and a run whose record stopped being
   kept can be resumed only from the last one that was. The person has to know
   that before an interruption, not after one.

   It can be false while the parent holds. The run is resumable from whatever
   the file holds, and a person told nothing resumes a run from a record some
   steps older than they think, and pays for those steps again.

.. feat_req:: A run's models are taken from a mapping kept apart from its documents
   :id: FEAT_RUNNER_MODELS_APART
   :derived_from: STKH_PROVIDER_CHOICE
   :ears_pattern: event
   :verification_method: test
   :statement: When a person asks for a run of a workflow from its documents, Agconflo shall take the model each role is played by from a mapping kept apart from those documents.

   The parent runs the same workflow against different providers. Hosted from
   documents, the workflow is its documents, and the same documents run
   against another provider only if which model plays each role is said
   somewhere else.

   It can be false while the parent holds. A host reading each role's model
   from the workflow's own documents runs any workflow against any provider,
   one edit to the documents at a time, and the workflow is then a different
   workflow on every machine it runs on.

.. feat_arch:: Running from documents splits into four parts of a runner and a command line
   :id: ARCH_RUNNER
   :realises: FEAT_RUNNER_STARTS_FROM_DOCUMENTS, FEAT_RUNNER_KEEPS_THE_RECORD, FEAT_RUNNER_RECORD_NOT_OVERWRITTEN, FEAT_RUNNER_RESUMES_FROM_THE_FILE, FEAT_RUNNER_TAKES_THE_ANSWER, FEAT_RUNNER_ONE_RUN_PER_FILE, FEAT_RUNNER_REFUSES_BEFORE_RUNNING, FEAT_RUNNER_CHECKS_WITHOUT_RUNNING, FEAT_RUNNER_TELLS_HOW_IT_STOPPED, FEAT_RUNNER_RECORD_NOT_KEPT_TOLD, FEAT_RUNNER_MODELS_APART
   :uses: COMP_PROJECT_READER, COMP_MODEL_MAP, COMP_RECORD_KEEPER, COMP_RUNNER, COMP_COMMAND_LINE
   :statement: Agconflo shall allocate running a workflow from its documents to the project reader, the model map, the record keeper, the runner and the command line.

   Four components in ``agconflo-runner`` and one in ``agconflo-cli``
   (``DEC_RUNNER_CRATES``), each answerable for one question:

   - The project reader answers what the documents say: the manifest and
     every file it names, read into what a scripted run is started with, or
     refused where the fault is.
   - The model map answers where each role's calls go: the model mapping read
     into a roster, or refused.
   - The record keeper answers what the record file holds and who holds it:
     each record replacing the last whole, one run at a time, and a record it
     could not write remembered.
   - The runner answers what is done with them: starting, resuming, answering
     or checking a run, and handing back how it stopped.
   - The command line answers what the person typed and what they are told:
     the command read, the result or the step printed, the status exited
     with.

   The first four know nothing of a terminal, which is what lets another
   interface sit beside the command line. The core and ``agconflo-lua`` are
   not among them: every behaviour of the run itself is already theirs, and
   nothing here adds one.

   The decisions this is built against are named here rather than linked:

   - ``DEC_RUN_FROM_A_MANIFEST``, ``DEC_UNKNOWN_KEYS_REFUSED``: what the project
     reader reads, and what it refuses.
   - ``DEC_MODELS_IN_A_FILE_OF_THEIR_OWN``,
     ``DEC_MODELS_NAMED_WITH_THEIR_PROVIDER``,
     ``DEC_KEYS_ONLY_BY_NAMED_VARIABLE``: the model map.
   - ``DEC_RECORD_REPLACED_BY_RENAME``, ``DEC_RECORD_FILE_LOCKED``,
     ``DEC_RECORD_NOT_KEPT_TOLD``: the record keeper.
   - ``DEC_ARGUMENTS_AS_TEXT``, ``DEC_RUNNER_ON_ONE_THREAD``: the runner.
   - ``DEC_EXIT_STATUS_PER_ENDING``, ``DEC_RESULT_ON_STANDARD_OUTPUT``,
     ``DEC_COMMAND_LINE_THROUGH_CLAP``: the command line.
