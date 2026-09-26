====================================
Components of running from documents
====================================

The components ``ARCH_RUNNER`` divides running a workflow from its documents
between, four in ``agconflo-runner`` and one in ``agconflo-cli``, and the
requirements allocated to them. Each title is the grammatical subject of the
requirements allocated to it, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

.. comp:: Project reader
   :id: COMP_PROJECT_READER
   :crate: agconflo-runner

   Reads a manifest and every file it names - the workflow document, the node
   type documents and the scripts - into what a scripted run is started with:
   the definition, the behaviours, the budget and the limits. Or refuses them,
   saying which file and where.

   Everything it does is a question about files and their text, with no run
   in it, which is what separates it from the runner that uses what it read.

.. comp:: Model map
   :id: COMP_MODEL_MAP
   :crate: agconflo-runner

   Reads a model mapping into the roster a scripted run calls models through:
   for each role, the model, the endpoint when one is given, and the key from
   the variable named. Or refuses it, at the model or the variable at fault.

   It answers where calls would go and with which key, and sends nothing.

.. comp:: Record keeper
   :id: COMP_RECORD_KEEPER
   :crate: agconflo-runner

   Holds a record file for one run: takes it, refusing one another run holds;
   replaces its contents with each record it is handed, whole; remembers a
   record it could not write; and lets it go when the run stops.

   It knows nothing of what a record says. That is the run record's, in the
   core.

.. comp:: Runner
   :id: COMP_RUNNER
   :crate: agconflo-runner

   Starts, resumes, answers or checks a run from what the project reader and
   the model map read, keeping its records through the record keeper, and
   hands back how the run stopped - its ending, the step it awaits, or why it
   was refused - with whether its record was kept.

   Its entry points are futures polled on one thread
   (``DEC_RUNNER_ON_ONE_THREAD``), and it prints nothing.

.. comp:: Command line
   :id: COMP_COMMAND_LINE
   :crate: agconflo-cli

   The ``agconflo`` binary: reads the command a person typed, asks the runner
   for it, prints what the runner handed back and exits with the status for
   it.

.. comp_req:: A manifest gives what it names, each file read beside it
   :id: CREQ_PROJECT_READS_THE_MANIFEST
   :derived_from: FEAT_RUNNER_STARTS_FROM_DOCUMENTS
   :allocated_to: COMP_PROJECT_READER
   :ears_pattern: event
   :statement: When a manifest is read, Project reader shall give the workflow, node types, scripts, person-performed node types, budget and limits it names, each file read from its path relative to the manifest's directory.

   What a scripted run is started with, from the files a person keeps
   (``DEC_RUN_FROM_A_MANIFEST``). A manifest without ``limits`` gives the
   limits a script host has by default, and each file is named in a fault as
   the manifest writes its path.

   Failure modes:

   - **A path resolved against the working directory.** The run reads another
     workflow's files, or none, depending on where the person stood.
   - **A script given to the wrong node type**, or a person-performed type
     dropped: the run performs a step with another step's behaviour, or
     refuses a workflow that was sound.
   - **A script's text changed on the way** - line endings translated, a
     byte-order mark kept or dropped: a fault is reported at a line the file
     does not have.
   - **The budget or a limit misread**, or absent limits read as zero: every
     script is stopped at its first instruction.

.. comp_req:: A manifest or file that cannot be read is refused where the fault is
   :id: CREQ_PROJECT_REFUSES_UNREADABLE
   :derived_from: FEAT_RUNNER_REFUSES_BEFORE_RUNNING
   :allocated_to: COMP_PROJECT_READER
   :ears_pattern: unwanted
   :statement: If a manifest or a file it names cannot be read, then Project reader shall refuse it naming the file as the manifest gives it and the line and column of a fault in its text.

   A file that is not there, is not text, or is not the TOML its kind of
   document is written in. A workflow or node type document's own faults are
   the topology reader's, with their places
   (``FEAT_TOPOLOGY_UNREADABLE_LOCATED``), and are handed back as they are; a
   manifest's are placed the same way, counted from one.

   Failure modes:

   - **A missing file reported without its path**, as the operating system's
     message alone: the person is told a file is missing and not which.
   - **A fault without its place**, or placed in the manifest when it is in a
     file the manifest names.
   - **A file that is not UTF-8 read with its bytes replaced**, or a panic on
     it: the run proceeds on text nobody wrote, or ends with no typed failure.

.. comp_req:: A key a manifest may not hold is refused at that key
   :id: CREQ_PROJECT_REFUSES_UNKNOWN_KEYS
   :derived_from: FEAT_RUNNER_REFUSES_BEFORE_RUNNING
   :allocated_to: COMP_PROJECT_READER
   :ears_pattern: unwanted
   :statement: If a manifest holds a key the project reader does not read, then Project reader shall refuse the manifest at that key.

   ``DEC_UNKNOWN_KEYS_REFUSED``: a key passed over is usually a misspelt one,
   and the manifest cannot be read as its author meant it. The keys of
   ``scripts`` are node type names, any of which may be written; a key under
   ``limits`` is one of that table's own three or refused.

   Failure modes:

   - **A misspelt key passed over.** ``persns`` leaves a person's step with no
     performer, refused later as a node type with no script.
   - **Only the top level checked.** A misspelt limit falls back to the
     default without a word.

.. comp_req:: A manifest lacking what it must hold is refused
   :id: CREQ_PROJECT_REFUSES_MISSING_KEYS
   :derived_from: FEAT_RUNNER_REFUSES_BEFORE_RUNNING
   :allocated_to: COMP_PROJECT_READER
   :ears_pattern: unwanted
   :statement: If a manifest lacks its workflow or its budget or gives a value of the wrong kind, then Project reader shall refuse it at the key at fault.

   The workflow and the budget have no default: a run of nothing, or a run
   under a budget nobody chose, is not what the manifest's author asked for
   (``STKH_STEP_BUDGET`` stops a run that exceeds its configured budget). A
   missing key is refused at the manifest's first line, a value of the wrong
   kind at the value.

   Failure modes:

   - **A missing budget given a default**, and a runaway loop spends it.
   - **A number written as a string, or negative, accepted** as whatever it
     parses to.
   - **A list of node type documents given as one string** read as one path.

.. comp_req:: Each role's calls go to the model, endpoint and key its mapping names
   :id: CREQ_MODEL_MAP_ROLE_TO_MODEL
   :derived_from: FEAT_RUNNER_MODELS_APART
   :allocated_to: COMP_MODEL_MAP
   :ears_pattern: event
   :statement: When a model mapping is read, Model map shall send each role's calls to the model the mapping names for it, at the endpoint it names when it names one, with the key from the variable it names or an empty one.

   The mapping is the whole account of where a role's calls go and with
   which key (``DEC_KEYS_ONLY_BY_NAMED_VARIABLE``): a role naming no endpoint
   goes to its provider's own, and one naming no variable is sent an empty key
   (``EVD_GENAI_EMPTY_KEY_SENT``), never one of ``genai``'s default variables.

   Failure modes:

   - **The namespace taken off before the name is handed over.** ``genai``
     takes it off itself (``EVD_GENAI_NAMESPACE_ROUTES``), and a bare name it
     does not recognise goes to Ollama.
   - **An endpoint ignored**, or one role's endpoint used for another's calls.
   - **A default variable's key sent** for a role naming none: a key set for
     the user is spent by a workflow whose mapping never mentions it.

.. comp_req:: A model named without a provider is refused
   :id: CREQ_MODEL_MAP_REFUSES_UNNAMESPACED
   :derived_from: FEAT_RUNNER_REFUSES_BEFORE_RUNNING
   :allocated_to: COMP_MODEL_MAP
   :ears_pattern: unwanted
   :statement: If a model mapping names a model without the name of one of genai's adapters before its double colon, then Model map shall refuse the mapping at that model.

   ``DEC_MODELS_NAMED_WITH_THEIR_PROVIDER``: a name ``genai`` does not
   recognise goes to Ollama (``EVD_GENAI_UNKNOWN_NAME_TO_OLLAMA``), so a
   mapping that does not say which provider it means cannot be read as
   meaning any. The namespace is checked against the names ``genai`` gives its
   own adapters; its few namespaces beyond those are refused as well, since
   none is needed to reach any adapter.

   Failure modes:

   - **A name ``genai`` recognises by its prefix accepted.** ``claude-x`` goes
     to Anthropic today and to whatever the prefix list says next release.
   - **A misspelt namespace accepted.** ``opnai::x`` has a double colon and no
     adapter, and falls back to the prefixes and then to Ollama.
   - **One role refused only when a script first calls it**, after the steps
     before it have run.

.. comp_req:: A key variable that is not set is refused before any call
   :id: CREQ_MODEL_MAP_REFUSES_UNSET_VARIABLE
   :derived_from: FEAT_RUNNER_REFUSES_BEFORE_RUNNING
   :allocated_to: COMP_MODEL_MAP
   :ears_pattern: unwanted
   :statement: If a model mapping names a key variable that is not set, then Model map shall refuse the mapping at that variable.

   ``genai`` finds an unset variable only when the call is made
   (``EVD_GENAI_KEY_CHECKED_AT_CALL``), after every step before it has run
   and been paid for. The map asks the environment when it reads the mapping.

   Failure modes:

   - **The variable left to ``genai``**, and the run fails at its first call
     to that role - after a person's step, perhaps, answered for nothing.
   - **A variable set to nothing taken as unset**, and a local server started
     with an empty token refused.

.. comp_req:: A model mapping that cannot be read is refused where the fault is
   :id: CREQ_MODEL_MAP_REFUSES_UNREADABLE
   :derived_from: FEAT_RUNNER_REFUSES_BEFORE_RUNNING
   :allocated_to: COMP_MODEL_MAP
   :ears_pattern: unwanted
   :statement: If a model mapping cannot be read or holds a key the model map does not read, then Model map shall refuse it naming the file and the line and column of the fault.

   A mapping file is TOML holding a ``roles`` table; each role is a model's
   name, or a table of ``model``, ``endpoint`` and ``key_env``
   (``DEC_MODELS_IN_A_FILE_OF_THEIR_OWN``, ``DEC_UNKNOWN_KEYS_REFUSED``).

   Failure modes:

   - **A misspelt ``key_env`` passed over**, and the role sent an empty key
     that fails at the provider with a message about authentication.
   - **A role given as a number or a list** read as a model named by its
     rendering.

.. comp_req:: A record handed over replaces the one before it whole
   :id: CREQ_KEEPER_REPLACES_WHOLE
   :derived_from: FEAT_RUNNER_KEEPS_THE_RECORD
   :allocated_to: COMP_RECORD_KEEPER
   :ears_pattern: event
   :statement: When a record is handed to it, Record keeper shall make the record file hold exactly that record, the one before it replaced whole.

   Written to a file beside the record and renamed over it
   (``DEC_RECORD_REPLACED_BY_RENAME``), so the file holds one whole record at
   every moment.

   Failure modes:

   - **Written in place.** A process ending mid-write leaves text that is
     neither record, and the run cannot be resumed at all.
   - **The text changed on the way**, line endings translated: a record a
     resume then refuses, or reads as another run.
   - **The new file written elsewhere** - the system's temporary directory -
     and the rename crossing file systems, which fails.
   - **A temporary file left behind** after each record.

.. comp_req:: A new run is refused a record file that already exists
   :id: CREQ_KEEPER_REFUSES_EXISTING
   :derived_from: FEAT_RUNNER_RECORD_NOT_OVERWRITTEN
   :allocated_to: COMP_RECORD_KEEPER
   :ears_pattern: unwanted
   :statement: If a new run is given a record file that already exists, then Record keeper shall refuse it and leave that file as it was.

   Failure modes:

   - **Checked after the first record is written**, by which time the other
     run's record is gone.
   - **The file truncated on opening** and then refused.
   - **An empty file taken as no file.** It may be a record another program
     is writing.

.. comp_req:: A record file held for a run in progress is refused to any other
   :id: CREQ_KEEPER_ONE_HOLDER
   :derived_from: FEAT_RUNNER_ONE_RUN_PER_FILE
   :allocated_to: COMP_RECORD_KEEPER
   :ears_pattern: unwanted
   :statement: If a record file is held for a run in progress, then Record keeper shall refuse it to any other run and name the lock file.

   Held through an exclusive lock on a file beside it, which the operating
   system releases when the holding process ends (``DEC_RECORD_FILE_LOCKED``,
   ``EVD_FILE_LOCK_DIES_WITH_ITS_PROCESS``). The lock is taken before the
   record is read, for a resume and an answer alike.

   Failure modes:

   - **Checked and then taken**, two steps: two runs both check, both find it
     free, and both continue from the record.
   - **Taken after the record is read.** A second process reads the record,
     waits, and continues from a record the first has already replaced.
   - **A killed holder's lock outliving it**, and every resume after a crash
     refused.
   - **A refused run deleting the lock file** the holder has locked, so a
     third run takes a new one.

.. comp_req:: A record file behind the run is reported with which record it holds
   :id: CREQ_KEEPER_TELLS_WHICH_IS_KEPT
   :derived_from: FEAT_RUNNER_RECORD_NOT_KEPT_TOLD
   :allocated_to: COMP_RECORD_KEEPER
   :ears_pattern: unwanted
   :statement: If the record file does not hold the latest record handed to it when the run stops, then Record keeper shall report which of those records the file holds.

   Which, counted in the order the records were handed over, so the person
   knows how far back a resume from the file would start
   (``DEC_RECORD_NOT_KEPT_TOLD``). A write that failed and a later one that
   succeeded leave the file current, and nothing is reported.

   Failure modes:

   - **Every failure forgotten**, and a run whose record stopped being kept
     reports nothing.
   - **A failure reported after the file caught up**, sending a person to a
     resume that is already where it should be.
   - **The run stopped at the first failure**, which the scripted run gives
     no way to do (``EVD_KEEP_CANNOT_STOP_A_RUN``) and a panic would.

.. comp_req:: A run is started from what was read for it
   :id: CREQ_RUNNER_STARTS
   :derived_from: FEAT_RUNNER_STARTS_FROM_DOCUMENTS
   :allocated_to: COMP_RUNNER
   :ears_pattern: event
   :statement: When a run is asked for, Runner shall start the scripted run with what the project reader and the model map read for it and the arguments given, keep its records through the record keeper, and hand back where it stopped.

   Where it stopped is an ending of the run, or the step it awaits
   (``DEC_SCRIPTED_RUN_RETURNS_TO_AWAIT``).

   Failure modes:

   - **The record file taken after the run starts**, so a refusal to take it
     comes once the first record is due.
   - **Records collected and written at the end**, and an interrupted run
     leaves none.
   - **Arguments made from another identifier source** than the run's, and
     every run given arguments refused.

.. comp_req:: An argument given as text becomes a context of its parameter's type
   :id: CREQ_RUNNER_ARGUMENTS_AS_TEXT
   :derived_from: FEAT_RUNNER_STARTS_FROM_DOCUMENTS
   :allocated_to: COMP_RUNNER
   :ears_pattern: event
   :statement: When text is given for an instance's parameter, Runner shall supply the run a context of the type that parameter declares holding that text exactly.

   ``DEC_ARGUMENTS_AS_TEXT_PER_PARAMETER``, changed by
   ``DEC_CHANGE_RUNNER_ARGUMENTS``. The type is the one the instance's node
   type declares for the parameter, whichever instance it is: a run's inputs
   are the parameters nothing binds, on any instance
   (``STKH_RUN_FROM_ANY_PARAMETER``). Whether the parameter is one the run
   takes is the run's to say (``CREQ_RUN_REFUSES_UNFILLED_SIGNATURE``).

   Failure modes:

   - **The text trimmed**, or a file's line endings translated.
   - **Every argument given one type**, and the run refused for a parameter
     that declares another.

.. comp_req:: An argument for no declared parameter is refused before the run
   :id: CREQ_RUNNER_REFUSES_UNKNOWN_ARGUMENT
   :derived_from: FEAT_RUNNER_STARTS_FROM_DOCUMENTS
   :allocated_to: COMP_RUNNER
   :ears_pattern: unwanted
   :statement: If text is given for a parameter no instance of the workflow declares, then Runner shall refuse the run before any node runs and name that instance and parameter.

   Changed by ``DEC_CHANGE_RUNNER_ARGUMENTS``. The runner makes the context,
   and a parameter nobody declares has no type to make it of, so it refuses
   before the run would.

   Failure modes:

   - **The argument dropped**, and a misspelt parameter leaves the real one
     unsupplied, refused under the other name.
   - **A made-up type given** so the run can refuse it, which it does, naming
     a type the person never wrote.

.. comp_req:: A run is resumed from the record its file holds
   :id: CREQ_RUNNER_RESUMES
   :derived_from: FEAT_RUNNER_RESUMES_FROM_THE_FILE
   :allocated_to: COMP_RUNNER
   :ears_pattern: event
   :statement: When a run is asked to resume from a record file, Runner shall resume the scripted run from the record that file holds and keep each record after it in that file.

   Failure modes:

   - **The record read before the file is held**, the failure
     ``CREQ_KEEPER_ONE_HOLDER`` names.
   - **Records after the resume kept elsewhere**, or not at all, and a second
     interruption goes back to the first.

.. comp_req:: A person's text answers the record its file holds
   :id: CREQ_RUNNER_ANSWERS
   :derived_from: FEAT_RUNNER_TAKES_THE_ANSWER
   :allocated_to: COMP_RUNNER
   :ears_pattern: event
   :statement: When text is given for the step of an instance, Runner shall answer the record its file holds with that text for that instance and keep each record after it in that file.

   The answer is refused, with nothing run and nothing written, unless the
   record awaits that instance (``FEAT_PERSON_ANSWER_ELSEWHERE_REFUSED``).

   Failure modes:

   - **The text changed on the way**, trimmed or its line endings translated.
   - **The file written after a refused answer**, and the record the person
     was answering lost.
   - **The file not replaced after a taken answer**, and the same step
     answerable again from it (``DEC_ANSWER_ONCE_BY_THE_KEEPER``).

.. comp_req:: A check hands back what would refuse a run and runs nothing
   :id: CREQ_RUNNER_CHECKS
   :derived_from: FEAT_RUNNER_CHECKS_WITHOUT_RUNNING
   :allocated_to: COMP_RUNNER
   :ears_pattern: event
   :statement: When a check is asked for, Runner shall hand back the faults of the project and model map read for it, the workflow's wiring defects and the scripts' faults, and run no node.

   What a start would refuse that does not depend on the arguments or on a
   record. No record file is named for a check, and none is taken.

   Failure modes:

   - **A script run to see whether it compiles.** Compiling is not running,
     and a script that loops or fails would do so in a check.
   - **Only the first fault handed back**, where the wiring check gives every
     defect together.
   - **A key variable left unchecked** because nothing is called, and the run
     the check passed refused at once.

.. comp_req:: A refused start takes nothing
   :id: CREQ_RUNNER_REFUSED_START_TAKES_NOTHING
   :derived_from: FEAT_RUNNER_REFUSES_BEFORE_RUNNING
   :allocated_to: COMP_RUNNER
   :ears_pattern: unwanted
   :statement: If the project or the model map read for a run is refused, then Runner shall hand back that refusal having started no run and taken no record file.

   Failure modes:

   - **The record file taken first**, and the corrected run then refused as
     one given a file that already exists.
   - **A lock left held** by a refusal in the same process.

.. comp_req:: How a run stopped is handed back as the scripted run gave it
   :id: CREQ_RUNNER_HANDS_BACK_HOW
   :derived_from: FEAT_RUNNER_TELLS_HOW_IT_STOPPED, FEAT_RUNNER_RECORD_NOT_KEPT_TOLD
   :allocated_to: COMP_RUNNER
   :ears_pattern: event
   :statement: When a run stops, Runner shall hand back its ending, the step it awaits or its refusal as the scripted run gave it, with which record its file holds when that is not the latest.

   Failure modes:

   - **A failure turned into text** before it leaves the runner, so an
     interface other than the command line cannot tell which it was.
   - **The record keeper's report dropped** when the run completed.

.. comp_req:: A command typed is asked of the runner
   :id: CREQ_COMMAND_READS_THE_COMMAND
   :derived_from: FEAT_RUNNER_STARTS_FROM_DOCUMENTS, FEAT_RUNNER_RESUMES_FROM_THE_FILE, FEAT_RUNNER_TAKES_THE_ANSWER, FEAT_RUNNER_CHECKS_WITHOUT_RUNNING
   :allocated_to: COMP_COMMAND_LINE
   :ears_pattern: event
   :statement: When a person types a command, Command line shall ask the runner for that command with the manifest, model mapping, record file, arguments and text given.

   The commands are ``run``, ``resume``, ``answer`` and ``check``
   (``DEC_COMMAND_LINE_THROUGH_CLAP``). Paths are taken as typed, relative to
   where the command is run; the manifest's own paths are the project
   reader's, relative to the manifest.

   Failure modes:

   - **An argument's instance and parameter swapped**, or the text split at a
     space.
   - **A text file's contents trimmed** on the way to an answer or an
     argument.

.. comp_req:: The command line exits with a status for each way a run stops
   :id: CREQ_COMMAND_EXIT_STATUS
   :derived_from: FEAT_RUNNER_TELLS_HOW_IT_STOPPED
   :allocated_to: COMP_COMMAND_LINE
   :ears_pattern: event
   :statement: When the runner hands back how a run stopped, Command line shall exit with the status of its own for that way of stopping.

   The statuses are ``DEC_EXIT_STATUS_PER_ENDING``'s.

   Failure modes:

   - **Two ways sharing a status**, and a script retrying on one retries on
     the other.
   - **A completed run whose record was not kept exiting 0.**
   - **A panic's 101**, for a refusal nobody turned into a status.

.. comp_req:: A completed run's result or an awaited step is printed to standard output
   :id: CREQ_COMMAND_RESULT_ON_STDOUT
   :derived_from: FEAT_RUNNER_TELLS_HOW_IT_STOPPED
   :allocated_to: COMP_COMMAND_LINE
   :ears_pattern: event
   :statement: When a run completes or awaits a person, Command line shall print the result's rendering or the awaited step to standard output and nothing else there.

   ``DEC_RESULT_ON_STANDARD_OUTPUT``. The result is its rendering, byte for
   byte, with nothing added; the awaited step is the instance, the type it
   produces, and each input by parameter with its rendering.

   Failure modes:

   - **A newline or a label added to the result**, and a script reading it
     reads something the workflow did not produce.
   - **Progress or warnings on standard output**, mixed into the result.

.. comp_req:: A refusal or a failure is named on standard error
   :id: CREQ_COMMAND_FAILURE_ON_STDERR
   :derived_from: FEAT_RUNNER_TELLS_HOW_IT_STOPPED
   :allocated_to: COMP_COMMAND_LINE
   :ears_pattern: unwanted
   :statement: If a run is refused or fails, then Command line shall say which refusal or failure it was to standard error and print nothing to standard output.

   Failure modes:

   - **A refusal printed without where it is**, the file, line and column the
     project reader gave.
   - **A node's failure printed without which node**, or without its message.
   - **Part of a result printed** before the failure.

.. comp_req:: A record not kept is named on standard error
   :id: CREQ_COMMAND_UNKEPT_RECORD_TOLD
   :derived_from: FEAT_RUNNER_RECORD_NOT_KEPT_TOLD
   :allocated_to: COMP_COMMAND_LINE
   :ears_pattern: unwanted
   :statement: If the runner hands back that a run's record was not kept, then Command line shall say to standard error which record its file holds.

   Beside whatever else the run's stopping prints: a completed run's result
   still goes to standard output.

   Failure modes:

   - **Said only when the run failed**, and a completed run's lost record
     noticed at the next interruption.

.. comp_req:: What cannot be read as a command is refused with its usage
   :id: CREQ_COMMAND_MISUSE
   :derived_from: FEAT_RUNNER_TELLS_HOW_IT_STOPPED
   :allocated_to: COMP_COMMAND_LINE
   :ears_pattern: unwanted
   :statement: If what a person typed cannot be read as a command, then Command line shall print its usage to standard error and exit with status 2 having asked the runner for nothing.

   Failure modes:

   - **A missing record file taken as a default path**, and a run started into
     a file the person never named.
   - **A file created before the command line is read**, a record file or a
     lock, left behind by a typo.
