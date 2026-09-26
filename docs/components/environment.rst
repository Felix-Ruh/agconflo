=======================================
Components of a tool in its environment
=======================================

The requirements ``ARCH_TOOL_ENVIRONMENT`` allocates to the four components it
uses, each defined where its first feature put it: the project reader and the
runner in ``components/runner``, the grants reader and the sandbox in
``components/tools``. Each title is the grammatical subject of the requirements
allocated to it, and the gate in ``scripts/gates`` refuses a component
requirement whose subject is anything else.

Two requirements of ``components/tools`` are revised in the same change, each
by its record in ``decisions/changes``: ``CREQ_SANDBOX_STEP_USER`` and
``CREQ_RUNNER_REFUSES_UNGRANTED``.

The last three are ``ARCH_TOOL_TRUST``'s, allocated to the grants reader and
the sandbox.

.. comp_req:: A manifest gives each tool the image and the container its entry names
   :id: CREQ_PROJECT_READS_TOOL_ENVIRONMENT
   :derived_from: FEAT_TOOL_WORKS_IN_ITS_ENVIRONMENT, FEAT_TOOL_ENVIRONMENT_SHARED_BY_NAME
   :allocated_to: COMP_PROJECT_READER
   :ears_pattern: event
   :statement: When a manifest is read, Project reader shall give for each tool the image its entry names or none and the container its entry names or the one named tools.

   ``DEC_TOOL_NAMES_ITS_ENVIRONMENT``: a tool's entry is its action, as
   before, or a table of ``action``, ``image`` and ``container``, the last two
   optional. A tool naming no container shares the one named ``tools`` with
   every other that names none, so a manifest of the first slice of tools
   means what it meant.

   Failure modes:

   - **An entry written as a string refused**, and every manifest written
     before this feature with it.
   - **A tool naming no container given one of its own**, and two tools that
     named nothing no longer sharing what they did.

.. comp_req:: A tool's environment named wrongly is refused at that tool
   :id: CREQ_PROJECT_REFUSES_ENVIRONMENT_FAULTS
   :derived_from: FEAT_TOOL_ENVIRONMENT_UNGRANTED_REFUSED
   :allocated_to: COMP_PROJECT_READER
   :ears_pattern: unwanted
   :statement: If a manifest gives a tool an image not named by digest or image id or a malformed container name or gives two tools one container but different images, then Project reader shall refuse the manifest at that tool.

   An image by tag could name another image by the next run
   (``DEC_IMAGE_BY_DIGEST_NEVER_PULLED``), and a container has one image
   (``DEC_ONE_CONTAINER_PER_NAME``). A container name is well formed when it
   is lower-case letters, digits, ``_``, ``.`` and ``-``, beginning with a
   letter or a digit. Two tools' images are compared as written: one naming
   the grants' image by its digest and one naming none are different, since
   the project reader does not read the grants.

   Failure modes:

   - **A tagged image accepted here** and refused only when the grants are
     read, or never.
   - **A container name the engine refuses** passed on, and failing as the
     engine's failure at the first step.
   - **Two images in one container accepted**, and the second tool run in
     the first one's image.

.. comp_req:: A grants file gives the images it allows beside its image
   :id: CREQ_GRANTS_READS_IMAGES
   :derived_from: FEAT_TOOL_ENVIRONMENT_UNGRANTED_REFUSED
   :allocated_to: COMP_GRANTS_READER
   :ears_pattern: event
   :statement: When a grants file is read, Grants reader shall give the images it names beside its image, none unless it names some.

   ``DEC_GRANTS_LIST_IMAGES``: ``images``, a list, beside ``image``, which
   stays the image of every tool naming none.

   Failure modes:

   - **The grants' image left out of what a tool may name**, refusing a tool
     that names it by its digest.

.. comp_req:: An image the grants allow by tag is refused
   :id: CREQ_GRANTS_REFUSES_TAGGED_IMAGES
   :derived_from: FEAT_TOOL_ENVIRONMENT_UNGRANTED_REFUSED
   :allocated_to: COMP_GRANTS_READER
   :ears_pattern: unwanted
   :statement: If a grants file names one of its images by anything but a digest or an image id, then Grants reader shall refuse it at that image.

   What ``CREQ_GRANTS_REFUSES_TAGGED_IMAGE`` holds of the image, for each of
   the others (``DEC_IMAGE_BY_DIGEST_NEVER_PULLED``).

   Failure modes:

   - **Only the first of the images checked.**

.. comp_req:: A grants file gives the size of /tmp
   :id: CREQ_GRANTS_READS_TMP_LIMIT
   :derived_from: FEAT_TOOL_KEPT_TO_ITS_GRANT
   :allocated_to: COMP_GRANTS_READER
   :ears_pattern: event
   :statement: When a grants file is read, Grants reader shall give the size its limits name for /tmp or 268435456 bytes when they name none.

   ``DEC_TMP_LIMITED_BY_GRANTS``: ``tmp`` under ``limits``, beside ``seconds``
   and ``output``, a whole number of bytes from 1. The parent keeps a tool from
   changing anything outside its grant, and ``/tmp`` is held in the machine's
   memory: filled without end, it takes that memory from everything else the
   machine runs.

   Failure modes:

   - **An absent size read as none**, and ``/tmp`` left without a limit.

.. comp_req:: A step is performed in the one container of its name, made from its image
   :id: CREQ_SANDBOX_CONTAINER_PER_NAME
   :derived_from: FEAT_TOOL_WORKS_IN_ITS_ENVIRONMENT, FEAT_TOOL_ENVIRONMENT_SHARED_BY_NAME
   :allocated_to: COMP_SANDBOX
   :ears_pattern: event
   :statement: When a step is asked for in a container name and image, Sandbox shall perform it in the one container of that name it has made, making it from that image at the first such step.

   ``DEC_ONE_CONTAINER_PER_NAME``: every container is locked down as
   ``CREQ_SANDBOX_LOCKED_DOWN`` says, labelled with the run's record file and
   with its name, and named ``agconflo-``, eight hexadecimal digits of a hash
   of the record file's path, ``-`` and its name. Everything carrying the
   run's label is removed before the first is made, as
   ``CREQ_SANDBOX_REMOVES_LEFTOVERS`` says, and every container made is
   removed at the end, as ``CREQ_SANDBOX_REMOVED_AT_THE_END`` says of one.

   Failure modes:

   - **Every step in one container** whatever its name: a build's leftovers
     seen by every tool.
   - **A container made at every step**, and nothing shared at all.
   - **A killed run's container of the same name left**, and the new one
     refused for its name (``EVD_CONTAINER_NAME_IN_USE``).

.. comp_req:: A step has a home folder it can write
   :id: CREQ_SANDBOX_STEP_HOME
   :derived_from: FEAT_TOOL_WORKS_IN_ITS_ENVIRONMENT
   :allocated_to: COMP_SANDBOX
   :ears_pattern: event
   :statement: When a step is run, Sandbox shall give it a home folder its user can write.

   ``DEC_STEP_HOME_IN_TMP``: ``HOME`` is ``/tmp``
   (``EVD_STEP_HOME_NOT_WRITABLE``).

   Failure modes:

   - **The image's home left**, ``/`` for a user no passwd names, and
     read-only.

.. comp_req:: A container's /tmp is kept to the size the grants give
   :id: CREQ_SANDBOX_TMP_LIMIT
   :derived_from: FEAT_TOOL_KEPT_TO_ITS_GRANT
   :allocated_to: COMP_SANDBOX
   :ears_pattern: event
   :statement: When a container is made for a run, Sandbox shall limit its /tmp to the size the grants give.

   ``DEC_TMP_LIMITED_BY_GRANTS``: the size of the file system in memory
   ``/tmp`` is.

   Failure modes:

   - **No limit**, and a command writing without end taking the machine's
     memory.

.. comp_req:: A step that ends with /tmp full is told so
   :id: CREQ_SANDBOX_TELLS_FULL_TMP
   :derived_from: FEAT_TOOL_PERFORMED
   :allocated_to: COMP_SANDBOX
   :ears_pattern: unwanted
   :statement: If a step's command ends with the container's /tmp full, then Sandbox shall add to its output a line saying /tmp is full and output may have been lost.

   ``DEC_TMP_LIMITED_BY_GRANTS``: a full ``/tmp`` loses a step's output
   without a word (``EVD_FULL_TMP_LOSES_OUTPUT``), and the parent continues
   the run with the tool's result as the step's output. An empty output from
   a command that printed is not its result, and the model reading it takes
   it for one.

   Failure modes:

   - **Nothing said**, the model reading an empty output as a command that
     printed nothing.
   - **The line said when /tmp is not full**, a command that printed nothing
     reported as having lost its output.

.. comp_req:: Each image asked about is found present with sh and timeout, or reported
   :id: CREQ_SANDBOX_EACH_IMAGE_READY
   :derived_from: FEAT_TOOL_ENVIRONMENT_UNGRANTED_REFUSED
   :allocated_to: COMP_SANDBOX
   :ears_pattern: unwanted
   :statement: If an image the sandbox is asked about is not present or holds no sh or timeout, then Sandbox shall report which it lacks for that image without pulling it.

   What ``CREQ_SANDBOX_IMAGE_READY`` holds of the grants' image, for each image
   a run's tools name (``DEC_IMAGE_BY_DIGEST_NEVER_PULLED``), asked once for
   each image however many tools name it.

   Failure modes:

   - **Only the grants' image asked about**, and a tool's own image absent
     found at its first step.
   - **A report without the image it is about.**

.. comp_req:: A step is performed in its tool's container and image
   :id: CREQ_RUNNER_STEPS_IN_THEIR_ENVIRONMENT
   :derived_from: FEAT_TOOL_WORKS_IN_ITS_ENVIRONMENT, FEAT_TOOL_ENVIRONMENT_SHARED_BY_NAME
   :allocated_to: COMP_RUNNER
   :ears_pattern: event
   :statement: When a tool step is performed, Runner shall have the sandbox perform it in its tool's container and image, the grants' image for a tool naming none.

   ``DEC_TOOL_NAMES_ITS_ENVIRONMENT``. A step a model's call makes is its
   called tool's, so it runs in that tool's container and image, not the
   caller's.

   Failure modes:

   - **Every step in the grants' image**, a tool's own ignored.
   - **A call's step in its caller's environment** rather than its tool's.

.. comp_req:: A tool asking for an image its grants do not allow or the sandbox does not find is refused
   :id: CREQ_RUNNER_REFUSES_UNGRANTED_IMAGE
   :derived_from: FEAT_TOOL_ENVIRONMENT_UNGRANTED_REFUSED
   :allocated_to: COMP_RUNNER
   :ears_pattern: unwanted
   :statement: If a tool the manifest names asks for an image its grants do not name or one the sandbox finds absent or lacking sh or timeout, then Runner shall refuse the run before any node runs naming each tool and why.

   ``DEC_GRANTS_LIST_IMAGES``, found before the record file is taken, as
   ``CREQ_RUNNER_REFUSES_UNGRANTED`` finds the rest. An engine that cannot be
   reached refuses nothing (``DEC_UNREACHABLE_ENGINE_REFUSES_NO_RUN``).

   Failure modes:

   - **An image outside the grants performed in**, which is performing a
     tool with a grant nobody gave.
   - **Only the first such tool named.**

.. comp_req:: A check reports every tool whose image the grants do not allow
   :id: CREQ_RUNNER_CHECKS_TOOL_ENVIRONMENT
   :derived_from: FEAT_RUNNER_CHECKS_WITHOUT_RUNNING
   :allocated_to: COMP_RUNNER
   :ears_pattern: event
   :statement: When a check is asked for of a manifest naming a tool, Runner shall also hand back every tool whose image its grants do not name and every image the sandbox does not find ready.

   What a run would be refused for is what a check reports, and it is refused
   for these.

   Failure modes:

   - **A check reporting only the grants' image**, and a tool's own absent
     image found by starting a run.

.. comp_req:: A grants file gives the certificate authorities it names
   :id: CREQ_GRANTS_READS_TRUST
   :derived_from: FEAT_TOOL_TRUSTS_GRANTED_AUTHORITIES
   :allocated_to: COMP_GRANTS_READER
   :ears_pattern: event
   :statement: When a grants file names a trust file, Grants reader shall give that file's path, read relative to the grants file's directory.

   ``DEC_TRUST_IN_THE_GRANTS``. The path is the file itself, a link resolved,
   so what is mounted is the file the grants name and not whatever a link
   names later; the file is checked with the grants, like a granted folder.

   Failure modes:

   - **The path read from the working directory**, and a run started elsewhere
     trusting another file or none.
   - **A link given as the path**, and the mount following it to another file.

.. comp_req:: A trust file that cannot be read refuses the grants at its key
   :id: CREQ_GRANTS_REFUSES_BAD_TRUST
   :derived_from: FEAT_TOOL_GRANTS_UNREADABLE_REFUSED
   :allocated_to: COMP_GRANTS_READER
   :ears_pattern: unwanted
   :statement: If a grants file names a trust file that is not a readable file, then Grants reader shall refuse it at that key naming the path.

   A grant that cannot be honoured refuses the run before anything runs, as a
   folder that is not there does.

   Failure modes:

   - **A missing file read as no trust**, and a run that needed it failing
     later at the first tool reaching the network, far from the cause.
   - **A directory accepted** as the file.

.. comp_req:: A step runs trusting the granted authorities mounted read-only
   :id: CREQ_SANDBOX_TRUSTS_GRANTED
   :derived_from: FEAT_TOOL_TRUSTS_GRANTED_AUTHORITIES
   :allocated_to: COMP_SANDBOX
   :ears_pattern: event
   :statement: When a step is run under grants that name a trust file, Sandbox shall run it with SSL_CERT_FILE naming that file mounted read-only in its container.

   ``DEC_TRUST_MOUNTED_READ_ONLY``: the file is mounted when the container is
   made, which ``CREQ_SANDBOX_LOCKED_DOWN`` allows since the grants name it.

   Failure modes:

   - **The file mounted writable**, and a step able to change what every later
     step of the container trusts, or the host's file.
   - **The variable set without the mount**, or the mount made in one
     container of a run and not in another.
   - **A step without trust given one**, where the grants name none.
