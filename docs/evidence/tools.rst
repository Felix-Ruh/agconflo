===============================
Evidence about confining a tool
===============================

Measurements taken for ``STKH_TOOLS_CONFINED`` and ``STKH_TOOLS_AS_NODES``
before anything was specified for them: what a container confines a tool to,
what a link a tool makes points at, what passes through a mounted folder, what
outlives the process that started it, and what a command inside a container
costs.

Each was taken on Windows 10 (19045) with Docker Desktop 29.7.2 running Linux
containers on WSL 2, in the image ``alpine:3``, and the Linux half through
WSL's Ubuntu against the same Docker daemon. Unless a body says otherwise the
container ran with ``--network none --read-only --tmpfs /tmp --cap-drop ALL
--security-opt no-new-privileges --user 1000:1000``, with a folder of the
scratch directory mounted as ``/work``.

What was not measured is named here: a build that needs a package registry
with no network, and a local model calling a node a tool performs. Both belong
to the live run of the feature rather than to its design.

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
   :observation: Links made in a writable mount to ../outside and to /etc led, inside the container, to nothing and to the container's own read-only /etc, while on the host they stayed as links that Git Bash lists and PowerShell does not follow.

   Inside the container, ``ln -s ../outside /work/up``, ``ln -s /etc
   /work/to-etc`` and a link through ``/work/../outside`` were made, where
   ``outside`` is a host folder beside the mounted one and not mounted.
   Writing through ``up`` failed as a nonexistent directory, reading through
   the third found no file, and appending to ``to-etc/passwd`` was refused. The
   host's ``outside`` folder was unchanged.

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
   was 777; uid 4242 was not tried at 755. ``ls -ln`` gave each file its writer's uid and gid. A file the host
   user made with mode 600 was refused to uid 4242 with "Permission denied".
