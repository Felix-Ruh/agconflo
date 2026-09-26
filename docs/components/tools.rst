===============================
Components of performing a tool
===============================

The components ``ARCH_TOOLS`` divides performing a tool within its grant
between, three new ones in ``agconflo-runner`` defined here and three of
running from documents defined in ``components/runner``, and the requirements
allocated to all six. Each title is the grammatical subject of the
requirements allocated to it, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

.. comp:: Grants reader
   :id: COMP_GRANTS_READER
   :crate: agconflo-runner

   Reads a grants file into what a run's tools may do: the image, each folder
   by name with whether it may be written, whether the network is granted,
   the actions allowed, and the limits on a command's time and output. Or
   refuses it, at the key or the folder at fault.

   Like the project reader, it answers a question about a file and its text,
   and nothing it reads is acted on until the runner asks.

.. comp:: Sandbox
   :id: COMP_SANDBOX
   :crate: agconflo-runner

   The container a run's tool steps are performed in, reached through the
   ``docker`` command: finds the image present, removes what a killed run
   left, makes the container locked down with the granted folders mounted,
   runs one step's command in it within its limits, hands back its output and
   exit status or the engine's own failure, and removes the container.

   It knows what a container is and nothing of what a step means.

.. comp:: Tool performer
   :id: COMP_TOOL_PERFORMER
   :crate: agconflo-runner

   Performs one tool step: reads the step's inputs by the parameters its
   action takes, refuses a path the action may not be given, asks the sandbox
   for what the action does, and makes the text the step is answered with -
   the file read, the write done, the command's status and output, or what
   went wrong.

   It knows the three actions and asks the sandbox for everything else, so
   the runner's handling of tools is tested against a stand-in for the
   sandbox.

.. comp_req:: A grants file gives the image, folders, network, actions and limits it names
   :id: CREQ_GRANTS_READS
   :derived_from: FEAT_TOOL_GRANTED_BY_THE_PERSON
   :allocated_to: COMP_GRANTS_READER
   :ears_pattern: event
   :statement: When a grants file is read, Grants reader shall give its image, each folder by name with its path relative to the file's directory and whether it may be written, whether the network is granted, the actions allowed and the limits.

   What a run's tools may do, as the person wrote it
   (``DEC_GRANTS_IN_A_FILE_OF_THEIR_OWN``). A folder is read-only and the
   network off unless the file says otherwise (``DEC_GRANTS_NARROW_BY_DEFAULT``),
   and the limits are 60 seconds and 16384 bytes unless it gives them.

   Failure modes:

   - **A folder given as writable by default**, or the network granted by a
     key left out: a grant wider than the person wrote, which is the one
     mistake this whole feature exists to prevent.
   - **A path resolved against the working directory** rather than the file's:
     a different folder mounted depending on where the person stood.
   - **An absent limit read as zero**: every command ended before it starts.

.. comp_req:: A grants file that cannot be read is refused where the fault is
   :id: CREQ_GRANTS_REFUSES_UNREADABLE
   :derived_from: FEAT_TOOL_GRANTS_UNREADABLE_REFUSED
   :allocated_to: COMP_GRANTS_READER
   :ears_pattern: unwanted
   :statement: If a grants file cannot be read or holds a key the grants reader does not read or lacks its image, then Grants reader shall refuse it naming the file and the line and column of the fault.

   A file not there, not text, not TOML, lacking the image, or holding a key
   nothing reads. ``DEC_UNKNOWN_KEYS_REFUSED`` reasons about a manifest and a
   model mapping, and the same holds here: a key passed over is usually a
   misspelt one. A misspelt ``writable`` or ``network`` fails safe; a
   misspelt action grants nothing the person meant to.

   Failure modes:

   - **A misspelt key passed over**, so the grant differs from what the person
     wrote without a word.
   - **A fault without its place**, or a panic on a file that is not UTF-8.
   - **An unknown action name kept** rather than refused: a tool refused later
     as ungranted, under a name that is not the misspelling.

.. comp_req:: An image named by a tag is refused
   :id: CREQ_GRANTS_REFUSES_TAGGED_IMAGE
   :derived_from: FEAT_TOOL_GRANTS_UNREADABLE_REFUSED
   :allocated_to: COMP_GRANTS_READER
   :ears_pattern: unwanted
   :statement: If a grants file names its image by anything but a digest or an image id, then Grants reader shall refuse it at the image.

   ``DEC_IMAGE_BY_DIGEST_NEVER_PULLED``: a tag can be moved to another image
   between two runs, and what a tool can run would change without the grant
   changing. A digest is a name, ``@sha256:`` and 64 lowercase hexadecimal
   digits; an image id is ``sha256:`` and the same 64 digits.

   Failure modes:

   - **A tag with a digest-like suffix accepted** - ``alpine:sha256`` - or a
     digest of the wrong length: the check passes a name that is not pinned.
   - **A bare name accepted**, which the engine reads as its ``latest`` tag.

.. comp_req:: A granted folder that does not exist or is named as a path is refused
   :id: CREQ_GRANTS_REFUSES_BAD_FOLDER
   :derived_from: FEAT_TOOL_GRANTS_UNREADABLE_REFUSED
   :allocated_to: COMP_GRANTS_READER
   :ears_pattern: unwanted
   :statement: If a grants file names a folder whose path is not an existing directory or whose name is not a single plain part of a path, then Grants reader shall refuse it at that folder.

   A folder's name is where it is mounted, under ``/work``
   (``DEC_PATHS_IN_GRANTED_FOLDERS``), so ``a/b``, ``..`` or ``.`` would mount
   somewhere its name does not say. A path that is not there would be made by
   the engine when it mounts it, as a directory nobody granted.

   Failure modes:

   - **A missing folder passed to the engine**, which creates it.
   - **A file granted as a folder**, mounted where a tool expects a directory.

.. comp_req:: A manifest gives the node types it names as tools and the action of each
   :id: CREQ_PROJECT_READS_TOOLS
   :derived_from: FEAT_TOOL_PERFORMED
   :allocated_to: COMP_PROJECT_READER
   :ears_pattern: event
   :statement: When a manifest is read, Project reader shall give the node types it names under tools and the action it names for each.

   ``DEC_TOOLS_NAMED_IN_THE_MANIFEST``: a ``tools`` table beside ``scripts``
   and ``persons``, each key a node type and each value ``read``, ``write`` or
   ``run``. A tool node type is handed to the scripted run as one a person
   performs (``DEC_TOOL_PERFORMED_BY_THE_RUNNER``), so that it comes back as a
   step rather than being refused as a type with no script.

   Failure modes:

   - **A tool left out of what the scripted run is told a person performs**:
     every run with a tool is refused before it starts.
   - **The action of one tool given to another.**

.. comp_req:: A tool the manifest names wrongly is refused at that tool
   :id: CREQ_PROJECT_REFUSES_TOOL_FAULTS
   :derived_from: FEAT_TOOL_UNGRANTED_REFUSED
   :allocated_to: COMP_PROJECT_READER
   :ears_pattern: unwanted
   :statement: If a manifest names a tool with an action that is not read or write or run or names one node type as more than one of a script's and a person's and a tool's, then Project reader shall refuse the manifest at that node type.

   A node type performed two ways would be performed by whichever the runner
   asked about first, an accident of the code rather than anything the
   manifest says (``DEC_TOOLS_NAMED_IN_THE_MANIFEST``).

   Failure modes:

   - **A misspelt action passed over**, and the tool refused later as
     ungranted.
   - **A node type named as a script and a tool accepted**, and the script
     never run, or the tool never performed.

.. comp_req:: A tool step reads a file, writes one or runs a command as its action says
   :id: CREQ_PERFORMER_PERFORMS_THE_ACTION
   :derived_from: FEAT_TOOL_PERFORMED
   :allocated_to: COMP_TOOL_PERFORMER
   :ears_pattern: event
   :statement: When a tool step is handed to it, Tool performer shall perform the step's action with the inputs of that action's parameter names and give back the file read, the write done, or the command's exit status and output.

   ``DEC_THREE_TOOL_ACTIONS``: ``read`` takes ``path`` and gives the file's
   text; ``write`` takes ``path`` and ``text``, makes the folders the path
   names inside its granted folder when they are missing, replaces the file
   with the text exactly, and gives a line naming the file and its bytes;
   ``run`` takes ``command`` and gives a line with its exit status, then its
   output. Each input is the rendering of the context given for that
   parameter.

   Failure modes:

   - **Text changed on the way** - line endings translated, a trailing
     newline added or dropped: a model rewriting a file with its own content
     changes it.
   - **The status line after the output**, where a cut output can lose it.
   - **An input taken by position** rather than by parameter name: a write
     whose path and text are the wrong way round.

.. comp_req:: A path absolute or reaching a parent folder is answered with its refusal
   :id: CREQ_PERFORMER_REFUSES_PATH
   :derived_from: FEAT_TOOL_KEPT_TO_ITS_GRANT
   :allocated_to: COMP_TOOL_PERFORMER
   :ears_pattern: unwanted
   :statement: If a read or write step is given a path that is absolute or has a part that names a parent folder, then Tool performer shall answer it with text saying the path was refused and ask the sandbox for nothing.

   ``DEC_PATHS_IN_GRANTED_FOLDERS``: the container confines a path either way,
   and this tells the model plainly what it asked for. A path is split on both
   ``/`` and ``\``, since a model writes either, and a drive letter makes it
   absolute.

   Failure modes:

   - **A parent part found only at the start**: ``project/../../etc`` passed
     on.
   - **Only ``/`` treated as a separator**: ``..\x`` passed on as a file name.
   - **The sandbox asked anyway**, so a refusal still costs a container.

.. comp_req:: A tool step whose action failed is answered with what went wrong
   :id: CREQ_PERFORMER_FAILURE_AS_TEXT
   :derived_from: FEAT_TOOL_PERFORMED
   :allocated_to: COMP_TOOL_PERFORMER
   :ears_pattern: unwanted
   :statement: If a read or write fails in the container or a step lacks an input its action needs, then Tool performer shall answer the step with text saying which action failed and what the container printed.

   ``DEC_TOOL_FAILURE_IS_OUTPUT``: the model that asked can mend a misspelt
   path, and cannot mend a run that ended. A command exiting other than 0 is
   not a failure of the step at all: its status is the first line of what
   ``run`` gives back. A step lacking an input asks the sandbox for nothing,
   and its text says which input.

   Failure modes:

   - **A failed read answered with nothing**, as if the file were empty.
   - **A missing input failing the node**, or panicking.

.. comp_req:: The container engine's failure is handed back as the engine's
   :id: CREQ_PERFORMER_PASSES_ENGINE_FAILURE
   :derived_from: FEAT_TOOL_ENGINE_FAILURE_LEAVES_THE_STEP
   :allocated_to: COMP_TOOL_PERFORMER
   :ears_pattern: unwanted
   :statement: If the sandbox reports the container engine's own failure for a step, then Tool performer shall hand that failure back and no text for the step.

   ``DEC_ENGINE_FAILURE_LEAVES_THE_STEP``: an engine's failure says nothing
   about the step, so it is not the step's output.

   Failure modes:

   - **The engine's message answered as the step's text**: the model reads
     "error during connect" as the result of its command and goes on.

.. comp_req:: A step's path and text reach the container as an argument and as input
   :id: CREQ_SANDBOX_PATHS_AS_ARGUMENTS
   :derived_from: FEAT_TOOL_KEPT_TO_ITS_GRANT
   :allocated_to: COMP_SANDBOX
   :ears_pattern: event
   :statement: When a step's path or text is handed to it, Sandbox shall pass the path to the container as a separate argument and the text on standard input and never as part of the text of a command.

   ``DEC_PATHS_AS_ARGUMENTS``, measured byte for byte
   (``EVD_WRITE_THROUGH_EXEC_EXACT``).

   Failure modes:

   - **A path quoted into a command**: a file name holding a quote and a
     semicolon runs what follows them.
   - **Text passed as an argument**, over the length an argument may have.

.. comp_req:: A tool's container is locked down with only the granted folders mounted
   :id: CREQ_SANDBOX_LOCKED_DOWN
   :derived_from: FEAT_TOOL_KEPT_TO_ITS_GRANT
   :allocated_to: COMP_SANDBOX
   :ears_pattern: event
   :statement: When a container is made for a run, Sandbox shall make it with an init process, no capabilities, no new privileges, a read-only root, nothing mounted but what the run's grants name and no network unless granted.

   ``DEC_CONTAINER_OWN_USER_APART`` and ``DEC_GRANTS_NARROW_BY_DEFAULT``. Each
   folder is mounted at ``/work`` under its name, read-only unless granted
   writable; ``/tmp`` is a writable file system in memory; the container is
   labelled with the run's record file and started from the image by its
   digest, never pulled. Its own process runs with no capabilities as a user
   no step runs as - root, or 65534 beside a step run as root - so a step's
   user cannot reach it. Changed by ``DEC_CHANGE_SANDBOX_LOCKED_DOWN``, which
   stated what is mounted by what the grants name rather than by the folders
   alone.

   Failure modes:

   - **A host path mounted that the grants do not name.**
   - **A folder mounted writable that was granted read-only**, which
     ``EVD_CONTAINER_CONFINES_TO_MOUNTS`` shows emptied through the container.
   - **The engine's default network left on.**
   - **The container's own process run as the step's user**, which the
     cleanup after each step then kills, ending the container.

.. comp_req:: A step runs as the person's user on Linux
   :id: CREQ_SANDBOX_STEP_USER
   :derived_from: FEAT_TOOL_KEPT_TO_ITS_GRANT
   :allocated_to: COMP_SANDBOX
   :ears_pattern: event
   :statement: When a step is run, Sandbox shall run it as a user whose files the person running the run can change and who can change no file on the host that person could not.

   Revised by ``DEC_CHANGE_SANDBOX_STEP_USER``, which named the user rather
   than what the parent needs of it. ``DEC_STEP_USER_ROOT_INCLUDED`` says
   which: on Linux the person's own user and group, read from
   ``/proc/self/status``, root included; elsewhere user and group 1000, whose
   files a Windows user can change.

   Failure modes:

   - **Root, for a person who is not**: a step writes files the person cannot
     delete, and changes what they could not.
   - **The lines of the status file misread**, and a step run as another
     user.

.. comp_req:: What a killed run left is removed before a tool step of its run
   :id: CREQ_SANDBOX_REMOVES_LEFTOVERS
   :derived_from: FEAT_TOOL_INTERRUPTED_PERFORMED_AGAIN
   :allocated_to: COMP_SANDBOX
   :ears_pattern: event
   :statement: When a container is made for a run, Sandbox shall first remove every container labelled with that run's record file.

   ``DEC_LEFTOVER_CONTAINERS_REMOVED``. A container is made only once the
   record file's lock is held, so every container carrying its label belongs
   to a process that has ended.

   Failure modes:

   - **Only stopped containers removed**: the running one a killed process
     left, the one that matters, stays.
   - **Containers of other record files removed**, and a run in another
     process losing its container mid-step.

.. comp_req:: A command past its time is ended with what it printed kept
   :id: CREQ_SANDBOX_TIME_LIMIT
   :derived_from: FEAT_TOOL_PERFORMED
   :allocated_to: COMP_SANDBOX
   :ears_pattern: unwanted
   :statement: If a step's command runs past the grants' time limit, then Sandbox shall end it and hand back what it printed before it was ended with the status it was ended with.

   ``DEC_COMMAND_WRAPPED``: ended by ``timeout`` inside the container, since
   a runner that stops waiting leaves it running
   (``EVD_EXEC_OUTLIVES_ITS_CLIENT``). A step whose command never returns is a
   run that never goes on, which the requirement this derives from rules out.

   Failure modes:

   - **The limit kept on the host**: the runner goes on and the command keeps
     running in the container.
   - **Output lost when a command is ended**, so a model never sees how far
     it got.

.. comp_req:: Nothing a step started is left running after it
   :id: CREQ_SANDBOX_NOTHING_LEFT_RUNNING
   :derived_from: FEAT_TOOL_PERFORMED
   :allocated_to: COMP_SANDBOX
   :ears_pattern: event
   :statement: When a step's command ends, Sandbox shall end every process the step's user has in the container before handing the step back.

   ``DEC_COMMAND_WRAPPED``: ``timeout`` ends only its own child
   (``EVD_TIMEOUT_SPARES_GRANDCHILDREN``), and killing every process of the
   step's user reaches the rest (``EVD_KILL_ALL_AFTER_A_COMMAND``). A step is
   whole in itself; a server started in one is gone by the next.

   Failure modes:

   - **A background process left**, still writing into a folder while the
     next step reads it.
   - **The container's own process killed** by the cleanup, and every later
     step failing as the engine's failure.

.. comp_req:: Output past its limit keeps its first and last halves
   :id: CREQ_SANDBOX_OUTPUT_LIMIT
   :derived_from: FEAT_TOOL_PERFORMED
   :allocated_to: COMP_SANDBOX
   :ears_pattern: unwanted
   :statement: If a step's output is longer than the grants' output limit, then Sandbox shall hand back its first and last halves of that limit with a line between them saying how many bytes were cut.

   ``DEC_OUTPUT_KEEPS_BOTH_ENDS``, cut inside the container so that only what
   is kept crosses to the host (``EVD_EXEC_COST``). Output up to the limit is
   handed back whole. A character of UTF-8 broken by a cut is replaced where
   it was cut, so what comes back is text.

   Failure modes:

   - **The count of bytes cut wrong**, off by the halves' rounding.
   - **Output cut through a character** and the step failed for not being
     text.
   - **Output exactly at the limit cut.**

.. comp_req:: The engine's own failure is told apart from a command's status
   :id: CREQ_SANDBOX_ENGINE_FAILURE_APART
   :derived_from: FEAT_TOOL_ENGINE_FAILURE_LEAVES_THE_STEP
   :allocated_to: COMP_SANDBOX
   :ears_pattern: unwanted
   :statement: If the container engine fails to make a container or to run a step in it, then Sandbox shall report the engine's failure with its message rather than a status and output of the step.

   ``DEC_COMMAND_WRAPPED``: ``docker`` exits 1 for its own failure as a
   command exiting 1 does, and only the command's status is written alone to
   standard error (``EVD_STATUS_APART_FROM_DOCKER_FAILURE``).

   Failure modes:

   - **An unreachable engine reported as a command exiting 1**: the model is
     told its command failed and tries another.
   - **A command whose own status is 125** - the status ``docker run`` gave
     for its own failure (``EVD_PULL_NEVER_REFUSES_ABSENT``) - reported as the
     engine's failure.

.. comp_req:: A run's container is removed when the call that made it stops
   :id: CREQ_SANDBOX_REMOVED_AT_THE_END
   :derived_from: FEAT_TOOL_KEPT_TO_ITS_GRANT
   :allocated_to: COMP_SANDBOX
   :ears_pattern: event
   :statement: When the start or resume or answer that made a container stops, Sandbox shall remove that container.

   ``DEC_ONE_CONTAINER_PER_CALL``, whatever the run's ending, the step it
   awaits, a refusal or an engine's failure: a container left behind keeps
   the granted folders mounted into something nobody is watching.

   Failure modes:

   - **Removed only when the run completes**, and left behind by a run that
     awaits a person for days.
   - **A panic or an early return skipping the removal.**

.. comp_req:: An image absent or lacking what every step needs is reported before the run
   :id: CREQ_SANDBOX_IMAGE_READY
   :derived_from: FEAT_TOOL_UNGRANTED_REFUSED
   :allocated_to: COMP_SANDBOX
   :ears_pattern: unwanted
   :statement: If the grants' image is not present or holds no sh or timeout, then Sandbox shall report which it lacks without pulling the image.

   ``DEC_IMAGE_BY_DIGEST_NEVER_PULLED``, asked before any node runs: presence
   in about 140 ms (``EVD_CONTAINER_START_COST``), and what it holds in one
   short container of its own, removed at once.

   Failure modes:

   - **The image pulled when absent**: network use nobody granted, and
     whatever the registry holds that day.
   - **An image lacking timeout accepted**, and every command run with no
     limit on its time.

.. comp_req:: A step of a tool is performed and answered by the runner
   :id: CREQ_RUNNER_PERFORMS_TOOLS
   :derived_from: FEAT_TOOL_PERFORMED, FEAT_TOOL_INTERRUPTED_PERFORMED_AGAIN
   :allocated_to: COMP_RUNNER
   :ears_pattern: event
   :statement: When the scripted run awaits a step of a node type the manifest names as a tool, Runner shall have the tool performer perform it and answer the step with the text it gave, keeping each record.

   ``DEC_TOOL_PERFORMED_BY_THE_RUNNER``. So whether the run was started,
   resumed or answered: a resumed run awaiting a tool step performs it. A
   step a person performs is handed back as before.

   Failure modes:

   - **A tool step handed back to the person** as if it were theirs: every
     run with a tool stops at its first one.
   - **Only the first awaited tool step performed**, and the run handed back
     at the second.

.. comp_req:: A run with tools asked for without readable grants is refused before it starts
   :id: CREQ_RUNNER_REFUSES_WITHOUT_GRANTS
   :derived_from: FEAT_TOOL_GRANTS_UNREADABLE_REFUSED
   :allocated_to: COMP_RUNNER
   :ears_pattern: unwanted
   :statement: If a run whose manifest names a tool is asked for without grants or the grants reader refuses them, then Runner shall hand back that refusal having started no run and taken no record file.

   The grants are asked for on every start, resume and answer
   (``DEC_GRANTS_IN_A_FILE_OF_THEIR_OWN``). A run whose manifest names no tool
   needs none, and grants given for one are read and otherwise unused.

   Failure modes:

   - **Missing grants found at the first tool step**, after every step before
     it was paid for.
   - **The record file taken before the grants are read**, and a refused
     answer leaving it held for no run.

.. comp_req:: A tool its grants do not provide for is refused before the run starts
   :id: CREQ_RUNNER_REFUSES_UNGRANTED
   :derived_from: FEAT_TOOL_UNGRANTED_REFUSED
   :allocated_to: COMP_RUNNER
   :ears_pattern: unwanted
   :statement: If a tool the manifest names has an action its grants do not allow or lacks a parameter its action takes or the image is absent or lacks sh or timeout, then Runner shall refuse the run before any node runs naming each tool and why.

   Revised by ``DEC_CHANGE_RUNNER_REFUSES_UNGRANTED``, which read "not ready"
   as covering an engine that cannot be reached. It does not: such a run goes
   on, and the first tool step it reaches is left awaiting with the engine's
   failure (``DEC_UNREACHABLE_ENGINE_REFUSES_NO_RUN``,
   ``CREQ_RUNNER_ENGINE_FAILURE_STOPS``).

   Found before the record file is taken. A parameter is found when the
   tool's node type declares it, as required or optional; one declared and
   left unbound at a step is that step's failure
   (``CREQ_PERFORMER_FAILURE_AS_TEXT``).

   Failure modes:

   - **Only the first ungranted tool named**, and the person fixing the
     grants one run at a time.
   - **An optional parameter taken as missing**, refusing a workflow that is
     sound.

.. comp_req:: A step the engine could not perform stops the run awaiting it
   :id: CREQ_RUNNER_ENGINE_FAILURE_STOPS
   :derived_from: FEAT_TOOL_ENGINE_FAILURE_LEAVES_THE_STEP
   :allocated_to: COMP_RUNNER
   :ears_pattern: unwanted
   :statement: If the tool performer hands back the container engine's failure for a step, then Runner shall hand back the step as awaited with the engine's failure beside it and keep the record that awaits it.

   ``DEC_ENGINE_FAILURE_LEAVES_THE_STEP``: the run stops as it does for a
   person's step, and the record in its file awaits the same step, so a
   resume performs it again and an answer supplies it.

   Failure modes:

   - **The run ended as a failed node**, unresumable over a failure of the
     machine.
   - **The engine's failure dropped**, and the person told a step awaits them
     with no reason why a tool's step does.

.. comp_req:: A check with grants reports what would refuse a run's tools
   :id: CREQ_RUNNER_CHECKS_TOOLS
   :derived_from: FEAT_RUNNER_CHECKS_WITHOUT_RUNNING
   :allocated_to: COMP_RUNNER
   :ears_pattern: event
   :statement: When a check is asked for of a manifest naming a tool, Runner shall also hand back the grants' absence or faults and every tool whose action or parameter is not provided for or whose image is not ready.

   What a run would be refused for is what a check reports, and with tools a
   run is refused for its grants too.

   Failure modes:

   - **A check that reports nothing about tools**, and the person finding the
     ungranted one by starting a run.
   - **A check that makes the run's container**, with the granted folders
     mounted, where finding the image ready is all it asks of the engine.

.. comp_req:: The grants file typed is handed to the runner
   :id: CREQ_COMMAND_READS_GRANTS
   :derived_from: FEAT_TOOL_GRANTED_BY_THE_PERSON
   :allocated_to: COMP_COMMAND_LINE
   :ears_pattern: event
   :statement: When a person types a command with a grants file, Command line shall hand the runner that file's path with the command.

   ``--grants``, for ``run``, ``resume``, ``answer`` and ``check`` alike, as
   ``--models`` is.

   Failure modes:

   - **The grants dropped for one command**: a resume refused for lacking
     grants the person gave.

.. comp_req:: A step left by the engine's failure is told with the failure
   :id: CREQ_COMMAND_TELLS_ENGINE_FAILURE
   :derived_from: FEAT_TOOL_ENGINE_FAILURE_LEAVES_THE_STEP
   :allocated_to: COMP_COMMAND_LINE
   :ears_pattern: unwanted
   :statement: If the runner hands back a step left awaiting by the container engine's failure, then Command line shall print the step as it prints any awaited step and the engine's failure to standard error.

   ``DEC_ENGINE_FAILURE_LEAVES_THE_STEP``: the step is awaited, so the status
   is the one for awaiting a person (``DEC_EXIT_STATUS_PER_ENDING``), and what
   the person reads says why a tool's step awaits them and that resuming
   performs it again.

   Failure modes:

   - **A status of its own**, a way of stopping the runner does not have.
   - **The failure on standard output**, where a script reading the awaited
     step reads it as the step.
