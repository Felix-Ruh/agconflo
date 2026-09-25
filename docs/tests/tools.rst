============================
Performing a tool test cases
============================

How each requirement in ``components/tools`` is to be checked, with cases at
the feature level where the claim is about a whole run or about separate
processes. Results are never written here: they are imported from the test
runner.

A case's id is the path of the Rust test that implements it, uppercased. The
cases live in ``agconflo-runner``'s modules ``grants``, ``project``,
``performer``, ``sandbox`` and ``runner``, and in ``agconflo-cli``'s
integration test file ``command``, which runs the real binary.

A case about what a container does uses Docker and the image the tests pin by
its digest, ``alpine:3`` as measured in ``evidence/tools``, and fails with a
message naming what is missing when either is (``DEC_TESTS_NEED_DOCKER``).
Every container a case makes carries a label naming the case, and a case
removes its own. The runner's and the tool performer's handling of a step is
checked against a stand-in for the sandbox that answers as told and counts
what it was asked. No case reaches the network or a provider, and every
process a case starts has the providers' key variables removed from its
environment (AGENTS.md).

Every failure mode listed in ``components/tools`` is named by the case that
catches it, and where one is caught only in part the case says so.

.. test_case:: A grants file gives what it names
   :id: TEST_GRANTS_READS_WHAT_IT_NAMES
   :verifies: CREQ_GRANTS_READS
   :test_kind: positive
   :coverage: partial

   A grants file naming an image by digest, two folders - one writable, one
   not - the network, all three actions and both limits. What is read gives
   each as written: the folders under their names with their paths joined to
   the file's directory and each one's writability, the network granted, the
   three actions, and the two limits.

   Catches: nothing on its own; it is the control the two cases below are
   read against.

.. test_case:: A grants file saying nothing grants nothing it did not say
   :id: TEST_GRANTS_NARROW_BY_DEFAULT
   :verifies: CREQ_GRANTS_READS
   :test_kind: positive
   :coverage: partial

   A grants file naming its image and one folder, and nothing else. The folder
   is read-only, the network is not granted, no action is allowed, and the
   limits are 60 seconds and 16384 bytes.

   Catches: a folder writable by default, the network granted by a key left
   out, an absent limit read as zero.

.. test_case:: A grants file's paths are read from its own directory
   :id: TEST_GRANTS_PATHS_FROM_THE_FILE
   :verifies: CREQ_GRANTS_READS
   :test_kind: positive
   :coverage: partial

   A grants file two directories below the test's, granting a folder by a
   path going up a level, read by a path from the test's own directory. The
   folder given is the one beside the file's directory, not the one the same
   path names from where the test stands, which exists and differs.

   Catches: a path resolved against the working directory.

.. test_case:: A grants file that cannot be read is refused where the fault is
   :id: TEST_GRANTS_UNREADABLE_REFUSED
   :verifies: CREQ_GRANTS_REFUSES_UNREADABLE
   :test_kind: error_path
   :coverage: full

   Each refused, naming the file: one that does not exist; one that is not
   UTF-8, refused without a panic; one whose TOML is broken on its third line,
   at that line and column; one lacking its image; one with a misspelt key at
   the top, and one with a misspelt key inside a folder's table, each at that
   key; and one allowing an action ``rn``, at that action.

   Catches: a misspelt key passed over, a fault without its place, a panic on
   a file that is not UTF-8, an unknown action name kept.

.. test_case:: An image not pinned by digest is refused
   :id: TEST_GRANTS_TAGGED_IMAGE_REFUSED
   :verifies: CREQ_GRANTS_REFUSES_TAGGED_IMAGE
   :test_kind: error_path
   :coverage: full

   ``alpine``, ``alpine:3``, ``alpine:sha256``, a digest of 63 digits, one of
   65, and one with an uppercase digit are each refused at the image. A name
   with ``@sha256:`` and 64 lowercase digits, and ``sha256:`` with 64, are
   read: the controls, without which a refusal of everything passes.

   Catches: a tag with a digest-like suffix accepted, a digest of the wrong
   length accepted, a bare name accepted.

.. test_case:: A granted folder that is missing, a file, or named as a path is refused
   :id: TEST_GRANTS_BAD_FOLDER_REFUSED
   :verifies: CREQ_GRANTS_REFUSES_BAD_FOLDER
   :test_kind: error_path
   :coverage: full

   A folder whose path does not exist, one whose path is a file, and folders
   named ``a/b``, ``a\b``, ``..``, ``.`` and the empty name are each refused at
   that folder; nothing was created where the missing path points. A folder
   named ``src-2`` at an existing directory is read.

   Catches: a missing folder passed on to be created, a file granted as a
   folder.

.. test_case:: A manifest gives its tools and hands them over as a person's steps
   :id: TEST_PROJECT_READS_TOOLS
   :verifies: CREQ_PROJECT_READS_TOOLS
   :test_kind: positive
   :coverage: full

   A manifest with a script, a person's node type and a ``tools`` table naming
   three node types, one per action. What is read gives each tool with its own
   action, and the behaviours handed to the scripted run name every tool's
   node type and the person's as performed by a person, and give none of them a
   script.

   Catches: a tool left out of what the scripted run is told a person
   performs, the action of one tool given to another.

.. test_case:: A tool the manifest names wrongly is refused at its node type
   :id: TEST_PROJECT_TOOL_FAULTS_REFUSED
   :verifies: CREQ_PROJECT_REFUSES_TOOL_FAULTS
   :test_kind: error_path
   :coverage: full

   Each refused at the node type: a tool with the action ``rum``; a node type
   named under ``scripts`` and ``tools``; one named in ``persons`` and
   ``tools``. The same manifest with each fault removed is read.

   Catches: a misspelt action passed over, a node type performed two ways
   accepted.

.. test_case:: Each action is performed with its own inputs
   :id: TEST_PERFORMER_ACTIONS_PERFORMED
   :verifies: CREQ_PERFORMER_PERFORMS_THE_ACTION
   :test_kind: positive
   :coverage: partial

   In a container with one writable folder: a ``write`` of text to
   ``work/new/dir/a.txt`` makes both folders and the file, whose text on the
   host is the text given, and answers naming the file and its bytes; a
   ``read`` of it answers with the text; a ``run`` of ``ls work/new/dir; exit
   3`` answers with a first line giving status 3, then ``a.txt``. The
   ``write``'s inputs are declared in the order ``text``, ``path``.

   Catches: the status line after the output, an input taken by position.

.. test_case:: A file written and read back is the text given, byte for byte
   :id: TEST_PERFORMER_TEXT_KEPT_EXACTLY
   :verifies: CREQ_PERFORMER_PERFORMS_THE_ACTION
   :test_kind: property
   :coverage: partial

   For text of ASCII and wider characters, quotes, dollar signs and backticks,
   with LF, CRLF and lone CR line endings, ending in a line ending or not: a
   ``write`` and then a ``read`` answers with that text, and the file on the
   host holds exactly its bytes. Few cases, since each is two steps in a
   container.

   Catches: text changed on the way.

.. test_case:: A path absolute or reaching a parent folder is refused without asking the sandbox
   :id: TEST_PERFORMER_PATH_REFUSED
   :verifies: CREQ_PERFORMER_REFUSES_PATH
   :test_kind: error_path
   :coverage: full

   Against the stand-in: ``read`` and ``write`` steps given ``/etc/passwd``,
   ``\etc``, ``C:\x``, ``c:x``, ``work/../../etc``, ``work/..`` and ``..\x``
   are each answered with text saying the path was refused, and the stand-in
   was asked nothing. ``work/a..b`` and ``work/.hidden`` are passed on: the
   controls.

   Catches: a parent part found only at the start, only ``/`` treated as a
   separator, the sandbox asked anyway.

.. test_case:: An action that failed is answered with what went wrong
   :id: TEST_PERFORMER_FAILURE_ANSWERED_AS_TEXT
   :verifies: CREQ_PERFORMER_FAILURE_AS_TEXT
   :test_kind: error_path
   :coverage: full

   In a container: a ``read`` of a file that does not exist answers with text
   naming the read and what the container printed, not with nothing; a
   ``write`` into a folder granted read-only answers the same way. Against the
   stand-in, a ``read`` step whose optional ``path`` was left unbound answers
   with text naming the missing input, and the stand-in was asked nothing.
   None of them fails the node.

   Catches: a failed read answered with nothing, a missing input failing the
   node or panicking.

.. test_case:: The engine's failure is passed back and not answered as text
   :id: TEST_PERFORMER_ENGINE_FAILURE_PASSED
   :verifies: CREQ_PERFORMER_PASSES_ENGINE_FAILURE
   :test_kind: error_path
   :coverage: full

   The stand-in reports an engine failure for a ``run`` step and for a
   ``read`` step. Each is handed back as the engine's failure with its
   message, and no text for the step.

   Catches: the engine's message answered as the step's text.

.. test_case:: A path and text are never read as shell
   :id: TEST_SANDBOX_PATHS_AND_TEXT_NOT_SHELL
   :verifies: CREQ_SANDBOX_PATHS_AS_ARGUMENTS
   :test_kind: error_path
   :coverage: full

   A write to a file named ``a'; touch pwned; '`` makes a file of exactly that
   name, and no ``pwned``. A write of 300000 bytes - past the 131072 one
   argument may hold on Linux - arrives whole.

   Catches: a path quoted into a command, text passed as an argument.

.. test_case:: A step reaches only what its container was granted
   :id: TEST_SANDBOX_CONFINED_TO_GRANTS
   :verifies: CREQ_SANDBOX_LOCKED_DOWN
   :test_kind: error_path
   :coverage: partial

   A container granted one folder read-only and one writable, beside a third
   folder not granted. ``rm -rf`` of the read-only folder fails and it is
   unchanged; ``rm -rf /`` leaves the read-only and the ungranted folders
   unchanged and the writable one emptied, which the grant allows. The step's
   effective capabilities in ``/proc/self/status`` are zero. Without the
   network granted, ``/sys/class/net`` lists only ``lo``; with it, it lists
   another as well - the control.

   Catches: a folder mounted writable that was granted read-only, the
   engine's default network left on.

.. test_case:: A step that kills everything it can leaves the container running
   :id: TEST_SANDBOX_OWN_PROCESS_SURVIVES
   :verifies: CREQ_SANDBOX_LOCKED_DOWN
   :test_kind: error_path
   :coverage: partial

   A step running ``kill -KILL -1; kill -KILL 1``, then a step running
   ``echo alive``, which answers ``alive`` with status 0.

   Catches: the container's own process run as the step's user.

.. test_case:: A step runs as the person's user
   :id: TEST_SANDBOX_STEP_USER
   :verifies: CREQ_SANDBOX_STEP_USER
   :test_kind: positive
   :coverage: partial

   A step running ``id -u; id -g`` answers the user and group of the test's
   own process on Linux and 1000 and 1000 elsewhere, never 0. On Linux a file
   the step writes into a writable folder belongs on the host to the test's
   user.

   Catches: root.

.. test_case:: A process's user and group are read from its status file
   :id: TEST_SANDBOX_USER_FROM_STATUS
   :verifies: CREQ_SANDBOX_STEP_USER
   :test_kind: positive
   :coverage: partial

   Texts shaped as ``/proc/self/status`` is, with ``Uid`` and ``Gid`` lines of
   four tab-separated ids whose real and effective ids differ, among lines
   such as ``Umask`` and ``NSpgid`` that begin alike: what is read is the
   effective user and group. A text lacking either line is read as none.

   Catches: the lines of the status file misread.

.. test_case:: Containers a killed run left are removed and no others
   :id: TEST_SANDBOX_LEFTOVERS_REMOVED
   :verifies: CREQ_SANDBOX_REMOVES_LEFTOVERS
   :test_kind: error_path
   :coverage: full

   A running container and a stopped one labelled with a record file's path,
   and a running one labelled with another record file's. Making a container
   for the first record file removes both of its own and leaves the other's
   running.

   Catches: only stopped containers removed, containers of other record files
   removed.

.. test_case:: A command past its time is ended with what it printed
   :id: TEST_SANDBOX_TIME_LIMIT
   :verifies: CREQ_SANDBOX_TIME_LIMIT
   :test_kind: error_path
   :coverage: full

   Under a limit of two seconds, ``echo started; sleep 30; echo never`` is
   handed back in under ten seconds with ``started`` and not ``never``, and
   the status 137; a step after it lists no ``sleep 30`` among the
   container's processes.

   Catches: the limit kept on the host, output lost when a command is ended.

.. test_case:: Nothing a step started is running after it
   :id: TEST_SANDBOX_NOTHING_LEFT_RUNNING
   :verifies: CREQ_SANDBOX_NOTHING_LEFT_RUNNING
   :test_kind: error_path
   :coverage: full

   A step running ``sleep 40 &``, a double fork of ``sleep 50`` ignoring TERM
   and HUP, and ``echo ok``. The next step lists the container's processes:
   none of the step's user remains but the one listing them, and no zombie.
   A third step runs, so the container's own process lived through it.

   Catches: a background process left, the container's own process killed by
   the cleanup.

.. test_case:: Output past its limit keeps both ends and says what was cut
   :id: TEST_SANDBOX_OUTPUT_KEEPS_BOTH_ENDS
   :verifies: CREQ_SANDBOX_OUTPUT_LIMIT
   :test_kind: error_path
   :coverage: full

   Under a limit of 100 bytes, outputs of 99, 100, 101 and 1000 bytes. The
   first two are handed back whole. The others are handed back as their first
   50 and last 50 bytes around a line naming 1 and 900 bytes cut. A 1000-byte
   output with a two-byte character across each cut is handed back as text,
   the broken characters replaced.

   Catches: the count of bytes cut wrong, output cut through a character and
   failing, output exactly at the limit cut.

.. test_case:: The engine's own failure is told apart from a command's status
   :id: TEST_SANDBOX_ENGINE_FAILURE_APART
   :verifies: CREQ_SANDBOX_ENGINE_FAILURE_APART
   :test_kind: error_path
   :coverage: full

   A sandbox whose ``docker`` is pointed by ``DOCKER_HOST`` at a closed port
   reports the engine's failure with its message on making a container. A
   container removed from outside between two steps reports the engine's
   failure on the second. Commands exiting 1 and 125 are handed back as those
   statuses with their output - the controls.

   Catches: an unreachable engine reported as a command exiting 1, a command's
   own 125 reported as the engine's failure.

.. test_case:: A container is removed however its call ends
   :id: TEST_SANDBOX_REMOVED_AT_THE_END
   :verifies: CREQ_SANDBOX_REMOVED_AT_THE_END
   :test_kind: error_path
   :coverage: partial

   A sandbox let go after one step, one let go by an early return, and one
   whose owner panics after a step, the panic caught: after each, no container
   with its label exists.

   Catches: a panic or an early return skipping the removal. That a run
   awaiting a person removes its container is
   ``TEST_RUNNER_TOOL_STEPS_END_TO_END``'s.

.. test_case:: An image absent or lacking sh or timeout is reported without a pull
   :id: TEST_SANDBOX_IMAGE_NOT_READY
   :verifies: CREQ_SANDBOX_IMAGE_READY
   :test_kind: error_path
   :coverage: partial

   An image digest present nowhere is reported absent, naming it, and is not
   listed by ``docker image ls`` afterwards. An image imported from an empty
   archive is reported lacking ``sh``; one committed from the pinned image
   with ``timeout`` removed is reported lacking ``timeout``; both are made
   without the network and removed after. The pinned image is ready - the
   control.

   Catches: an image lacking timeout accepted. The image pulled when absent is
   caught only in part: a digest a registry holds would need the network, and
   one present nowhere fails a pull too; the case asserts the report is the
   absence, not a registry's error.

.. test_case:: A run's tool steps are performed and answered, wherever the run was started
   :id: TEST_RUNNER_PERFORMS_TOOL_STEPS
   :verifies: CREQ_RUNNER_PERFORMS_TOOLS
   :test_kind: positive
   :coverage: full

   Against the stand-in: a workflow whose entry is followed by a ``read``
   tool, a ``run`` tool, a script and a person's step. Started, the run
   awaits the person with the script's output built on both tools' answers,
   having asked the stand-in twice. A record awaiting the ``run`` step,
   resumed, performs it and goes on to the person.

   Catches: a tool step handed back to the person, only the first awaited
   tool step performed.

.. test_case:: A tool step's answer is its output exactly
   :id: TEST_RUNNER_TOOL_TEXT_IS_THE_OUTPUT
   :verifies: FEAT_PERSON_TEXT_IS_THE_OUTPUT
   :test_kind: property
   :coverage: partial

   For any text the stand-in answers a tool step with - ASCII and wider,
   empty, with each kind of line ending - the step's output renders as that
   text and has the type its node type declares.

.. test_case:: A tool step recorded is not performed again on resume
   :id: TEST_RUNNER_RECORDED_TOOL_STEP_NOT_REPEATED
   :verifies: FEAT_RESUME_REPEATS_NO_OUTPUT
   :test_kind: error_path
   :coverage: partial

   A run through two tool steps to a person's step, then resumed from its
   file against a fresh stand-in: the stand-in is asked nothing, and the run
   awaits the same person's step.

.. test_case:: Tool steps count against the run's budget
   :id: TEST_RUNNER_TOOL_STEPS_COUNT_AGAINST_BUDGET
   :verifies: FEAT_RUN_BUDGET_STOPS
   :test_kind: error_path
   :coverage: partial

   A model stub on the loopback interface that calls a tool in every answer,
   under a budget of five: the run ends as having exceeded its budget of
   five, the stand-in having been asked fewer than five times.

.. test_case:: A model's call to a tool is performed and the model goes on with it
   :id: TEST_RUNNER_MODEL_CALLS_A_TOOL
   :verifies: FEAT_YIELD_CALL_PERFORMED
   :test_kind: positive
   :coverage: partial

   A model stub that calls a ``read`` tool twice, then answers with a sentence:
   the stand-in is asked for both reads with the paths the stub gave, the
   stub's third request holds both answers, and the run completes with its
   sentence.

.. test_case:: A run with tools and no readable grants is refused and takes nothing
   :id: TEST_RUNNER_REFUSES_WITHOUT_GRANTS
   :verifies: CREQ_RUNNER_REFUSES_WITHOUT_GRANTS
   :test_kind: error_path
   :coverage: full

   A start, a resume and an answer of a manifest naming a tool, each given no
   grants, and a start given a grants file that cannot be read: each is
   refused, no record file and no lock file exist afterwards, and the
   stand-in was asked nothing. The same manifest without its tool starts with
   no grants - the control.

   Catches: missing grants found at the first tool step, the record file
   taken before the grants are read.

.. test_case:: Every tool the grants do not provide for is named before the run starts
   :id: TEST_RUNNER_REFUSES_UNGRANTED
   :verifies: CREQ_RUNNER_REFUSES_UNGRANTED
   :test_kind: error_path
   :coverage: full

   Against the stand-in: a manifest with two tools whose actions the grants do
   not allow is refused naming both; a ``read`` tool whose node type declares
   no ``path`` is refused naming it; the stand-in reporting the image not
   ready refuses the run with its report. None takes the record file. A
   ``read`` tool declaring ``path`` as optional starts - the control.

   Catches: only the first ungranted tool named, an optional parameter taken
   as missing.

.. test_case:: An engine's failure stops the run awaiting the step, resumable either way
   :id: TEST_RUNNER_ENGINE_FAILURE_LEAVES_THE_STEP
   :verifies: CREQ_RUNNER_ENGINE_FAILURE_STOPS
   :test_kind: error_path
   :coverage: full

   The stand-in reports an engine failure at a ``run`` step. The run stops
   awaiting that step with the failure beside it, and its file holds the
   record awaiting it. Resumed against a stand-in that answers, the step is
   performed and the run goes on; answered by text instead, from a copy of the
   same record, the text is the step's output.

   Catches: the run ended as a failed node, the engine's failure dropped.

.. test_case:: A check reports what would refuse a run's tools and makes no container
   :id: TEST_RUNNER_CHECK_REPORTS_TOOLS
   :verifies: CREQ_RUNNER_CHECKS_TOOLS
   :test_kind: error_path
   :coverage: full

   Against the stand-in: a check of a manifest naming a tool, without grants,
   reports that grants are needed; with grants not allowing its action,
   reports the tool; with the stand-in reporting the image not ready, reports
   that. The stand-in was asked whether the image is ready and never for a
   container.

   Catches: a check that reports nothing about tools, a check that makes the
   run's container.

.. test_case:: A run's tools are performed in a container and it is gone when the run awaits
   :id: TEST_RUNNER_TOOL_STEPS_END_TO_END
   :verifies: FEAT_TOOL_PERFORMED
   :test_kind: positive
   :coverage: partial

   With Docker: a workflow writing a file in a granted folder, running
   ``cat`` on it, and stopping at a person's step. The file is on the host,
   the person's step was given the ``run`` step's output with status 0, and
   no container with the record file's label exists once the run awaits.

.. test_case:: The grants file typed is used by every command
   :id: TEST_COMMAND_GRANTS_HANDED_ON
   :verifies: CREQ_COMMAND_READS_GRANTS
   :test_kind: positive
   :coverage: full

   With Docker: ``check``, ``run``, ``answer`` and ``resume`` of a workflow
   with a ``read`` tool, each with ``--grants``, exit as a check with no
   finding, a run awaiting a person having read the file, an answer
   completing, and a resume of the completed record; ``run`` without
   ``--grants`` exits 4 naming the grants.

   Catches: the grants dropped for one command.

.. test_case:: A step left by the engine's failure is printed with the failure beside it
   :id: TEST_COMMAND_ENGINE_FAILURE_TOLD
   :verifies: CREQ_COMMAND_TELLS_ENGINE_FAILURE
   :test_kind: error_path
   :coverage: full

   With Docker: a run whose ``run`` step sleeps for five seconds has its
   container removed from outside while it sleeps. The command exits 3,
   prints the awaited step to standard output and nothing else there, and
   names the engine's failure and ``resume`` on standard error.

   Catches: a status of its own, the failure on standard output.

.. test_case:: A tool step interrupted by a killed run is performed once more, alone
   :id: TEST_COMMAND_KILLED_TOOL_STEP_PERFORMED_ALONE
   :verifies: FEAT_TOOL_INTERRUPTED_PERFORMED_AGAIN
   :test_kind: error_path
   :coverage: full

   With Docker: a ``run`` step appends ``start`` to a file in a writable
   folder, sleeps three seconds, and appends ``end``. The command is killed
   once ``start`` is written, then the run resumed. The file ends as
   ``start``, ``start``, ``end``: the interrupted performance never wrote its
   ``end``, since its container was removed before the step was performed
   again.

.. test_case:: A run's tool deleting everything changes nothing outside its grant
   :id: TEST_COMMAND_TOOL_KEPT_TO_ITS_GRANT
   :verifies: FEAT_TOOL_KEPT_TO_ITS_GRANT
   :test_kind: error_path
   :coverage: partial

   With Docker: a run whose ``run`` step is given ``rm -rf /work/* /`` as its
   argument, with one folder granted read-only, one writable, and a third
   beside them not granted. The read-only and the ungranted folders are
   unchanged, the writable one is empty, and the run completes with the step's
   status in its result.
