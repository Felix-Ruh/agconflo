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
with no network, a local model calling a node a tool performs, and a Docker
engine installed on Linux itself, with or without root - rootless Docker or
Podman among them. The first two belong to the live run of the feature rather
than to its design.

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
