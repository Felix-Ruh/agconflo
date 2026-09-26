====================================
A tool in its environment test cases
====================================

How each requirement in ``components/environment`` and
``features/environment`` is to be checked, and the case that checks the
narrowing of ``CREQ_RUNNER_REFUSES_UNGRANTED``. Results are never written
here: they are imported from the test runner.

A case's id is the path of the Rust test that implements it, uppercased. The
cases live in ``agconflo-runner``'s modules ``grants``, ``project``,
``sandbox`` and ``runner``, and in ``agconflo-cli``'s integration test file
``command``.

A case about what a container does uses Docker, the image the tests pin by its
digest, and ``python:3.14-slim`` pinned by its own, both named in
``evidence/tools``; it fails with a message naming what is missing when any is
(``DEC_TESTS_NEED_DOCKER``). The runner's handling of environments is checked
against the stand-in for the sandbox, which records the container and the
image each step was asked in. No case reaches the network or a provider.

.. test_case:: A grants file gives the images it allows beside its image
   :id: TEST_GRANTS_READS_IMAGES
   :verifies: CREQ_GRANTS_READS_IMAGES
   :test_kind: positive
   :coverage: full

   A grants file naming its image and two images by digest gives both, in its
   order, beside the image; one naming no ``images`` gives none.

   Catches: the images dropped, the grants' own image counted among them.

.. test_case:: An image the grants allow by tag is refused at that image
   :id: TEST_GRANTS_TAGGED_IMAGES_REFUSED
   :verifies: CREQ_GRANTS_REFUSES_TAGGED_IMAGES
   :test_kind: error_path
   :coverage: full

   ``images`` holding a pinned image then ``python:3``, and one holding
   ``python:3`` alone, are each refused at ``python:3``, its line and column;
   one holding only pinned images is read - the control.

   Catches: only the first of the images checked.

.. test_case:: A grants file gives the size of /tmp, 256 MiB unless it says
   :id: TEST_GRANTS_TMP_LIMIT_READ
   :verifies: CREQ_GRANTS_READS_TMP_LIMIT
   :test_kind: positive
   :coverage: full

   ``tmp = 1048576`` under ``limits`` gives 1048576; no ``tmp`` gives
   268435456; ``tmp = 0`` is refused as the other limits' zero is.

   Catches: an absent size read as none.

.. test_case:: A manifest gives each tool its image and container, or the shared one
   :id: TEST_PROJECT_READS_TOOL_ENVIRONMENT
   :verifies: CREQ_PROJECT_READS_TOOL_ENVIRONMENT
   :test_kind: positive
   :coverage: full

   A ``[tools]`` table with one tool as a string, one as a table naming only
   its action, one naming an image, and two naming a container ``build``,
   one with an image. What is read gives the first two no image and the
   container ``tools``, the third its image and ``tools``, and the last two
   ``build`` with their images.

   Catches: an entry written as a string refused, a tool naming no container
   given one of its own.

.. test_case:: A tool's environment named wrongly is refused at that tool
   :id: TEST_PROJECT_ENVIRONMENT_FAULTS_REFUSED
   :verifies: CREQ_PROJECT_REFUSES_ENVIRONMENT_FAULTS
   :test_kind: error_path
   :coverage: full

   Each refused at the tool: an image ``python:3``; containers named
   ``Build``, ``a b``, ``-x``, ``.x``, ``a/b`` and the empty name; two tools
   naming ``build`` with two different images; one naming ``build`` with an
   image and another naming it with none. ``b.u_i-l0`` is read, and two
   tools naming ``build`` with one image are - the controls.

   Catches: a tagged image accepted, a container name the engine refuses
   passed on, two images in one container accepted.

.. test_case:: Each container name has its own container, made from its image and named after the run
   :id: TEST_SANDBOX_CONTAINER_PER_NAME
   :verifies: CREQ_SANDBOX_CONTAINER_PER_NAME
   :test_kind: positive
   :coverage: full

   With Docker: a step in ``a`` of the pinned image writes a file in
   ``/tmp``; a second step in ``a`` reads it; a step in ``b`` of the Python
   image finds no such file and runs ``python3``, which ``a`` has not. Two
   containers carry the run's label, named ``agconflo-``, eight hexadecimal
   digits the same for both, ``-a`` and ``-b``. A container left under the
   name ``-a`` by a killed run, carrying the label, is removed before ``a`` is
   made rather than refusing it.

   Catches: every step in one container, a container made at every step, a
   killed run's container of the same name left.

.. test_case:: The wrapper keeps its statuses, limits and cleanup in the Python image
   :id: TEST_SANDBOX_WRAPPER_IN_ANOTHER_IMAGE
   :verifies: CREQ_SANDBOX_CONTAINER_PER_NAME
   :test_kind: error_path
   :coverage: partial

   With Docker, in the Python image, whose ``sh`` is dash and whose
   ``timeout`` is GNU's: an exit 3 comes back as 3 with both streams; a
   command past a limit of two seconds as 137 with what it printed first; an
   output past 100 bytes cut to both ends naming the bytes cut; and a
   background process and a double fork are gone by the next step.

   Catches: a step made in another image that works in the letter and loses
   its limits.

.. test_case:: A step's home folder is one it can write
   :id: TEST_SANDBOX_STEP_HOME
   :verifies: CREQ_SANDBOX_STEP_HOME
   :test_kind: positive
   :coverage: full

   With Docker, in both images: ``cd ~ && touch here`` succeeds, and a second
   step finds the file there.

   Catches: the image's home left.

.. test_case:: A container's /tmp holds no more than the grants give it
   :id: TEST_SANDBOX_TMP_LIMITED
   :verifies: CREQ_SANDBOX_TMP_LIMIT
   :test_kind: error_path
   :coverage: full

   With Docker, under a ``tmp`` of 1 MiB: writing 2 MB to ``/tmp`` fails,
   and ``df`` gives ``/tmp`` a size of 1 MiB. Under the default, ``df`` gives
   256 MiB - the control.

   Catches: no limit.

.. test_case:: A step ending with /tmp full is told so, and no other
   :id: TEST_SANDBOX_FULL_TMP_TOLD
   :verifies: CREQ_SANDBOX_TELLS_FULL_TMP
   :test_kind: error_path
   :coverage: full

   With Docker, under a ``tmp`` of 1 MiB: a step filling ``/tmp`` and
   echoing a word comes back with its status and the line saying ``/tmp`` is
   full; the next step, while it stays full, too. A step printing nothing
   with ``/tmp`` not full comes back empty, without the line - the control.

   Catches: nothing said, the line said when ``/tmp`` is not full.

.. test_case:: Each image is found ready or reported by its name
   :id: TEST_SANDBOX_EACH_IMAGE_READY
   :verifies: CREQ_SANDBOX_EACH_IMAGE_READY
   :test_kind: error_path
   :coverage: partial

   With Docker: the pinned image and the Python image are each ready; a
   digest present nowhere is reported absent, naming that digest, and is not
   pulled.

   Catches: a report without the image it is about. Only the grants' image
   asked about is the runner's to catch, in
   ``TEST_RUNNER_REFUSES_UNGRANTED_IMAGE``.

.. test_case:: A step is performed in its tool's container and image
   :id: TEST_RUNNER_STEPS_IN_THEIR_ENVIRONMENT
   :verifies: CREQ_RUNNER_STEPS_IN_THEIR_ENVIRONMENT
   :test_kind: positive
   :coverage: full

   Against the stand-in: a workflow of a ``read`` tool naming nothing, a
   ``run`` tool naming the container ``build`` and an image, and a model
   calling a third tool naming the container ``other``. Each step was asked
   in its tool's container and image: ``tools`` and the grants' image, then
   ``build`` and its image, then ``other`` - the called tool's, not its
   caller's.

   Catches: every step in the grants' image, a call's step in its caller's
   environment.

.. test_case:: A tool whose image the grants do not name or the sandbox does not find is refused
   :id: TEST_RUNNER_REFUSES_UNGRANTED_IMAGE
   :verifies: CREQ_RUNNER_REFUSES_UNGRANTED_IMAGE
   :test_kind: error_path
   :coverage: full

   Against the stand-in: two tools naming images the grants do not list are
   refused naming both; a tool naming an image the grants list, which the
   stand-in finds absent, is refused naming that image, and the stand-in was
   asked about each distinct image once. None takes the record file. A tool
   naming the grants' image by its digest starts - the control.

   Catches: an image outside the grants performed in, only the first such
   tool named, only the grants' image asked about.

.. test_case:: An engine out of reach refuses no run, and its first tool step is left awaiting
   :id: TEST_RUNNER_UNREACHABLE_ENGINE_NOT_REFUSED
   :verifies: CREQ_RUNNER_REFUSES_UNGRANTED
   :test_kind: error_path
   :coverage: partial

   Against a stand-in reporting the engine out of reach, and failing every
   step as the engine's failure: a start goes on to its first tool step and
   stops awaiting it with the failure beside it; an answer of that step, with
   the engine still out of reach, is taken, and the run goes on to its next
   tool step and stops there alike. The check of the same project reports
   the engine.

   Catches: a run refused for an engine out of reach, which is what the
   revision in ``DEC_CHANGE_RUNNER_REFUSES_UNGRANTED`` removes; against the
   code before it, both are refused.

.. test_case:: A check reports a tool's image the grants do not name
   :id: TEST_RUNNER_CHECK_REPORTS_TOOL_ENVIRONMENT
   :verifies: CREQ_RUNNER_CHECKS_TOOL_ENVIRONMENT
   :test_kind: error_path
   :coverage: full

   Against the stand-in: a check of a manifest whose tool names an image the
   grants do not list reports that tool; with the stand-in finding that image
   absent once it is listed, reports that image; and was asked for no
   container.

   Catches: a check reporting only the grants' image.

.. test_case:: A tool's steps run in the image the manifest names for it
   :id: TEST_RUNNER_ENVIRONMENTS_END_TO_END
   :verifies: FEAT_TOOL_WORKS_IN_ITS_ENVIRONMENT
   :test_kind: positive
   :coverage: partial

   With Docker: a workflow whose ``run`` tool naming the Python image runs
   ``python3 -c`` printing a sum, and whose ``run`` tool naming nothing runs
   ``command -v python3``. The first step's output holds the sum, the second
   says ``python3`` is not there, and each wrote into ``/tmp`` through
   ``~``.

.. test_case:: Tools naming one environment share what they leave, and no other tool sees it
   :id: TEST_RUNNER_ENVIRONMENT_SHARED_END_TO_END
   :verifies: FEAT_TOOL_ENVIRONMENT_SHARED_BY_NAME
   :test_kind: positive
   :coverage: partial

   With Docker: a workflow whose ``make`` and ``use`` tools name the
   container ``build`` and whose ``peek`` tool names ``other``, all of the
   pinned image. ``make`` writes a file in ``/tmp``; ``use`` reads it; ``peek``
   finds none. Once the run awaits its person, no container carries its
   label.

.. test_case:: A tool's environment the grants do not allow is refused before the run
   :id: TEST_COMMAND_ENVIRONMENT_UNGRANTED_REFUSED
   :verifies: FEAT_TOOL_ENVIRONMENT_UNGRANTED_REFUSED
   :test_kind: error_path
   :coverage: full

   With Docker: a run whose tool names an image the grants do not list exits
   4 naming the tool; one whose grants list an image present nowhere exits 4
   naming that image; one whose two tools name one container with two images
   exits 4 naming the second tool. No record file is left by any. The same
   run with the Python image listed and present completes - the control.
