======================================
Decisions about a tool a model can use
======================================

How a node type reaches the world outside the engine - reading a file,
writing one, running a command - as ``STKH_TOOLS_AS_NODES`` asks, and how what
it can change is kept to what the person running the workflow granted, as
``STKH_TOOLS_CONFINED`` asks: what performs a tool, where a workflow says which
node types are tools and a person says what they may do, what confines them,
and what a person is told when a tool could not be performed.

Nineteen rest on measurements recorded in ``evidence/tools``. The other nine
are judgements between the alternatives each names. Four of the twenty-eight
are superseded by later ones, and say so.

What they do not settle is named here. Nothing below decides a tool reached
through MCP, a tool of any kind but the three actions, grants that differ from
one container to another, a network reachable only in part, or a file that is
not text. Each is one
more action, or one more field of a grant, of the shape these decisions give,
and waits for a workflow that needs it. Nothing decides how a person reviews
what a tool changed inside a writable grant; ``STKH_TOOLS_CONFINED`` allows
any change there, mistakes included; the README is to advise granting a git
worktree.

.. dec:: A tool is a node type the runner performs as it would hand a person the step
   :id: DEC_TOOL_PERFORMED_BY_THE_RUNNER
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_ANSWER_COST_FLAT, EVD_CALL_ANSWER_COST_LINEAR
   :statement: Agconflo shall perform a tool as a node type whose activations the runner is handed as a person's step and answers with the text the tool produced.

   ``DEC_PERSON_NAMED_BY_CALLER`` takes which node types a person performs from
   the caller, and its body says the engine cannot tell whether whoever answers
   is a person and does not try. A tool is one more thing the caller answers a
   step with. Everything a step already is comes with it: the tool's output is
   an output of the run, recorded, counted against the budget
   (``DEC_BUDGET_COUNTS_ACTIVATIONS``), and never performed again once
   recorded; a model calls a tool as it calls any node type its instance
   declares (``DEC_CALL_IS_AN_ACTIVATION``); and a node's script keeps
   touching nothing outside the run, as ``DEC_HOST_FUNCTIONS_CONTEXT_API``
   has it.

   Answering from outside the run replays its record each time, and measured
   that costs nothing that matters: 150 answers to a chain of steps took 50 to
   61 ms each, process start included (``EVD_ANSWER_COST_FLAT``), and 40
   answers to one model's calls took 59 to 106 ms, the record growing by about
   2.8 kB a call (``EVD_CALL_ANSWER_COST_LINEAR``).

   Host functions a script calls - ``host.run``, ``host.write`` - were the
   alternative. A file written or a command run from inside a script happens
   inside an activation, which an interruption repeats whole
   (``EVD_WORK_INSIDE_AN_ACTIVATION_REPEATS``), outside the budget, and
   recorded nowhere unless the record gains a kind of entry of its own - the
   shape ``DEC_PERSON_PERFORMS_AN_ACTIVATION`` rejected for a person on the
   same measurement.

.. dec:: Which node types are tools, and what each does, is named in the manifest
   :id: DEC_TOOLS_NAMED_IN_THE_MANIFEST
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall take the node types a tool performs, and the action each performs, from the manifest, beside its scripts and the node types a person performs.

   What a node type does is part of what the workflow is: a type called
   ``run_tests`` means running a command, whoever runs the workflow. The
   manifest already says what performs every other type - a script or a person
   (``DEC_RUN_FROM_A_MANIFEST``) - and a tool is one more answer to the same
   question.

   The grants file was the alternative, and the first form of this plan. A
   workflow handed to someone else would then not run until they wrote grants
   that also said what its node types mean, and a grants file would be tied to
   the one workflow it was written for. What the person decides - whether a
   tool may do what it does - stays theirs, in the grants
   (``DEC_GRANTS_IN_A_FILE_OF_THEIR_OWN``).

   A node type named as more than one of a script's, a person's and a tool's is
   refused, since which of them performs it would otherwise be decided by the
   order the runner happens to ask in.

.. dec:: A tool reads a file, writes one, or runs a command
   :id: DEC_THREE_TOOL_ACTIONS
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall give a tool one of three actions, read taking a path, write taking a path and text, and run taking a command, each from the node type's parameter of that name.

   Three are what a model needs to work on a project in a folder: look at it,
   change it, and build or test it. ``read`` gives the file's text; ``write``
   gives a line naming the file and how many bytes were written; ``run`` gives
   a line with the command's exit status and then everything it printed,
   standard output and standard error together as they were printed. The
   status comes first so that an output cut to its limit never hides it.

   Taking the inputs from parameters of fixed names keeps a tool node type an
   ordinary declaration (``STKH_NO_PRIVILEGED_TYPES``): its description and
   parameters are what a model is offered (``DEC_TOOLS_OFFERED_AS_CONTEXTS``),
   and a type lacking the parameter its action needs is refused before the run
   starts. A mapping from action inputs to parameter names in the manifest was
   the alternative; it costs a table per tool for a freedom no workflow has
   asked for.

   ``run`` alone would do everything the other two do. They are kept because
   a grant can allow ``read`` without ``run``, and a model reading a file
   through ``cat`` would be granted everything a shell can do.

.. dec:: What a run's tools may do is read from a grants file of its own
   :id: DEC_GRANTS_IN_A_FILE_OF_THEIR_OWN
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall read what a run's tools are granted, the image, folders, network, actions and limits, from a grants file named when the run is asked for, apart from its manifest.

   ``STKH_TOOLS_CONFINED`` makes the grant the person's, made when they run the
   workflow, and neither the workflow's nor the model's. A grant inside the
   manifest would arrive with a workflow downloaded from anyone, written by
   its author. It is apart from the model mapping too, as that is apart from
   the manifest (``DEC_MODELS_IN_A_FILE_OF_THEIR_OWN``): which models a
   machine reaches and what a tool may touch on it are two questions, and a
   person may keep one answer for several workflows and change the other.

   The file is asked for on every start, resume and answer of a run whose
   manifest names a tool, and whatever it grants then is what holds. A person
   narrowing a grant between two steps is doing what the goal gives them the
   right to do.

   The limits are the seconds a command may run and the bytes of output a step
   gives back, 60 and 16384 unless the file says otherwise. A command a small
   model makes up can run for ever - ``yes``, a server, ``find /`` - and
   output it cannot use costs the whole of its window.

.. dec:: A grant is read-only and offline unless it says otherwise
   :id: DEC_GRANTS_NARROW_BY_DEFAULT
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_CONTAINER_CONFINES_TO_MOUNTS
   :statement: Agconflo shall mount a granted folder read-only and give a run's tools no network unless the grants file marks that folder writable or grants the network.

   A read-only mount refused ``rm -rf``, and a writable one was emptied through
   the same container on the host (``EVD_CONTAINER_CONFINES_TO_MOUNTS``): the
   container keeps everything it was not given and nothing it was. A folder
   is therefore given for writing only by a line saying so, and forgetting that
   line fails safe.

   The network is off for the same reason, and for one of its own: with it, a
   command a model makes up can fetch and run anything, which no grant of
   folders describes.

.. dec:: The image is named by its digest and never pulled
   :id: DEC_IMAGE_BY_DIGEST_NEVER_PULLED
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_PULL_NEVER_REFUSES_ABSENT, EVD_CONTAINER_START_COST
   :statement: Agconflo shall refuse a grants file naming its image by anything but a digest or an image id, and refuse to start a run whose image is not present, without pulling it.

   An image is what a tool can run, so it is part of the grant, and a tag can
   be moved to another image between two runs without the grants file
   changing. A digest cannot, and an image id names one built locally.

   Whether it is present is found in about 140 ms without anything run in it
   (``EVD_CONTAINER_START_COST``), so it is checked before the run starts,
   with the ``sh`` and ``timeout`` every step needs. Pulling on demand was the
   alternative: it uses the network nobody granted, midway through a run, and
   what arrives is whatever the registry holds that day.

.. dec:: A run's tool steps are performed in one container for each time the run is asked for
   :id: DEC_ONE_CONTAINER_PER_CALL
   :dec_status: superseded
   :decided_on: 2026-09-26
   :supported_by: EVD_EXEC_COST, EVD_CONTAINER_START_COST, EVD_CONTAINER_OUTLIVES_ITS_CLIENT
   :statement: Agconflo shall perform the tool steps of one start, resume or answer of a run in one container, made at its first tool step, labelled with the run's record file, and removed when it stops.

   A command run in a container already running costs about 160 ms
   (``EVD_EXEC_COST``), and starting a container and removing it again adds
   about 500 ms (``EVD_CONTAINER_START_COST``). A model working on a project
   makes tens of calls, and one container for the steps of one call to the
   runner pays the second cost once rather than on every step.

   One container for the whole run was the alternative. A run is resumed in
   another process, hours later by ``STKH_RESUMABLE_RUN``'s own reckoning, and
   a container outlives the process that started it
   (``EVD_CONTAINER_OUTLIVES_ITS_CLIENT``), so one kept between calls is one
   nothing is known to be using. A container per step was the other, and
   costs the second on every step.

   A container is made at the first tool step rather than at the start, so a
   run that reaches no tool, or only persons, starts none.

.. dec:: Containers left by a run that was killed are removed before its next tool step
   :id: DEC_LEFTOVER_CONTAINERS_REMOVED
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_CONTAINER_OUTLIVES_ITS_CLIENT, EVD_EXEC_OUTLIVES_ITS_CLIENT, EVD_FILE_LOCK_DIES_WITH_ITS_PROCESS
   :statement: Agconflo shall remove every container labelled with a run's record file once it holds that file's lock and before it performs any tool step of the run.

   A runner that is killed leaves its container running, and in it whatever
   command was running (``EVD_CONTAINER_OUTLIVES_ITS_CLIENT``,
   ``EVD_EXEC_OUTLIVES_ITS_CLIENT``). A run resumed then performs the
   interrupted step again, and the command from before would still be writing
   into the same folder as the one performing it.

   The record file's lock is what makes removing them safe: it is held by one
   process at a time and freed when that process ends
   (``EVD_FILE_LOCK_DIES_WITH_ITS_PROCESS``), so a container labelled with a
   record file whose lock this process holds belongs to no process still
   running. The label is the record file's full path.

   Removing containers by age was the alternative, and would remove one a run
   in another process is using.

.. dec:: A tool's container is locked down and its steps run as an unprivileged user
   :id: DEC_CONTAINER_LOCKED_DOWN
   :dec_status: superseded
   :decided_on: 2026-09-26
   :supported_by: EVD_CONTAINER_CONFINES_TO_MOUNTS, EVD_KILL_ALL_AFTER_A_COMMAND
   :statement: Agconflo shall start a tool's container with an init process, no capabilities, no new privileges and a read-only root, and perform each tool step in it as an unprivileged user.

   The confinement ``EVD_CONTAINER_CONFINES_TO_MOUNTS`` measured was of this
   container: dropped capabilities, no new privileges, a read-only root with
   a writable ``/tmp``, and only the granted folders mounted. Each is one more
   way a mistaken command reaches less.

   The container's own process runs as root, with no capabilities, and each
   step runs through ``docker exec`` as the unprivileged user. That split is
   what lets a step's leftovers be killed without killing the container: as
   the step's user, killing every process reached everything it left and not
   the container's own, and the init process collected them
   (``EVD_KILL_ALL_AFTER_A_COMMAND``). Without it they stayed as zombies.

   ``STKH_TOOLS_CONFINED`` is met against accident, not attack. None of this
   is claimed against code built to escape a container.

.. dec:: A tool step runs as the person's user on Linux
   :id: DEC_STEP_USER_IS_THE_PERSONS
   :dec_status: superseded
   :decided_on: 2026-09-26
   :supported_by: EVD_MOUNT_OWNERSHIP_ON_LINUX, EVD_UID_READABLE_FROM_PROC
   :statement: Agconflo shall perform tool steps on Linux as the user and group of the process running the run, read from /proc/self/status, and elsewhere as user and group 1000.

   On Linux a file a step writes belongs on the host to the user the step ran
   as (``EVD_MOUNT_OWNERSHIP_ON_LINUX``): as the person, what a tool writes is
   theirs to edit and delete, and what they could not read the tool cannot
   either. ``/proc/self/status`` gives the user and group without the
   ``unsafe`` a system call would need, which the workspace forbids
   (``EVD_UID_READABLE_FROM_PROC``).

   On Windows a mounted folder's files belong to the Windows user whoever
   wrote them (``EVD_CONTAINER_CONFINES_TO_MOUNTS``), so any user but root
   serves, and 1000 is the one every measurement here ran as.

   An engine running without root maps a container's users to others on the
   host, and was not measured; a run on one may find its files owned by
   someone else.

.. dec:: A command is run inside the container under a timeout, and everything it leaves is killed
   :id: DEC_COMMAND_WRAPPED
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_EXEC_OUTLIVES_ITS_CLIENT, EVD_TIMEOUT_SPARES_GRANDCHILDREN, EVD_KILL_ALL_AFTER_A_COMMAND, EVD_STATUS_APART_FROM_DOCKER_FAILURE
   :statement: Agconflo shall run a tool step's command in the container under timeout, kill every process of its user once it ends, and write its output to standard output and its exit status alone to standard error.

   Killing ``docker exec`` leaves its command running
   (``EVD_EXEC_OUTLIVES_ITS_CLIENT``), so a limit kept by the runner would stop
   waiting and leave the command at work. ``timeout`` inside the container
   ends the command, but only the process it started
   (``EVD_TIMEOUT_SPARES_GRANDCHILDREN``); what that process started goes on.
   Killing every process of the step's user once it ends reaches all of them
   (``EVD_KILL_ALL_AFTER_A_COMMAND``). A command that means to leave a server
   running between steps therefore cannot, and that is the cost: each step is
   whole in itself.

   ``docker exec`` exits 1 when the engine cannot be reached, as a command
   exiting 1 does. With the output on standard output and nothing but the
   status on standard error, anything else there is the engine's own failure
   (``EVD_STATUS_APART_FROM_DOCKER_FAILURE``), which
   ``DEC_ENGINE_FAILURE_LEAVES_THE_STEP`` treats as a step not performed. The
   output is written to a file in ``/tmp`` first, so what a command printed
   before it was ended is kept.

.. dec:: A step's output past its limit keeps its start and its end
   :id: DEC_OUTPUT_KEEPS_BOTH_ENDS
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_EXEC_COST
   :statement: Agconflo shall give a tool step's output whole up to the grant's limit and past it its first and last halves of that limit with a line saying how many bytes were cut between them.

   A build or a test run says what went wrong at its end as often as at its
   start, and a model shown only one of them fixes the wrong thing. Returning
   50 MB took five seconds (``EVD_EXEC_COST``), so the output is cut inside the
   container, where it was written, and only what is kept crosses to the host.

   Cutting at the limit and dropping the rest was the alternative; it keeps the
   start of a compiler's output and loses the summary at its end.

.. dec:: Paths and text reach the container as arguments and input, never as command text
   :id: DEC_PATHS_AS_ARGUMENTS
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_WRITE_THROUGH_EXEC_EXACT
   :statement: Agconflo shall hand a read or write step's path to the container as a separate argument and its text on standard input, never as part of the text of a command.

   A path a model gives may hold a quote, a space or a ``$``. Passed as an
   argument of ``sh -c`` it is never read as shell, and text piped through
   ``docker exec -i`` arrived byte for byte, line endings and UTF-8 included
   (``EVD_WRITE_THROUGH_EXEC_EXACT``). Quoting a path into the command was the
   alternative, and is one escaping mistake from running part of a file name.

   A ``run`` step's command is the one thing that is shell, since running it
   is what was granted.

.. dec:: A path names a granted folder and a place inside it
   :id: DEC_PATHS_IN_GRANTED_FOLDERS
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_LINKS_RESOLVE_IN_THE_CONTAINER
   :statement: Agconflo shall mount each granted folder under /work by its grant's name, run commands in /work, and answer a read or write step whose path is absolute or climbs to a parent folder with its refusal.

   One way of naming a file serves both a command and a read: ``project/src``
   is the same place to ``cat`` in ``/work`` as to a ``read`` step. The folder
   names are the grant's, so a workflow's node type descriptions and a
   person's grants agree on them by name.

   The container is what confines a path (``EVD_CONTAINER_CONFINES_TO_MOUNTS``);
   a link a tool makes leads, inside it, only to what is inside it
   (``EVD_LINKS_RESOLVE_IN_THE_CONTAINER``). The refusal of absolute paths and
   ``..`` is there for the model, which is told plainly what it asked for
   instead of reaching whatever ``/etc`` the image holds.

.. dec:: A tool's own failure is its output
   :id: DEC_TOOL_FAILURE_IS_OUTPUT
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall answer a tool step whose action failed, by a refused path, a missing file or a command exiting other than 0 among others, with text saying so rather than fail its node.

   The model that called a tool is the one who can do something about a path
   it misspelt or a test that failed, and it can only if it is told. A small
   model mistypes; failing the node on each mistake ends a run that one more
   call would have mended.

   Failing the node was the alternative, and would need a way to fail a step
   from outside the run that the person route does not have: a step handed to
   the caller is answered with text (``DEC_PERSON_SUPPLIES_TEXT``). A wired
   tool whose output only a script reads gets the same text, and a script that
   needs to know can look for it.

.. dec:: A step the container engine could not perform is left awaiting
   :id: DEC_ENGINE_FAILURE_LEAVES_THE_STEP
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_STATUS_APART_FROM_DOCKER_FAILURE
   :statement: Agconflo shall stop a run at a tool step the container engine failed to perform, leaving the step awaiting and telling the person why, rather than end the run.

   An engine that stopped, a container removed from outside: neither says
   anything about the step, and the run is resumable from where it stood. The
   step is awaited as a person's would be, so the person can resume the run
   once the engine is back, which performs the step again, or answer the step
   themselves; ``DEC_PERSON_NAMED_BY_CALLER`` leaves who answers to the caller.

   Ending the run as a failed node was the alternative. It needs a way to fail
   a step handed to the caller, which ``agconflo-lua`` does not give, and
   throws away a run over a failure of the machine rather than of the
   workflow. A new ending of its own was the other: the runner performs a run
   until it ends or awaits a person, and a third way of stopping would be a
   stop neither.

.. dec:: The container engine is reached through its command
   :id: DEC_ENGINE_THROUGH_ITS_COMMAND
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall reach the container engine by running the docker command, one step at a time on the runner's thread, rather than through a client library.

   The ``docker`` command is what a person installing Docker has, on Windows
   and on Linux alike, and it finds its engine the way the person's own
   commands do - its context, ``DOCKER_HOST`` - with nothing for Agconflo to
   configure. A client library was the alternative: a dependency tree of its
   own, Windows' named pipe and Linux's socket to find, and a second account
   of where the engine is that can disagree with the person's.

   A step's command blocks the runner's thread while it runs. Nothing else runs
   on it meanwhile: a step is performed between two calls into the scripted
   run (``DEC_RUNNER_ON_ONE_THREAD``), not during one.

.. dec:: The tests that confine a tool need Docker and fail without it
   :id: DEC_TESTS_NEED_DOCKER
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall need Docker and the image its tests name to run its tests, failing a test that uses them with a message naming what is missing rather than skipping it.

   Whether a tool is kept to its grant is a property of the container, and a
   test that pretends to be one checks the pretence. A skipped test is a gate
   that goes green having checked nothing, which ``AGENTS.md`` names as the
   failure to fear most. The runner's own logic - which steps are tools, what
   is answered, what is refused - is tested without a container, against a
   stand-in, and only what the container does needs one.

   Docker becomes a prerequisite of development like Rust, and the image is
   pulled by its digest in continuous integration, as a step of its own.

.. dec:: A step whose wrapper ended without its status is the engine's failure only once its container is gone
   :id: DEC_KILLED_WRAPPER_TOLD_BY_ITS_CONTAINER
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_REMOVED_CONTAINER_LOOKS_KILLED
   :statement: Agconflo shall take a tool step whose wrapper was ended by a signal as the step's own result while its container still runs, and as the container engine's failure once it does not.

   ``DEC_COMMAND_WRAPPED`` tells a command's status from the engine's failure
   by what the wrapper writes to standard error. A command can end the
   wrapper itself: it runs as the same user, and a ``kill -9 -1`` or a
   ``pkill`` a model makes up reaches it. ``docker exec`` then reports exactly
   what it reports for a container removed during the step
   (``EVD_REMOVED_CONTAINER_LOOKS_KILLED``), and taking both as the engine's
   failure stops a run over a command, which ``DEC_TOOL_FAILURE_IS_OUTPUT``
   answers with text instead.

   Whether the container still runs tells the two apart, at the cost of one
   ``docker inspect`` for a step that ended so. Once it is the step's own
   doing, whatever the step's user left is killed, as the wrapper would have.
   Running the wrapper as a user the step cannot signal was the alternative:
   switching to the step's user from another needs a capability the container
   drops (``DEC_CONTAINER_LOCKED_DOWN``).

   The status is the last line the wrapper writes to standard error. A command
   that removes the wrapper's output file makes the wrapper say so first, and
   that line is added to the step's output rather than taken for the engine's.

.. dec:: A tool may name its image and its container beside its action
   :id: DEC_TOOL_NAMES_ITS_ENVIRONMENT
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall let a manifest name beside a tool's action an image by digest or image id and a container, and perform a tool naming neither in the grants' image and the container every such tool shares.

   ``STKH_TOOL_ENVIRONMENT`` has the workflow name where each tool runs, and
   the manifest is where it already says what each node type does
   (``DEC_TOOLS_NAMED_IN_THE_MANIFEST``). A tool's entry is its action, as
   before, or a table of the action, the image and the container. A tool
   naming no image runs in the grants' image, and one naming no container in
   the container of that name every tool naming none shares, so a manifest
   written for the first slice of tools means what it meant.

   Naming the container apart from the image is what the goal asks for: a
   build and the tests of what it built share one container by naming it,
   while a third tool of the same image names another and sees nothing of
   theirs. A container per node type was the alternative, and no two tools
   could then share what one of them left.

   The grants mapping container names to images was the other, which keeps
   the image wholly the person's but ties a grants file to one workflow's
   names, and puts what a workflow means in a file written for one machine.
   The maintainer chose the manifest; the grants still decide which images a
   run may use at all (``DEC_GRANTS_LIST_IMAGES``).

.. dec:: A tool runs only in an image its grants name
   :id: DEC_GRANTS_LIST_IMAGES
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall perform a tool only in an image its grants name as their image or among their images, and refuse before it starts a run whose manifest gives a tool any other.

   An image is what a tool can run, so it stays part of the grant
   (``STKH_TOOLS_CONFINED``): a manifest asking for an image the person has
   not listed is refused, as one asking for an action they have not allowed
   is. The grants' ``image`` stays required and is the image of every tool
   naming none; ``images`` lists the others a tool may name.

   Making ``image`` optional when every tool names its own was the
   alternative, and would change what a grants file lacking one means - a
   refusal today - to save one line.

.. dec:: A run's tool steps are performed in one container for each name, each time the run is asked for
   :id: DEC_ONE_CONTAINER_PER_NAME
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_EXEC_COST, EVD_CONTAINER_START_COST, EVD_CONTAINER_OUTLIVES_ITS_CLIENT, EVD_CONTAINER_NAME_IN_USE
   :supersedes: DEC_ONE_CONTAINER_PER_CALL
   :statement: Agconflo shall perform the tool steps of one start, resume or answer of a run in one container per container name its tools use, each made at its first step, labelled and named after the run, and removed when the call stops.

   What ``DEC_ONE_CONTAINER_PER_CALL`` decided stands for each name: a
   container made at the first step that needs it, shared by the steps of one
   call to the runner, removed when the call stops, and never kept between
   calls, since nothing could then tell whether another process was using it.
   Its one container becomes one per name, since a tool may now name its own
   (``DEC_TOOL_NAMES_ITS_ENVIRONMENT``).

   Two tools naming one container must name one image, and a manifest that
   does otherwise is refused when it is read: a container has one image.

   Each container is named ``agconflo-``, eight hexadecimal digits of a hash
   of the run's record file, ``-`` and its name, so the person finds a run's
   containers by name in ``docker ps``, as the maintainer asked; the record
   file is the run's identity, since one run at a time holds it. A container
   name is lower-case letters, digits, ``_``, ``.`` and ``-``, beginning with a
   letter or a digit, which the engine accepts in a name. A name in use
   refuses a new container (``EVD_CONTAINER_NAME_IN_USE``), which is why every
   container carrying the run's label is removed before any is made
   (``DEC_LEFTOVER_CONTAINERS_REMOVED``): a killed run's container holds its
   name until then.

.. dec:: A tool step runs as the person's user on Linux unless that is root
   :id: DEC_STEP_USER_NEVER_ROOT
   :dec_status: superseded
   :decided_on: 2026-09-26
   :supported_by: EVD_MOUNT_OWNERSHIP_ON_LINUX, EVD_UID_READABLE_FROM_PROC, EVD_KILL_ALL_AFTER_A_COMMAND
   :supersedes: DEC_STEP_USER_IS_THE_PERSONS
   :statement: Agconflo shall perform tool steps on Linux as the user and group of the process running the run unless that user is root, and otherwise as user and group 1000.

   What ``DEC_STEP_USER_IS_THE_PERSONS`` decided stands for a person who is
   not root: a file a step writes is theirs, and what they could not read the
   step cannot either (``EVD_MOUNT_OWNERSHIP_ON_LINUX``). For a person who is
   root it cannot hold beside ``DEC_CONTAINER_LOCKED_DOWN``, which runs every
   step as an unprivileged user: as root, the step's cleanup - killing every
   process of its user - reaches the container's own process
   (``EVD_KILL_ALL_AFTER_A_COMMAND``), and every later step fails as the
   engine's failure.

   User 1000 serves a root person as it serves anyone off Linux: what it
   writes, root may change, and it can change nothing root could not. The
   code of the first slice already did this; the decision it followed said
   otherwise, which is what this one mends.

.. dec:: A tool step's home folder is /tmp
   :id: DEC_STEP_HOME_IN_TMP
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_STEP_HOME_NOT_WRITABLE
   :statement: Agconflo shall set the home folder of every tool step to /tmp.

   A step runs as a user no image's passwd names, so its home is ``/`` and
   read-only, and a tool keeping anything in its home fails; with ``/tmp`` it
   writes there (``EVD_STEP_HOME_NOT_WRITABLE``). An environment of its own is
   of little use to a tool that cannot work in it.

   A home in a granted folder was the alternative, and would write a tool's
   caches and settings into the person's project.

.. dec:: A tool container's /tmp is limited by the grants, and a step that fills it is told so
   :id: DEC_TMP_LIMITED_BY_GRANTS
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_FULL_TMP_LOSES_OUTPUT
   :statement: Agconflo shall limit each tool container's /tmp to the size its grants give, 268435456 bytes unless they say otherwise, and add to the output of a step that ends with /tmp full a line saying so.

   ``/tmp`` is held in memory, and without a limit a command writing without
   end - a runaway build, a log - fills the machine's memory before its time
   runs out. The limit is the person's, beside the grants' others, and 256
   MiB unless they give one: the home folder is there too
   (``DEC_STEP_HOME_IN_TMP``), and caches grow.

   A full ``/tmp`` loses a step's output silently, its own and the next
   one's (``EVD_FULL_TMP_LOSES_OUTPUT``), since the output is written there
   first. The model would read an empty answer as a command that printed
   nothing. The wrapper therefore looks at ``/tmp`` once the command ends and
   says when it is full. Writing the output to a file system of its own was
   the alternative: one more mount, sized apart, to save the output of a
   command that filled ``/tmp`` itself, which the line tells the model to
   stop doing.

.. dec:: An unreachable container engine refuses no run
   :id: DEC_UNREACHABLE_ENGINE_REFUSES_NO_RUN
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall start, resume and answer a run whose container engine cannot be reached, leaving the first tool step it reaches awaiting with the engine's failure, and report the unreachable engine to a check.

   An engine that cannot be reached says nothing about the workflow or its
   grants: it is the failure ``DEC_ENGINE_FAILURE_LEAVES_THE_STEP`` leaves a
   step awaiting for, found before the step rather than during it. Refusing
   the run for it refused, too, the answer that decision offers a person for
   a step the engine could not perform - one of its two ways on, and the only
   one that does not wait for the engine.

   Refusing only a start was the alternative: two rules for one failure, and
   a run refused at its start only to stop, a minute later on a resume, at the
   same step with the same failure. A check still reports it, since a person
   checking a workflow wants to know.

.. dec:: A tool's container is locked down and its own process runs as a user its steps do not
   :id: DEC_CONTAINER_OWN_USER_APART
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_CONTAINER_CONFINES_TO_MOUNTS, EVD_KILL_ALL_AFTER_A_COMMAND, EVD_KILL_ALL_AS_ROOT_BESIDE_65534
   :supersedes: DEC_CONTAINER_LOCKED_DOWN
   :statement: Agconflo shall start a tool's container with an init process, no capabilities, no new privileges and a read-only root, and run its own process as user and group 65534 when its steps run as root and as root otherwise.

   What ``DEC_CONTAINER_LOCKED_DOWN`` decided stands but for the user a step
   runs as. Every step still runs with no capabilities, now as the person's
   own user, which is root for a person who is
   (``DEC_STEP_USER_ROOT_INCLUDED``). Its reason for an unprivileged step was
   the cleanup after each step, which kills every process of the step's user
   (``DEC_COMMAND_WRAPPED``) and must not reach the container's own. What that
   needs is the two users apart, which a root step does not rule out.

   The container's own process runs as root, as it did, beside a step of any
   other user, and as 65534, the user Linux names ``nobody``, beside a root
   step. A root step without capabilities can signal no process of another
   user, and the cleanup then ended everything the step left and nothing of
   the container's, in both images the tests use
   (``EVD_KILL_ALL_AS_ROOT_BESIDE_65534``).

   Running the container's own process as 65534 beside every step was the
   alternative: one rule where this has two, and a collision with a person
   whose own user is 65534. Killing only the step's process group was the
   other, and a process that leaves its group, as ``setsid`` does, would
   outlive the step.

.. dec:: A tool step runs as the person's user on Linux, root included
   :id: DEC_STEP_USER_ROOT_INCLUDED
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_MOUNT_OWNERSHIP_ON_LINUX, EVD_UID_READABLE_FROM_PROC, EVD_ROOT_FOLDER_CLOSED_TO_1000, EVD_ROOT_STEP_WITHOUT_CAPABILITIES
   :supersedes: DEC_STEP_USER_NEVER_ROOT
   :statement: Agconflo shall perform tool steps on Linux as the user and group of the process running the run, and elsewhere as user and group 1000.

   ``DEC_STEP_USER_NEVER_ROOT`` ran a root person's steps as 1000, and on an
   engine installed on Linux that closes every folder they grant writable to
   the step: root made it, with mode 755, and 1000 can make nothing in it
   (``EVD_ROOT_FOLDER_CLOSED_TO_1000``). The grant says writable and the tool
   cannot write. Its body held that 1000 serves a root person as it serves
   anyone off Linux, where a step as 1000 wrote into its mount
   (``EVD_CONTAINER_CONFINES_TO_MOUNTS``); on Linux it does not.

   As root without the capabilities the container drops
   (``DEC_CONTAINER_OWN_USER_APART``), a step writes where root owns and what
   it writes is root's, and it cannot read a file of another user that root
   reads only by those capabilities (``EVD_ROOT_STEP_WITHOUT_CAPABILITIES``):
   a user whose files the person can change and who can change nothing they
   could not, as ``CREQ_SANDBOX_STEP_USER`` asks. The cleanup that kept
   ``DEC_STEP_USER_IS_THE_PERSONS`` from holding for root is kept from the
   container by moving the container's own process, not the step.

   A root step can set the setuid bit on a file it writes, which on the host
   is then a program running as root for whoever may run it
   (``EVD_ROOT_STEP_WITHOUT_CAPABILITIES``). Any person's step can do the same
   as that person. ``STKH_TOOLS_CONFINED`` is met against accident, not
   attack, and allows any change inside a writable grant, so the README tells
   it rather than the sandbox undoing it: clearing the bit would scan every
   writable folder after every step, for a change the goal allows.

   Keeping 1000 and having a root person give their folders to it was the
   alternative. A grant saying writable would work only once the person
   changed its owner, and the tests would change their own folders' owner to
   pass, hiding the gap rather than closing it. Refusing a writable grant to a
   root person was the other: a refusal no requirement makes, leaving tools
   unusable wherever a person works as root, as in a container.
