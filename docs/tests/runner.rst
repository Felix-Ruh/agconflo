=================================
Running from documents test cases
=================================

How each requirement in ``components/runner`` is to be checked, with three
feature-level cases where the claim is about separate processes. Results are
never written here: they are imported from the test runner.

A case's id is the path of the Rust test that implements it, uppercased. The
runner's cases live in ``agconflo-runner``'s modules ``project``,
``model_map``, ``keeper`` and ``runner``; the command line's in
``agconflo-cli``'s integration test file ``command``, which runs the real
binary. ``junit-to-needs`` already has an integration file ``cli``, and a
test's id carries no crate, so the command line's is named apart from it.

Every file a case writes is in a directory of its own, and every case reading
a manifest reads it from a directory other than the one the test runs in. A
model is answered by a stub on the loopback interface, as the scripted run's
own cases are, and no case reaches a provider. Every process a case starts
has the providers' key variables removed from its environment, and a case
that needs one set sets a value of its own (AGENTS.md).

Every failure mode listed in ``components/runner`` is named by the case that
catches it.

.. test_case:: A manifest gives what it names
   :id: TEST_PROJECT_READS_WHAT_IT_NAMES
   :verifies: CREQ_PROJECT_READS_THE_MANIFEST
   :test_kind: positive
   :coverage: full

   A manifest naming a workflow, two node type documents, a script for each
   of three node types, one type a person performs, a budget and all three
   limits, each under a name of its own. What is read gives the workflow's
   name and instances, every node type of both documents, each script under
   its own type with the path the manifest wrote as its document, the
   person's type as performed by a person and given no script, the budget,
   and the limits as written.

   Catches: a script given to the wrong node type or a person-performed type
   dropped; the budget or a limit misread.

.. test_case:: A manifest's paths are read from its own directory
   :id: TEST_PROJECT_PATHS_FROM_THE_MANIFEST
   :verifies: CREQ_PROJECT_READS_THE_MANIFEST
   :test_kind: positive
   :coverage: partial

   The same project in a directory two levels below the test's, one of its
   paths going up a level to a shared node type document, read by a path to
   the manifest from the test's own directory. Everything is read; the same
   manifest copied beside a workflow document of the same file name holding
   something else still reads its own.

   Catches: a path resolved against the working directory.

.. test_case:: A script's text is kept exactly
   :id: TEST_PROJECT_SCRIPTS_KEPT_EXACTLY
   :verifies: CREQ_PROJECT_READS_THE_MANIFEST
   :test_kind: property
   :coverage: partial

   For any text of ASCII and wider characters with LF, CRLF and lone CR line
   endings, a leading byte-order mark or none, written as a script's file, the
   script read is that text byte for byte.

   Catches: a script's text changed on the way.

.. test_case:: A manifest without limits gives the script host's defaults
   :id: TEST_PROJECT_LIMITS_DEFAULT_WHEN_ABSENT
   :verifies: CREQ_PROJECT_READS_THE_MANIFEST
   :test_kind: positive
   :coverage: partial

   A manifest with no ``limits`` table gives ``Limits::default()`` field for
   field; one naming a single limit gives that one and the defaults for the
   other two.

   Catches: absent limits read as zero.

.. test_case:: A file that is not there is refused by the path the manifest gives
   :id: TEST_PROJECT_MISSING_FILE_NAMED
   :verifies: CREQ_PROJECT_REFUSES_UNREADABLE
   :test_kind: error_path
   :coverage: partial

   A manifest that is not there, and manifests naming a workflow document, a
   node type document and a script that are not there, each refused as a file
   not found, naming the path as the manifest writes it, the manifest's own as
   given.

   Catches: a missing file reported without its path.

.. test_case:: A fault in a manifest or a file it names carries its place
   :id: TEST_PROJECT_FAULTS_CARRY_THEIR_PLACE
   :verifies: CREQ_PROJECT_REFUSES_UNREADABLE
   :test_kind: error_path
   :coverage: partial

   A manifest that is not TOML at line 3, column 5; a workflow document it
   names that is not TOML; a node type document with a context type the model
   refuses. Each is refused naming its own file and the line and column the
   fault is at, the documents' with the topology reader's own fault handed
   back unchanged.

   Catches: a fault without its place, or placed in the manifest when it is in
   a file the manifest names.

.. test_case:: A file that is not UTF-8 is refused
   :id: TEST_PROJECT_UNDECODABLE_FILE_REFUSED
   :verifies: CREQ_PROJECT_REFUSES_UNREADABLE
   :test_kind: error_path
   :coverage: partial

   A manifest, a workflow document and a script each holding the byte 0xFF,
   each refused as not text, naming the file. Nothing panics.

   Catches: a file that is not UTF-8 read with its bytes replaced, or a panic
   on it.

.. test_case:: A key a manifest may not hold is refused at that key
   :id: TEST_PROJECT_UNKNOWN_KEY_REFUSED
   :verifies: CREQ_PROJECT_REFUSES_UNKNOWN_KEYS
   :test_kind: error_path
   :coverage: full

   ``persns`` at the top of a manifest, and ``instruction`` under ``limits``,
   each refused as a key the manifest may not hold, naming the key and placed
   at it. A ``scripts`` table keyed by a node type name holding ``/``, ``:``
   and a space is read.

   Catches: a misspelt key passed over; only the top level checked.

.. test_case:: A manifest lacking what it must hold is refused at the key
   :id: TEST_PROJECT_MISSING_OR_MISTYPED_KEY_REFUSED
   :verifies: CREQ_PROJECT_REFUSES_MISSING_KEYS
   :test_kind: error_path
   :coverage: full

   A manifest without ``workflow``, one without ``budget``, and ones with
   ``budget = "5"``, ``budget = -1``, ``types = "types.toml"`` and a script
   path given as a number. Each is refused naming the key; a missing one at
   line 1, column 1, a value of the wrong kind at the value.

   Catches: a missing budget given a default; a number written as a string,
   or negative, accepted; a list given as one string read as one path.

.. test_case:: Each role's calls reach the model, endpoint and key its mapping names
   :id: TEST_MODEL_MAP_ROLES_REACH_THEIR_MODELS
   :verifies: CREQ_MODEL_MAP_ROLE_TO_MODEL
   :test_kind: positive
   :coverage: full

   Two stubs on the loopback interface and a mapping of three roles: an
   ``openai::`` model at the first stub's endpoint with a key from a variable
   the test sets, an ``anthropic::`` model at the second's with no variable,
   and an ``openai::`` model at the second's with another variable. A call to
   each role reaches the stub its endpoint names, on the path its adapter
   uses, with the model's name without its namespace, and with a key as long
   as its variable's value, or an empty one.

   Catches: the namespace taken off before the name is handed over; an
   endpoint ignored, or one role's endpoint used for another's calls.

.. test_case:: A provider's default variable is never sent
   :id: TEST_MODEL_MAP_DEFAULT_KEY_NEVER_SENT
   :verifies: CREQ_MODEL_MAP_ROLE_TO_MODEL
   :test_kind: error_path
   :coverage: partial

   The test runs itself again as a child process whose environment has
   ``ANTHROPIC_API_KEY`` and ``OPENAI_API_KEY`` set to values of the test's own,
   of different lengths - a test cannot set its own environment, since
   ``unsafe`` is forbidden here - and the child maps an ``anthropic::`` and an
   ``openai::`` role naming no variable to a stub. Each call reaches the stub
   with an empty key.

   Catches: a default variable's key sent.

.. test_case:: A model named without a provider is refused
   :id: TEST_MODEL_MAP_UNNAMESPACED_REFUSED
   :verifies: CREQ_MODEL_MAP_REFUSES_UNNAMESPACED
   :test_kind: error_path
   :coverage: full

   Mappings naming ``claude-sonnet``, ``gpt-probe``, ``cluade-probe``,
   ``opnai::probe`` and ``::probe``, each refused naming the role and placed
   at the model; the same mapping with ``anthropic::claude-sonnet`` is read.
   A mapping whose first role is sound and whose last is refused is refused
   whole, before any call.

   Catches: a name ``genai`` recognises by its prefix accepted; a misspelt
   namespace accepted; one role refused only when a script first calls it.

.. test_case:: A key variable that is not set is refused when the mapping is read
   :id: TEST_MODEL_MAP_UNSET_VARIABLE_REFUSED
   :verifies: CREQ_MODEL_MAP_REFUSES_UNSET_VARIABLE
   :test_kind: error_path
   :coverage: full

   The environment is the one the model map is given to look variables up
   in. A mapping naming a variable it does not have is refused naming the
   variable and placed at it, and the stub it names receives nothing. A
   variable set to the empty string is read, and its role sent an empty key.

   Catches: the variable left to ``genai``; a variable set to nothing taken as
   unset.

.. test_case:: A model mapping that cannot be read is refused where the fault is
   :id: TEST_MODEL_MAP_UNREADABLE_REFUSED
   :verifies: CREQ_MODEL_MAP_REFUSES_UNREADABLE
   :test_kind: error_path
   :coverage: full

   A mapping that is not TOML, one without ``roles``, one with ``key_evn``, one
   with ``timeout`` at the top, and ones giving a role as a number and as a
   list. Each is refused naming the file, the key where there is one, and its
   line and column.

   Catches: a misspelt ``key_env`` passed over; a role given as a number or a
   list read as a model.

.. test_case:: Each record handed over replaces the one before it whole
   :id: TEST_KEEPER_RECORD_REPLACED_WHOLE
   :verifies: CREQ_KEEPER_REPLACES_WHOLE
   :test_kind: property
   :coverage: full

   For any sequence of texts with LF and CRLF line endings, each handed to a
   keeper in turn, the record file holds exactly the last one handed over
   after each, and the directory holds nothing but the record file and its
   lock file once the keeper is let go.

   Catches: the text changed on the way; the new file written elsewhere; a
   temporary file left behind. Written in place is caught by
   ``TEST_KEEPER_UNKEPT_RECORD_TOLD``, where a failed write leaves the record
   before it whole.

.. test_case:: A new run is refused a record file that already exists
   :id: TEST_KEEPER_EXISTING_FILE_REFUSED
   :verifies: CREQ_KEEPER_REFUSES_EXISTING
   :test_kind: error_path
   :coverage: full

   A record file holding a record, and an empty one, each given to a new run's
   keeper: refused as existing, naming the file, and the file afterwards is
   the same bytes, with the same modification time, as before.

   Catches: checked after the first record is written; the file truncated on
   opening; an empty file taken as no file.

.. test_case:: A record file held for a run is refused to another
   :id: TEST_KEEPER_HELD_FILE_REFUSED
   :verifies: CREQ_KEEPER_ONE_HOLDER
   :test_kind: error_path
   :coverage: partial

   A keeper holds a record file. A second keeper for the same file, for a
   resume, is refused as held, naming the lock file, having read nothing; the
   first keeper then hands over a record, which the file holds. Once the first
   is let go a third takes the file. Sixteen threads released together to take
   one free file succeed once.

   Catches: checked and then taken; taken after the record is read; a refused
   run deleting the lock file the holder has locked.

.. test_case:: A killed holder's lock is released
   :id: TEST_KEEPER_KILLED_HOLDER_RELEASES
   :verifies: CREQ_KEEPER_ONE_HOLDER
   :test_kind: error_path
   :coverage: partial

   A child process - the test binary itself, asked to hold a record file -
   takes it and says so. A keeper in the test is refused it. The child is
   killed, and a keeper then takes the file at once. Run on Linux in
   continuous integration as well, where it measures what
   ``EVD_FILE_LOCK_DIES_WITH_ITS_PROCESS`` measured on Windows only.

   Catches: a killed holder's lock outliving it.

.. test_case:: A record file behind the run is reported with which record it holds
   :id: TEST_KEEPER_UNKEPT_RECORD_TOLD
   :verifies: CREQ_KEEPER_TELLS_WHICH_IS_KEPT
   :test_kind: error_path
   :coverage: full

   Five records handed to a keeper, the third and fourth while the path of its
   temporary file is taken by a directory, so that each write fails on every
   system. Let go after the fourth, it reports that the file holds the second
   record of four, and the file holds the second's text. Handed the fifth after
   the directory is gone, and let go, it reports nothing, and the file holds
   the fifth. No record's handing over panics.

   Catches: every failure forgotten; a failure reported after the file caught
   up; the run stopped at the first failure.

.. test_case:: A run is started from its documents and completes
   :id: TEST_RUNNER_RUNS_TO_COMPLETION
   :verifies: CREQ_RUNNER_STARTS
   :test_kind: positive
   :coverage: full

   A project of three script nodes in a chain, one of them calling a model
   answered by a stub, started with an argument. The run completes with the
   rendering the scripts and the model make of the argument, and the record
   file holds the record handed over last, taken after each record by a
   keeper watching it: one per output and per answer, none held back to the
   end. The same run asked for again with that record file is refused as
   existing, and the stub has been asked nothing more.

   Catches: records collected and written at the end; arguments made from
   another identifier source than the run's; the record file taken after the
   run starts.

.. test_case:: An argument given as text is a context of its parameter's type
   :id: TEST_RUNNER_ARGUMENTS_KEPT_EXACTLY
   :verifies: CREQ_RUNNER_ARGUMENTS_AS_TEXT
   :test_kind: property
   :coverage: full

   For any text with LF and CRLF line endings, and leading and trailing
   spaces, given as an argument for an entry instance's required parameter
   of one type and its optional parameter of another, the entry node's script
   returns what it was given, and the run completes with the rendering of
   exactly that text and with each input's declared type.

   Catches: the text trimmed; every argument given one type.

.. test_case:: An argument for no entry parameter is refused before the run
   :id: TEST_RUNNER_UNKNOWN_ARGUMENT_REFUSED
   :verifies: CREQ_RUNNER_REFUSES_UNKNOWN_ARGUMENT
   :test_kind: error_path
   :coverage: full

   Arguments for an instance the workflow does not have, for a parameter the
   entry instance does not declare, and for a parameter of an instance that
   is not an entry. Each is refused naming the instance and the parameter as
   given, no script runs, and no record file is made.

   Catches: the argument dropped; a made-up type given.

.. test_case:: An interrupted run is resumed from its file
   :id: TEST_RUNNER_RESUMES_AN_INTERRUPTED_RUN
   :verifies: CREQ_RUNNER_RESUMES
   :test_kind: positive
   :coverage: full

   A run whose second step calls a model answered by a stub that never
   answers is dropped once the record file holds the record after the first
   output. Resumed from that file against a stub that answers, it completes,
   and the file holds the last record of the resumed run; a second keeper
   tried while the resume awaits the stub is refused.

   Catches: the record read before the file is held; records after the resume
   kept elsewhere, or not at all.

.. test_case:: A person's text answers the step the file's record awaits
   :id: TEST_RUNNER_ANSWERS_THE_AWAITED_STEP
   :verifies: CREQ_RUNNER_ANSWERS
   :test_kind: positive
   :coverage: full

   A run with a person's step between two scripts stops awaiting it; the file
   holds the record handed over last. Text with a trailing newline and a CRLF
   is given for that instance: the run completes with the text in its result
   exactly, the file holds the record after the answer, and answering the
   same file again is refused as an answer for a step the run does not await.

   Catches: the text changed on the way; the file not replaced after a taken
   answer.

.. test_case:: An answer for a step the record does not await leaves the file
   :id: TEST_RUNNER_ANSWER_ELSEWHERE_LEAVES_THE_FILE
   :verifies: CREQ_RUNNER_ANSWERS
   :test_kind: error_path
   :coverage: partial

   The awaiting record above, answered for another instance. Refused as the
   scripted run refuses it, no script runs, and the file is the same bytes as
   before; a keeper then takes the file at once.

   Catches: the file written after a refused answer.

.. test_case:: A check hands back what would refuse a run and runs nothing
   :id: TEST_RUNNER_CHECK_RUNS_NOTHING
   :verifies: CREQ_RUNNER_CHECKS
   :test_kind: positive
   :coverage: full

   A sound project whose scripts each loop forever, under an instruction limit
   too large to stop them, is checked and nothing is found: a check that ran
   one would not return before the test runner kills the test at 20 s. A
   project with two wiring defects and a script that does not compile hands
   back both defects and the script's fault; a model mapping naming an unset
   variable hands back that. No record file is made.

   Catches: a script run to see whether it compiles; only the first fault
   handed back; a key variable left unchecked.

.. test_case:: A refused start takes nothing
   :id: TEST_RUNNER_REFUSED_START_TAKES_NOTHING
   :verifies: CREQ_RUNNER_REFUSED_START_TAKES_NOTHING
   :test_kind: error_path
   :coverage: full

   A run asked for with an unreadable manifest, and one with an unreadable
   model mapping, each refused as the reader refused it. Neither record file
   nor lock file exists afterwards, and the same run asked for once the
   manifest is mended starts.

   Catches: the record file taken first; a lock left held.

.. test_case:: How a run stopped is handed back unchanged
   :id: TEST_RUNNER_ENDINGS_HANDED_BACK
   :verifies: CREQ_RUNNER_HANDS_BACK_HOW
   :test_kind: positive
   :coverage: full

   Runs that complete, await a person, fail at a script raising an error, run
   out of budget and end quiescent are each handed back as the ending or the
   step the scripted run gave, the failure with its instance and its message.
   A run that completes while its record file cannot be written is handed back
   completed, with which record the file holds.

   Catches: a failure turned into text; the record keeper's report dropped
   when the run completed.

.. test_case:: The commands reach the runner
   :id: TEST_COMMAND_COMMANDS_REACH_THE_RUNNER
   :verifies: CREQ_COMMAND_READS_THE_COMMAND
   :test_kind: positive
   :coverage: full

   The binary run with ``check`` on a project, then ``run`` with an argument
   given with ``--arg`` and another with ``--arg-file``, which stops awaiting a
   person, then ``answer`` with ``--text-file`` holding text with spaces and a
   CRLF, which completes. Each argument and the answer reach the result
   exactly, and ``resume`` on the completed run's file hands back the ending
   again.

   Catches: an argument's instance and parameter swapped, or the text split at
   a space; a text file's contents trimmed.

.. test_case:: A run is resumed after its process is killed
   :id: TEST_COMMAND_RESUMED_AFTER_A_KILL
   :verifies: FEAT_RUNNER_RESUMES_FROM_THE_FILE
   :test_kind: positive
   :coverage: partial

   The binary started with ``run`` on a project whose first step calls a
   model at a stub that counts its requests and whose second calls one at a
   stub that never answers, killed once the record file holds the record after
   the first step. ``resume`` on that file, with the second role mapped to a
   stub that answers, exits 0 with the result, and the counting stub was asked
   once in all.

.. test_case:: A record file held by a running process is refused to another
   :id: TEST_COMMAND_HELD_RECORD_REFUSED
   :verifies: FEAT_RUNNER_ONE_RUN_PER_FILE
   :test_kind: error_path
   :coverage: partial

   While one ``run`` of the binary waits on a stub that never answers, a
   second process's ``resume`` of the same file exits 4 naming the lock file,
   and the record file is unchanged by it.

.. test_case:: A person answers in a process of their own
   :id: TEST_COMMAND_ANSWERED_IN_A_NEW_PROCESS
   :verifies: FEAT_RUNNER_TAKES_THE_ANSWER
   :test_kind: positive
   :coverage: partial

   ``run`` exits 3 with the awaited step printed, and the process has ended.
   ``answer`` in a new process, given only the manifest, the record file and
   the text, exits 0 with a result holding the text.

.. test_case:: Each way a run stops exits with its own status
   :id: TEST_COMMAND_EXIT_STATUS_PER_ENDING
   :verifies: CREQ_COMMAND_EXIT_STATUS
   :test_kind: positive
   :coverage: full

   Projects whose runs complete, await a person, are refused, fail at a
   script, run out of budget, end quiescent, and complete without their record
   kept exit 0, 3, 4, 5, 6, 7 and 8, all different, and a misread command line
   exits 2. No status is 101.

   Catches: two ways sharing a status; a completed run whose record was not
   kept exiting 0; a panic's 101.

.. test_case:: A result or an awaited step is all that goes to standard output
   :id: TEST_COMMAND_RESULT_ON_STANDARD_OUTPUT
   :verifies: CREQ_COMMAND_RESULT_ON_STDOUT
   :test_kind: positive
   :coverage: full

   A completed run whose result has no final newline prints exactly its
   rendering to standard output. An awaiting run prints the instance, the
   type it produces and each input's parameter and rendering, and nothing
   else. A run given a model mapping and a budget prints nothing more on
   standard output than the result.

   Catches: a newline or a label added to the result; progress or warnings on
   standard output.

.. test_case:: A refusal or failure is named on standard error and nothing is printed to standard output
   :id: TEST_COMMAND_FAILURE_ON_STANDARD_ERROR
   :verifies: CREQ_COMMAND_FAILURE_ON_STDERR
   :test_kind: error_path
   :coverage: full

   A manifest with a fault at line 3, column 5, and a run whose second node's
   script raises an error after the first produced its output. Standard
   output is empty for both; standard error names the manifest's file, line
   and column for the first, and the failing instance and its message for the
   second.

   Catches: a refusal printed without where it is; a node's failure printed
   without which node; part of a result printed before the failure.

.. test_case:: A record not kept is named on standard error beside the result
   :id: TEST_COMMAND_UNKEPT_RECORD_TOLD
   :verifies: CREQ_COMMAND_UNKEPT_RECORD_TOLD
   :test_kind: error_path
   :coverage: full

   A run that completes while its temporary record file's path is taken by a
   directory prints its result to standard output, says on standard error
   which record the file holds, and exits 8.

   Catches: said only when the run failed.

.. test_case:: What cannot be read as a command exits 2 and makes nothing
   :id: TEST_COMMAND_MISUSE_EXITS_2
   :verifies: CREQ_COMMAND_MISUSE
   :test_kind: error_path
   :coverage: full

   No command, an unknown command, ``run`` without ``--record``, ``answer``
   without ``--instance``, ``--arg`` with one value, and an unknown flag. Each
   exits 2 with the usage on standard error and nothing on standard output,
   and the directory it ran in is left empty.

   Catches: a missing record file taken as a default path; a file created
   before the command line is read.
