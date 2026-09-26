============================================
A tool in the environment its workflow names
============================================

A tool performed in the environment the workflow's manifest names for it,
from what the person's grants allow, beside the other tools naming the same
one and apart from the rest. Every requirement here derives from
``STKH_TOOL_ENVIRONMENT``, some from others beside it, and each is written
against the decisions in ``decisions/tools``.

What a tool may touch is not decided here: it is ``features/tools``', and its
requirements hold for a tool in any environment unchanged. Nor is how an
environment is made - an image, a container shared by name, a home folder -
which those decisions say. What is added is where a tool's step is performed,
what it shares, and the refusal of an environment the person has not granted.

Each requirement was checked by hand against the two questions every body here
answers: could it be false while its parents hold, and could they hold while it
is false? Three statements are ``event`` and one ``unwanted``. The last, that a
tool step trusts the authorities the person grants, has an architecture of its
own, ``ARCH_TOOL_TRUST``, since it came after the first three were allocated.

.. feat_req:: A tool's step is performed in the environment its manifest names for it
   :id: FEAT_TOOL_WORKS_IN_ITS_ENVIRONMENT
   :derived_from: STKH_TOOL_ENVIRONMENT
   :ears_pattern: event
   :verification_method: test
   :statement: When a tool's step is performed, Agconflo shall perform it in the environment the manifest names for that tool, or the grants' own when it names none, with a home folder the step can write.

   The parent at the level of a step: the environment a tool runs in is the
   one its workflow named, not the one every other tool of the run was given.
   A tool naming none keeps what the first slice of tools gave every tool,
   the environment the person granted.

   It can be false while the parent holds. A host that reads the environment
   a manifest names, and performs the step in it with no home it can write,
   runs the tool where the workflow said while every tool keeping a cache or
   its settings in its home fails there - named in the letter of the parent,
   of no use in its reason, which is that tools need their environment to
   work.

   It claims no more than the parent: which environments exist and what a
   home is are decisions (``DEC_TOOL_NAMES_ITS_ENVIRONMENT``,
   ``DEC_STEP_HOME_IN_TMP``).

.. feat_req:: Tools naming one environment share what they leave in it within a call
   :id: FEAT_TOOL_ENVIRONMENT_SHARED_BY_NAME
   :derived_from: STKH_TOOL_ENVIRONMENT
   :ears_pattern: event
   :verification_method: test
   :statement: When steps of tools naming the same environment are performed within one start or resume or answer, Agconflo shall let each see what the others left outside the granted folders, and keep it from every tool naming another environment.

   The parent's second reason: a build and the tests of what it built share
   what the build left, and a third tool sees none of it, even in the same
   image. What lies inside a granted folder every tool of the run sees, by
   its grant; what is shared by name is everything else a step leaves behind
   it in its environment.

   It holds within one start, resume or answer, and says so. An environment
   kept between them would be one nothing could tell was still in use
   (``DEC_ONE_CONTAINER_PER_NAME``), and what must outlast the call belongs in
   a granted folder, where it lasts. The parent asks that what one tool left
   be seen by the next, not that it be kept for days.

   It can be false while the parent holds. A host giving every tool its own
   environment, or all of them one, names environments by the workflow in the
   letter of the parent, and either shares nothing a build left or shares it
   with everything.

.. feat_req:: A tool's environment its grants do not allow is refused before the run starts
   :id: FEAT_TOOL_ENVIRONMENT_UNGRANTED_REFUSED
   :derived_from: STKH_TOOL_ENVIRONMENT, STKH_TOOLS_CONFINED, STKH_WIRING_CHECKED
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a tool a run's manifest names asks for an environment its grants do not allow or that is absent or that another tool asks for differently, then Agconflo shall refuse the run before any node in it runs and say which tool and why.

   ``STKH_TOOL_ENVIRONMENT`` has the workflow ask and the person grant, and
   ``STKH_TOOLS_CONFINED`` refuses the other answer to an environment not
   granted, which is performing the tool in it anyway. An environment absent
   from the machine, or asked for two ways by two tools, is a run that cannot
   finish as written, which ``STKH_WIRING_CHECKED`` refuses before anything
   runs.

   It can be false while all three hold, for the reason
   ``FEAT_TOOL_UNGRANTED_REFUSED`` can: found at the tool's first step, the
   fault is refused only after every step before it was paid for.

.. feat_arch:: A tool's environment is read by the project and grants readers, made by the sandbox and chosen by the runner
   :id: ARCH_TOOL_ENVIRONMENT
   :realises: FEAT_TOOL_WORKS_IN_ITS_ENVIRONMENT, FEAT_TOOL_ENVIRONMENT_SHARED_BY_NAME, FEAT_TOOL_ENVIRONMENT_UNGRANTED_REFUSED
   :uses: COMP_PROJECT_READER, COMP_GRANTS_READER, COMP_SANDBOX, COMP_RUNNER
   :statement: Agconflo shall allocate performing a tool in its environment to the project reader, the grants reader, the sandbox and the runner.

   Four components of performing a tool, each given one more thing to do:

   - The project reader reads the image and the container each tool's entry
     names, and refuses an entry naming them wrongly.
   - The grants reader reads the images the grants allow beside their image,
     and the size ``/tmp`` is given.
   - The sandbox makes one container for each name a call's tools use, each
     from its tool's image, gives every step a home folder it can write, keeps
     ``/tmp`` to its size and says when a step filled it, and finds each image
     ready.
   - The runner performs each step in its tool's container and image, and
     refuses a tool whose image the grants do not allow or the sandbox does not
     find.

   The tool performer is not among them: what a step's action comes to does
   not depend on where it is performed. Neither is the command line, whose
   grants file carries the images as it carries the rest.

   The decisions this is built against are named here rather than linked:

   - ``DEC_TOOL_NAMES_ITS_ENVIRONMENT``: the project reader.
   - ``DEC_GRANTS_LIST_IMAGES``, ``DEC_TMP_LIMITED_BY_GRANTS``: the grants
     reader.
   - ``DEC_ONE_CONTAINER_PER_NAME``, ``DEC_STEP_HOME_IN_TMP``,
     ``DEC_TMP_LIMITED_BY_GRANTS``, ``DEC_STEP_USER_NEVER_ROOT``: the sandbox.
   - ``DEC_GRANTS_LIST_IMAGES``, ``DEC_UNREACHABLE_ENGINE_REFUSES_NO_RUN``: the
     runner.

.. feat_req:: A tool step trusts the certificate authorities the person grants
   :id: FEAT_TOOL_TRUSTS_GRANTED_AUTHORITIES
   :derived_from: STKH_TOOL_ENVIRONMENT
   :ears_pattern: event
   :verification_method: test
   :statement: When a person's grants name certificate authorities, Agconflo shall have every tool step of the run trust them.

   The parent has the environment a tool runs in come from what the person
   grants, and on a machine whose outgoing TLS is intercepted an environment
   that does not trust the machine's authority is one where no tool reaching
   the network can work (``EVD_CONTAINER_TLS_INTERCEPTED``).

   It can be false while the parent holds. A host that performs every tool in
   the environment its workflow names, granted a network it cannot use, meets
   the parent in its letter and leaves every tool that fetches anything
   failing - ``ubc`` among them, which is the use a self-hosting workflow puts
   a tool to first.

   It claims no more than the parent: which authorities is the person's, and
   a person naming none is given none.

.. feat_arch:: Trusting granted authorities splits between the grants reader and the sandbox
   :id: ARCH_TOOL_TRUST
   :realises: FEAT_TOOL_TRUSTS_GRANTED_AUTHORITIES
   :uses: COMP_GRANTS_READER, COMP_SANDBOX
   :statement: Agconflo shall allocate trusting the granted certificate authorities to the grants reader and the sandbox.

   The grants reader answers what was granted: the file of authorities, read
   when the grants are, or the grants refused where it names one that cannot
   be read. The sandbox answers what a container does with it: the file
   mounted read-only in each container, and every step run trusting it
   (``DEC_TRUST_IN_THE_GRANTS``, ``DEC_TRUST_MOUNTED_READ_ONLY``).
