===============================
Evidence about confining a tool
===============================

Measurements taken for ``STKH_TOOLS_CONFINED`` and ``STKH_TOOLS_AS_NODES``
before anything was specified for them: what a container confines a tool to,
what a link a tool makes points at, what passes through a mounted folder, what
outlives the process that started it, what a command inside a container
costs, and what answering a run's steps from outside the run costs as the run
grows.

Each was taken on Windows 10 (19045) with Docker Desktop 29.7.2 running Linux
containers on WSL 2, in the image ``alpine:3``, and the Linux half through
WSL's Ubuntu against the same Docker daemon. The two measurements of
answering a run's steps involved no container. Unless a body says otherwise
the container ran with ``--network none --read-only --tmpfs /tmp --cap-drop ALL
--security-opt no-new-privileges --user 1000:1000``, with a folder of the
scratch directory mounted as ``/work``.

What was not measured is named here: a build that needs a package registry
with no network, and a Docker engine installed on Linux itself and used
without root - rootless Docker or Podman among them. The last seven below
were taken on an engine installed on Linux and used by root, in a cloud
development container whose outgoing TLS is intercepted, and each says so. A
local model calling a node a tool performs was left to the live run of the
feature, which ``EVD_TOOLS_LIVE_RUN`` records; its project needed no package.

.. evd:: A container reaches only the folders mounted into it
   :id: EVD_CONTAINER_CONFINES_TO_MOUNTS
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: In a locked-down container, rm -rf on a read-only mount was refused and rm -rf / reached nothing unmounted, nor did the network, while every file in the one writable mount was deleted on the host.

   One folder was mounted read-only as ``/project`` and one writable as
   ``/work``, each holding one file. ``rm -rf /project/*`` failed with
   "Read-only file system" and the file was unchanged on the host. ``rm -rf /``
   exited 1; the host folders were untouched but the writable one, whose file
   was gone. ``wget`` to a public address failed. A file written to
   ``/work`` appeared on the host, owned by the Windows user.

   A folder on the ``F:`` drive mounted read-only the same way was listed, and
   ``touch`` in it failed with "Read-only file system".

.. evd:: A link a tool makes resolves inside the container
   :id: EVD_LINKS_RESOLVE_IN_THE_CONTAINER
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: Links made in a writable mount to ../outside and to /etc led inside the container to nothing and to its own /etc, where writing passwd was refused; on the host they stayed links Git Bash lists and PowerShell does not follow.

   Inside the container, ``ln -s ../outside /work/up``, ``ln -s /etc
   /work/to-etc`` and a link through ``/work/../outside`` were made, where
   ``outside`` is a host folder beside the mounted one and not mounted.
   Writing through ``up`` failed as a nonexistent directory, reading through
   the third found no file, and appending to ``to-etc/passwd`` was refused
   with "Permission denied" - the refusal of the user the container ran as,
   so what the read-only root would have said was not seen. The host's
   ``outside`` folder was unchanged.

   On the host the three are reparse points. Git Bash lists them as links to
   ``../outside``, ``/etc`` and the third path; PowerShell reports no link type
   for them and failed to read ``up\host.txt`` through the first. A program on
   the host that follows such a link - on Linux, any - reaches what it names
   relative to the host, which is outside the grant.

.. evd:: Text passes through a mounted folder byte for byte
   :id: EVD_LINE_ENDINGS_THROUGH_A_MOUNT
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: A file written on the host with a CRLF and an LF was read in the container as the same five bytes, and one written in the container with an LF and a CRLF was read on the host as the same five bytes.

   ``od -c`` on each side: ``a \r \n b \n`` in, ``c \n d \r \n`` out.

.. evd:: A container outlives the client that started it
   :id: EVD_CONTAINER_OUTLIVES_ITS_CLIENT
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: A container started by an attached docker run was still running after its client was killed, and docker ps found it by the label it was started with.

   ``docker run --label agconflo.run=probe alpine:3 sleep 300`` was started in
   the background and its client killed three seconds later. ``docker
   inspect`` reported the container ``running``, and ``docker ps --filter
   label=agconflo.run=probe`` listed it with the other container carrying the
   label. ``docker rm -f`` on both left none running.

.. evd:: A command in a container outlives the exec that started it
   :id: EVD_EXEC_OUTLIVES_ITS_CLIENT
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: A command started by docker exec kept running after its client was killed, as did one whose reader stopped reading, and one run under timeout -s KILL inside the container was ended after 2.18 s with status 137.

   ``docker exec c sleep 400`` had its client killed after two seconds; ``ps``
   in the container still listed ``sleep 400``. ``docker exec c sh -c yes``
   read through ``head -c 1000`` returned in 217 ms, and ``yes`` was still
   listed afterwards. ``docker exec c timeout -s KILL 2 sh -c 'sleep 30'``
   returned 137 after 2177 ms, and no ``sleep 30`` was left. Removing the
   container ended everything still running in it.

   The command there was a single ``sleep``. One that starts others leaves
   them running, which ``EVD_TIMEOUT_SPARES_GRANDCHILDREN`` measured.

.. evd:: A command in a running container costs about 160 ms
   :id: EVD_EXEC_COST
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: docker exec of a command that does nothing took 161 ms on average over 20, and returning 1, 10 and 50 MB of output took 204, 805 and 4935 ms.

   Measured from Git Bash on the host, the container already running. The
   output was ``yes`` cut to size by ``head -c`` inside the container and
   counted on the host.

.. evd:: A file written in a container on Linux belongs to the user it ran as
   :id: EVD_MOUNT_OWNERSHIP_ON_LINUX
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: Through WSL's Ubuntu, whose user is uid 1000, files written into a mounted folder as uid 1000 and as uid 4242 were owned by 1000 and 4242 on the host, and uid 4242 could not read a host file of mode 600.

   The folder was ``/tmp/agp`` in the Ubuntu distribution, mounted with ``-v``.
   As uid 1000 a file could be made in it at mode 755, and as uid 4242 once it
   was 777; uid 4242 was not tried at 755. ``ls -ln`` gave each file its
   writer's uid and gid. A file the host user made with mode 600 was refused
   to uid 4242 with "Permission denied".

   The daemon was Docker Desktop's, reached from the distribution. An engine
   installed on Linux itself was not measured, with root or without.

.. evd:: Answering a run's steps from outside it costs the same as the run grows
   :id: EVD_ANSWER_COST_FLAT
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: A chain of 150 steps a person performs, each answered by its own agconflo answer process, took 50 ms for the first answer and 61 ms for the last, while the record grew from 419 to 25078 bytes.

   A debug build of the command line, a workflow of one scripted entry and 150
   instances of one node type named in the manifest's ``persons``, each bound
   to the one before. Each answer was under 60 characters. The 10th, 50th
   and 100th answers took 53, 52 and 58 ms: process start and the files read
   dominate, and replaying the record through the run is lost in them.

.. evd:: Answering a model's calls from outside the run grows with the calls made
   :id: EVD_CALL_ANSWER_COST_LINEAR
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: A model making 40 calls to a node type a person performs, each answered with about 2000 bytes by its own agconflo answer process, sent one request per answer counted, took 59 ms to 106 ms, and left a record of 112042 bytes.

   The model was a loopback stub asking for a call until its request held 40
   call results, then answering; no provider key was in the environment. The
   record after the 1st, 10th, 20th, 30th and 40th answer held 3552, 28455,
   56345, 84235 and 112042 bytes, about 2.8 kB per call - the windows recorded
   for earlier calls are not copied into later ones. The request the stub
   received grew by about 2.2 kB per call, to 89579 bytes. The calls already
   answered were answered from the record: at each of the six answers whose
   requests were counted - the 1st, 10th, 20th, 30th, 39th and 40th - the stub
   received exactly one.

.. evd:: Text written through docker exec arrives byte for byte
   :id: EVD_WRITE_THROUGH_EXEC_EXACT
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: Text holding a CRLF, UTF-8, quotes, a dollar sign and backticks, piped into sh -c 'cat > "$1"' through docker exec -i with a path holding spaces as the argument, was byte-identical on the host and read back byte-identical by cat.

   Compared with ``cmp`` on the host against the file piped in. The path was
   passed as the positional argument ``$1`` of ``sh -c`` and never became
   part of the command's text.

.. evd:: A timeout inside the container ends only the command it started
   :id: EVD_TIMEOUT_SPARES_GRANDCHILDREN
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: Under busybox's timeout -s KILL 2, a shell running sleep 30 and then echo, and one that also started sleep 40 in the background, returned 137 with what they printed first, and left both sleeps running.

   ``timeout -s KILL 2 sh -c 'echo started; sleep 30; echo never'`` returned
   137 with ``started`` and ``Killed`` on its output; ``ps`` still listed
   ``sleep 30``. With ``sleep 40 &`` before it, both sleeps were listed. The
   output was written to a file in ``/tmp`` and read back, so what a command
   printed before it was ended was kept.

.. evd:: Killing every process after a command leaves only the container's own
   :id: EVD_KILL_ALL_AFTER_A_COMMAND
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: With --init and the container's own process run as root, kill -KILL -1 as the command's user after each command ended all it had left, double-forked included, left no zombie, and could not signal the container's own processes.

   The container ran ``--init`` with ``sleep 300`` as root, everything else as
   in the header, and ``--tmpfs /tmp:mode=1777``. Each command was run by
   ``docker exec -u 1000:1000``. After a sleep left past its time, a
   background sleep, and one double-forked while ignoring TERM and HUP,
   ``ps`` listed only ``docker-init`` and ``sleep 300``. ``kill -KILL 1`` and a
   kill of the ``sleep 300`` as uid 1000 failed with "Operation not
   permitted", and the container kept running.

   Without ``--init``, the same cleanup in a container whose own process was
   ``sleep`` as uid 1000 left every killed process as a zombie, since
   ``sleep`` collects no child: ``ps`` listed ``[timeout]`` and ``[sleep]``
   entries accumulating with each command.

.. evd:: A command's status and docker's own failure are told apart by where each is written
   :id: EVD_STATUS_APART_FROM_DOCKER_FAILURE
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: docker exec exited 1 for an unreachable daemon and a missing container, as for a command exiting 1, while a wrapper writing the command's output to stdout and only its status to stderr gave 3, 0, 137 and 127 alone there.

   The daemon was made unreachable by pointing ``DOCKER_HOST`` at a closed
   port, which failed in 145 ms with "error during connect" on stderr. The
   container was removed before the exec, which failed with "No such
   container". The wrapper ran the command under ``timeout``, its output and
   errors both to a file, then wrote the file to stdout and the status to
   stderr: a command exiting 3 with a line on each stream, one printing past
   its cap, one past its time and one not found gave ``3``, ``0``, ``137`` and
   ``127``. A command closing its own stderr left the status written.

.. evd:: An image absent locally is found absent before anything runs in it
   :id: EVD_PULL_NEVER_REFUSES_ABSENT
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: docker run --pull never with an image digest not present exited 125 in 139 ms, docker image inspect exited 1 for it and 0 for one present, and alpine:3 held timeout and sh.

   The absent image was ``alpine@sha256:`` followed by 64 zeros; the present
   one ``alpine:3`` by the digest ``docker image inspect`` reported for it. In
   that image ``command -v`` found ``/usr/bin/timeout``, a link to
   ``/bin/busybox``, and ``/bin/sh``.

   The digest of zeros exists in no registry either, so this shows that the
   run failed at once, not that it would have refused an image a registry
   holds.

.. evd:: A process's user and group on Linux are readable from its status file
   :id: EVD_UID_READABLE_FROM_PROC
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: In WSL's Ubuntu, the Uid and Gid lines of /proc/self/status gave 1000 four times each, matching id -u and id -g.

   Kernel ``6.18.33.2-microsoft-standard-WSL2``. The four fields are the real,
   effective, saved and file-system ids.

.. evd:: Starting and removing a locked-down container costs about half a second
   :id: EVD_CONTAINER_START_COST
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: Over five containers started with --init and locked down as a tool's is, docker run -d took 297 to 322 ms, the first exec in each 177 to 182 ms, and docker rm -f 186 to 221 ms, while docker image inspect took 147 ms and 137 ms.

   Each container ran ``sleep 60`` from ``alpine:3`` by its digest with
   ``--pull never``, and its first command was ``true`` as uid 1000. The two
   ``image inspect`` times are for that digest, present, and for a digest of
   zeros, absent.

.. evd:: A container removed during a command looks to docker exec like a command that killed its wrapper
   :id: EVD_REMOVED_CONTAINER_LOOKS_KILLED
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: docker exec exited 137 with nothing on stderr when its container was removed during a sleep and when the command killed every process of its user; docker inspect then failed for the first and gave the second as running.

   The first was ``docker exec -u 1000:1000 c sh -c 'sleep 5'``, with ``docker
   rm -f c`` run 1.5 s in; ``docker inspect`` of the container then exited 1
   with "no such object". The second was the sandbox's own wrapper running
   ``kill -KILL -1; kill -KILL 1`` as uid 1000: the wrapper runs as the step's
   user, so the command's ``kill`` ended it too, and ``docker inspect`` gave
   the container ``running``. Its own process, root's, was out of the
   command's reach, as ``EVD_KILL_ALL_AFTER_A_COMMAND`` found.

   A third, ``rm -rf /`` run through the same wrapper, removed the file in
   ``/tmp`` the wrapper writes the output to. The wrapper then wrote "can't
   open /tmp/out" to standard error before the status, and ``docker exec``
   exited 0.

.. evd:: A local model fixes a failing project through its tools, kept to its grant
   :id: EVD_TOOLS_LIVE_RUN
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: Against qwen3.8-27b-ridge on LM Studio, agconflo ran a model node calling read, write and run tools on a failing shell project in a granted git worktree to completion in 39 s: 6 calls, its 3 tests then passing on the host.

   Taken with the ``agconflo`` binary at ``4a5bc46``, built for debugging,
   from a scratch directory: a manifest naming ``read_file``, ``write_file``
   and ``run_command`` as tools, each a node type with a description; a
   workflow of an entry instance giving a task and one instance of a node
   type whose script asks the model once, declaring twelve calls of each
   tool; a model mapping sending its role to ``openai::qwen3.8-27b-ridge`` at
   LM Studio's endpoint, the one model it had loaded; and grants of the
   pinned ``alpine`` image, all three actions, a limit of 30 seconds and 8000
   bytes, no network, and one folder, ``project``, writable, at a git worktree
   of a scratch repository. Every ``*_API_KEY`` variable was removed from the
   process's environment. ``check`` found nothing first.

   The project was ``slug.sh``, which only turned spaces into hyphens, and
   ``test.sh``, of which two of three cases failed. The task, in English,
   named the folder, the command running the tests, and busybox's ``tr``,
   ``sed`` and ``awk``. The model read ``project/slug.sh`` and
   ``project/test.sh`` in one turn, ran the tests, wrote a ``slug.sh`` whose
   ``sed`` expression was broken, wrote it again mended before running
   anything, ran the tests, which passed, and answered with a paragraph
   saying what was wrong and what it changed. The run exited 0 having spent 8
   of a budget of 60.

   Afterwards, on the host, the tests passed; ``git status`` in the worktree
   named ``slug.sh`` alone, whose diff was the one line the model wrote; the
   repository the worktree came from was unchanged; and no container carried
   the run's label. One run of one model on one small task: it shows the
   feature working end to end, and says nothing of how often such a model
   finds its way, or of a task needing the network.

.. evd:: A step's user has no home it can write unless it is given one
   :id: EVD_STEP_HOME_NOT_WRITABLE
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: Run as uid 1000, which neither image's passwd names, a step in alpine and in python:3.14-slim had HOME set to / and could not write there; given HOME=/tmp, the shell's home and Python's were /tmp and writable.

   Each container was locked down as a tool's is, read-only root and a
   writable ``/tmp`` included. ``cd ~ && touch here`` failed with
   "Read-only file system" in both images. With ``docker exec -e HOME=/tmp``
   the same command succeeded, and in the Python image ``Path.home()`` gave
   ``/tmp`` and a ``.cache`` folder could be made under it. No build tool was
   run: which of them write to the home directory was not measured here.

.. evd:: A full /tmp loses a step's output, and the next step's
   :id: EVD_FULL_TMP_LOSES_OUTPUT
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: With /tmp limited to 1 MiB, a command filling it and then printing came back through the wrapper with status 1 and no output at all, and the next command's output was lost too while /tmp stayed full, with status 0 and no word of it.

   The container was alpine's, locked down, with ``--tmpfs
   /tmp:mode=1777,size=1m``; the wrapper was the sandbox's own, which writes a
   step's output to a file in ``/tmp`` before cutting it. The first command
   wrote 2 MB to ``/tmp/big`` and then echoed a word; the second echoed a
   word and removed the file; the third, with ``/tmp`` free again, echoed the
   word and it came back. Neither the command's own error nor the wrapper's
   reached standard error: only the status did.

.. evd:: A container name in use refuses a new container until the old one is removed
   :id: EVD_CONTAINER_NAME_IN_USE
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: docker run --name exited 125 with a conflict naming the container already holding the name, and the same run succeeded once the containers carrying the first one's label had been removed.

   The first container was left running under the name and a label; the
   second run gave "Conflict. The container name ... is already in use by
   container ..." on standard error. ``docker rm -f`` of every container the
   label filter listed freed the name.

.. evd:: The wrapper behaves the same under dash and GNU coreutils
   :id: EVD_WRAPPER_UNDER_DASH
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: In python:3.14-slim, whose sh is dash and whose timeout is GNU coreutils 9.7, the wrapper gave 3 with both streams, 137 after 2.28 s with what was printed first, 992 bytes named cut from 1092, and left only the container's own processes.

   The container was locked down as a tool's is, with ``--init`` and ``sleep
   infinity`` as root; each command ran through the sandbox's own wrapper as
   uid 1000 under a limit of 2 seconds and 100 bytes. The commands were an
   exit 3 writing a line to each stream, ``echo started; sleep 30; echo
   never``, ``seq 1 300``, and a background sleep beside a double fork
   ignoring TERM and HUP; afterwards ``/proc`` listed only ``docker-init``,
   ``sleep infinity`` and the listing, all root's. A command running ``kill
   -KILL -1`` ended its own wrapper and ``docker exec`` exited 137 with
   nothing on standard error, as in alpine
   (``EVD_REMOVED_CONTAINER_LOOKS_KILLED``).

.. evd:: A local model fixes a failing Python project through a tool of its own image
   :id: EVD_TOOL_ENVIRONMENT_LIVE_RUN
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: Against qwen3.8-27b-ridge on LM Studio, a run whose run_tests tool named python:3.14-slim and a container of its own fixed a failing Python project in a granted worktree in 11 s, over 5 calls, its 3 tests then passing.

   Taken with the ``agconflo`` binary at ``9eb2b37``, built for debugging,
   from a scratch directory: the workflow of ``EVD_TOOLS_LIVE_RUN``, with
   ``read_file`` and ``write_file`` as before and a ``run_tests`` tool
   naming the Python image by its digest and the container ``py``; grants of
   the pinned ``alpine`` as their image, the Python image among their
   images, all three actions, 60 seconds and 8000 bytes, no network, and
   ``project`` writable at a git worktree. Every ``*_API_KEY`` variable was
   removed from the process's environment. ``check`` found nothing; with the
   Python image left out of the grants it named ``run_tests`` and exited 4.

   The project was ``slug.py``, whose ``slug`` only turned spaces into
   hyphens, and ``test_slug.py``, of whose three ``unittest`` cases two
   failed. The model read both files in one turn, ran the tests through
   ``run_tests`` - ``python3`` answered, which the alpine image has not -
   wrote ``"-".join(title.lower().split())``, ran them again, and answered
   with a paragraph saying what was wrong. It spent 7 of a budget of 60.

   Afterwards, on the host, the tests passed; ``git status`` in the worktree
   named ``slug.py`` changed, with the one line the model wrote, and two
   files of ``__pycache__`` Python wrote beside it, inside the grant; the
   repository the worktree came from was unchanged; and no container carried
   the run's label. One run, one small task: it shows two environments
   serving one run, and says nothing of how often a model finds its way.

.. evd:: A step as uid 1000 cannot write in a folder root made, on an engine used by root on Linux
   :id: EVD_ROOT_FOLDER_CLOSED_TO_1000
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: On a Docker engine installed on Linux and used by root, a step run as uid 1000 in a container locked down as a tool's is could make neither a file nor a folder in a writable mount of a folder root had made with mode 755.

   Docker Engine 29.3.1 on Linux 6.18.44, with cgroup v1 and the overlayfs
   snapshotter, in a cloud development container whose every process ran as
   root; the image was the ``alpine`` the tests pin. ``touch w/new`` and
   ``mkdir -p w/d`` each failed with "Permission denied", exit 1. The tests
   make their folders as the test's process, with mode 755, and the nine of
   them that write into a folder granted writable failed the same way there.

.. evd:: A step as root without capabilities reaches what root owns and no more
   :id: EVD_ROOT_STEP_WITHOUT_CAPABILITIES
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: On the same engine, a step run as uid 0 with no capabilities wrote into a mount of a root-owned folder as root, but could not read a mode 600 file of uid 4242, chown, or write the image, and could set the setuid bit on a file it wrote.

   The container was locked down as a tool's is, its own process run as uid
   65534 (``EVD_KILL_ALL_AS_ROOT_BESIDE_65534``), and the step ran through
   ``docker exec -u 0:0``, where ``CapEff`` read ``0000000000000000``. A file
   and a folder it made were ``0:0`` on the host. ``cat`` of the other user's
   file gave "Permission denied", ``chown 4242`` of its own file "Operation not
   permitted", ``touch /etc/x`` "Read-only file system", and ``rm -rf`` in a
   read-only mount the same. ``cp /bin/busybox w/bb && chmod u+s w/bb``
   exited 0, and on the host the file was ``-rwsr-xr-x``, root's. Measured in
   ``alpine`` alone.

.. evd:: A root step's cleanup leaves a container whose own process runs as uid 65534
   :id: EVD_KILL_ALL_AS_ROOT_BESIDE_65534
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: With --init and the container's own process run as uid 65534, kill -KILL -1 as a root step with no capabilities ended all it had left, double-forked included, left no zombie, and could not signal the container's own processes.

   The same engine, and a container locked down as a tool's is but for
   ``--user 65534:65534``. After ``sleep 40 &`` and a double fork of ``sleep
   50`` ignoring TERM and HUP, the cleanup left ``ps`` listing only
   ``docker-init`` and ``sleep infinity``, both ``nobody``'s, and the
   listing; ``kill -KILL 1`` and a kill of the ``sleep infinity`` failed with
   "Operation not permitted", and the container kept running. In the pinned
   ``python:3.14-slim`` a background sleep was ended the same way and
   ``/proc`` listed only the container's own processes.

   With the container's own process left as root, the same cleanup ended it:
   26 tests on that engine then failed at a step after it as the engine's
   failure, ``docker exec`` exiting 137 or the container no longer running.

.. evd:: A container's outgoing TLS is intercepted where the host's is, and trusted only with the host's authority
   :id: EVD_CONTAINER_TLS_INTERCEPTED
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: From a bridge-network container, github.com was reached but refused as a self-signed chain, the host's proxy refused the bridge, and on the host network Python 3.14 refused the authority's certificate for its key usage.

   The engine of ``EVD_ROOT_FOLDER_CLOSED_TO_1000``, where the host's own
   HTTPS goes through a proxy whose certificate authority the host trusts
   from a bundle. Each probe was ``urllib.request.urlopen`` in the pinned
   ``python:3.14-slim``. On the bridge network, direct: "self-signed
   certificate in certificate chain", so the connection left the container
   and was intercepted. On the bridge network with the proxy named at the
   bridge's gateway: "Connection refused", the proxy listening on the host's
   loopback only. On the host network with the proxy and the bundle: "CA cert
   does not include key usage extension", Python's own strictness about that
   authority rather than a route refused.

.. evd:: ubc checks a project in a locked-down container once the container trusts the host's authority
   :id: EVD_UBC_IN_A_CONTAINER
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: In a locked-down container on the bridge network, ubc 0.35.0 checked a clone's 73 documents and answered a Cypher query when the host's CA bundle was at the system path or named by SSL_CERT_FILE, and without it refused as not open source.

   The image was the pinned ``python:3.14-slim`` with ``ubc`` copied in, built
   locally and named by its image id; the container ran with a read-only
   root, no capabilities, no new privileges and ``HOME=/tmp``, with a full
   clone of this repository mounted. With the bundle mounted over
   ``/etc/ssl/certs/ca-certificates.crt``, or mounted elsewhere and named by
   ``SSL_CERT_FILE``, ``ubc check --deny warning`` found no errors and
   ``query cypher`` counted 27 stakeholder requirements; with neither, the
   check exited 1 with "Not determined to be an open source project", which
   is ubc's answer to a licence determination it could not complete.

.. evd:: ubc does not find its licence in a git worktree mounted into a container
   :id: EVD_UBC_REFUSES_A_WORKTREE
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: With the CA trusted, ubc refused a mounted git worktree as not open source, also with the worktree's git directory mounted read-only at its own path, where the clone it came from passed.

   The worktree's ``.git`` is a file naming a directory outside the mounted
   folder. Mounting that directory at the same absolute path, read-only,
   changed nothing; why ubc still found no licence was not investigated.
   The README advises granting a worktree, so a tool that runs ubc has to be
   granted a clone instead.

.. evd:: A single file bind-mounted read-only into a locked-down container stays unchanged and is trusted by ubc
   :id: EVD_TRUST_FILE_MOUNTS_READ_ONLY
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: A CA bundle bind-mounted read-only at /etc/agconflo/trust.pem into a read-only-root container could be neither removed nor written by a root or a 1000 step, and ubc trusted it through SSL_CERT_FILE.

   The engine of ``EVD_UBC_IN_A_CONTAINER``, and a container made as the
   sandbox makes one: ``--init``, the bridge network, a read-only root, a
   ``/tmp`` in memory, no capabilities, no new privileges and its own process
   as 65534, with a full clone mounted writable at ``/work/repo``. The host's
   bundle, copied to a path holding a comma and a space, was mounted with
   ``readonly`` at a target outside ``/work`` that the image does not hold;
   the engine made the mount point on the read-only root itself.

   As root and as 1000, ``rm`` answered "Read-only file system"; appending
   answered the same for root and "Permission denied" for 1000; the host file
   compared equal afterwards. ``docker inspect`` listed the two mounts, the
   file's as not writable.

   With the licence cache in the clone's ``docs/.ub_cache`` removed first,
   ``ubc check --deny warning`` without ``SSL_CERT_FILE`` exited with "Not
   determined to be an open source project", and then with it set found no
   errors, and ``query cypher`` counted 27 stakeholder requirements. Two
   confounds were met on the way and removed: a clone whose ``origin`` named
   the local path rather than the GitHub repository was refused whatever was
   trusted, and a run after a passing one passed without the file, answered
   from that cache.
