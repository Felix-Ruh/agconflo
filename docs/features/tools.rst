=======================================
A tool performed within what is granted
=======================================

A node type performed by a tool - reading a file, writing one, running a
command - in a run a person asked for, kept to what that person granted it.
Every requirement here derives from a goal in ``stakeholder/execution``, and
each is written against the decisions in ``decisions/tools``.

What this does not do is what keeps it a slice. A tool is one of three
actions (``DEC_THREE_TOOL_ACTIONS``); nothing here reaches an MCP server, gives
a tool an image of its own, or lets a network in part. The core and the
scripted run are not changed: a tool's step is handed to the runner as a
person's is (``DEC_TOOL_PERFORMED_BY_THE_RUNNER``), and what is added is the
runner performing it.

Much of what a tool step needs is already required of every step, in words
that already cover one, and is not said again here. A tool step counts against
the budget (``FEAT_RUN_BUDGET_STOPS``); a model calls one as it calls any node
type its instance declares (``FEAT_YIELD_CALL_PERFORMED``); a resumed run
performs no step again whose output its record holds
(``FEAT_RESUME_REPEATS_NO_OUTPUT``); and the text a step is answered with
becomes its output (``FEAT_PERSON_TEXT_IS_THE_OUTPUT``). Each gains test cases
in ``tests/tools`` and nothing else. A person is told how a run stopped
(``FEAT_RUNNER_TELLS_HOW_IT_STOPPED``) with no new way of stopping: a step the
engine could not perform is awaited, as a person's is.

Each requirement was checked by hand against the two questions every body here
answers: could it be false while its parents hold, and could they hold while it
is false? Three statements are ``event`` and four ``unwanted``.

The feature's architecture closes the file. It realises all seven requirements
and names the components they are divided between; ``components/tools`` defines
the three new ones and holds the requirements this feature allocates to all
six.

.. feat_req:: A tool's step is performed and the run goes on with its result
   :id: FEAT_TOOL_PERFORMED
   :derived_from: STKH_TOOLS_AS_NODES
   :ears_pattern: event
   :verification_method: test
   :statement: When a run a person asked for reaches a step of a node type its manifest names as a tool, Agconflo shall perform that tool and continue the run with the tool's result as the step's output.

   The parent lets a model reach a tool through a node that wraps it, and this
   is that node, hosted where a person runs a workflow: the tool is performed
   as the step, and what it gives back is a context with a place in the run,
   which is the parent's own reason for wanting it that way.

   It can be false while the parent holds. A host that recognises a tool's
   node type and refuses to run it, or performs it and ends the run, lets a
   model reach tools only through nodes in the letter of the parent - by
   letting it reach none.

   It names no mechanism and no set of tools: which actions there are, what a
   result holds and how a step that never returns is ended are decisions
   (``DEC_THREE_TOOL_ACTIONS``, ``DEC_COMMAND_WRAPPED``). A step a model calls
   is a step of the run (``DEC_CALL_IS_AN_ACTIVATION``), so it is covered.

.. feat_req:: A tool changes nothing outside its grant
   :id: FEAT_TOOL_KEPT_TO_ITS_GRANT
   :derived_from: STKH_TOOLS_CONFINED
   :ears_pattern: event
   :verification_method: test
   :statement: When a tool's step is performed, Agconflo shall keep the tool from changing anything outside what the run's grants allow it.

   The parent's sentence at the level where it is tested: a step performed,
   and what lies outside the grant unchanged afterwards. What that is, for a
   test, is every file outside the folders the grants mark writable and every
   address beyond the machine when no network is granted.

   It cannot be false while the parent holds: it is the parent restated
   where a test can hold a run to it, naming the step and the grant the
   parent's sentence leaves general. It claims no more. Inside a grant a tool
   may change anything, as the parent allows, and code built to break out of
   what confines it is outside the parent's reach, which is accident, not
   attack.

.. feat_req:: What a run's tools may do is the person's to grant when they run it
   :id: FEAT_TOOL_GRANTED_BY_THE_PERSON
   :derived_from: STKH_TOOLS_CONFINED, STKH_RUN_FROM_DOCUMENTS
   :ears_pattern: event
   :verification_method: test
   :statement: When a person asks for a run whose manifest names a tool, Agconflo shall take what the run's tools may do from grants that person gives with the request, apart from the workflow's documents.

   ``STKH_TOOLS_CONFINED`` keeps a tool to what the person running the
   workflow granted it, and ``STKH_RUN_FROM_DOCUMENTS`` says that person is
   the one asking for the run from its documents. A grant written into those
   documents is the author's, which is the person running it only when they
   are the same.

   It can be false while both hold. A host reading grants from the manifest
   confines every tool to a grant, runs every workflow from its documents, and
   confines a workflow someone else wrote to what its author granted it.

.. feat_req:: Grants that cannot be read refuse the run
   :id: FEAT_TOOL_GRANTS_UNREADABLE_REFUSED
   :derived_from: STKH_WIRING_CHECKED, STKH_TOOLS_CONFINED
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a run whose manifest names a tool is asked for without grants or with grants that cannot be read, then Agconflo shall refuse the run before any node in it runs and say where the fault is.

   ``STKH_WIRING_CHECKED`` refuses what cannot run before anything runs, and a
   tool with no grant it can be read from has nothing it may do.
   ``STKH_TOOLS_CONFINED`` rules out the other way out of the same fault:
   performing it with some grant nobody gave.

   It can be false while both hold. A host that finds grants missing at the
   first tool step, and fails the step there, confines every tool and refuses
   an invalid workflow before any node runs - while a run that has spent every
   step before the tool learns only then that it could never finish.

.. feat_req:: A tool its grants do not allow is refused before the run starts
   :id: FEAT_TOOL_UNGRANTED_REFUSED
   :derived_from: STKH_WIRING_CHECKED, STKH_TOOLS_CONFINED
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a tool a run's manifest names lacks the grant or the parameter its action needs or the image its grants name is absent, then Agconflo shall refuse the run before any node in it runs and say which tool and why.

   Each is a run that cannot finish as written: a tool whose action is not
   granted cannot be performed within its grant, one lacking the parameter its
   action takes cannot be performed at all, and nothing is performed without
   its image. ``STKH_WIRING_CHECKED`` refuses such a run before anything runs,
   and ``STKH_TOOLS_CONFINED`` refuses the other answer to a missing grant,
   which is performing the tool anyway.

   It can be false while both hold, for the reason the requirement above can:
   found at the step, the fault is refused only after every step before it has
   been paid for.

.. feat_req:: An interrupted tool step is performed again with nothing of the first still running
   :id: FEAT_TOOL_INTERRUPTED_PERFORMED_AGAIN
   :derived_from: STKH_RESUMABLE_RUN
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a run is interrupted while a tool's step is being performed, then Agconflo shall perform that step again when the run is resumed with nothing of the interrupted performance still running.

   The parent resumes an interrupted run, and a run interrupted mid-step is
   resumed by performing the step again: its output was never recorded.
   What the parent needs of that is a step performed once more, as it would
   have been, and not twice at once.

   It can be false while the parent holds. A host that resumes the run and
   performs the step again, while the command the interrupted process started
   is still writing into the same folder (``EVD_EXEC_OUTLIVES_ITS_CLIENT``),
   resumes the run in the letter of the parent with two performances of one
   step racing, and a result neither would have given alone.

.. feat_req:: A step the container engine could not perform is left for the person
   :id: FEAT_TOOL_ENGINE_FAILURE_LEAVES_THE_STEP
   :derived_from: STKH_RESUMABLE_RUN
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If the container engine fails while a tool's step is performed, then Agconflo shall stop the run awaiting that step, resumable from its record, and tell the person why.

   An engine that stopped or a container removed from outside interrupts the
   run the way a process that ended does, and the parent resumes a run that
   was interrupted. The step says nothing about the workflow, so the run is
   left where it stood (``DEC_ENGINE_FAILURE_LEAVES_THE_STEP``), and a person
   told why can resume it once the engine is back, or answer the step.

   It can be false while the parent holds. A host that ends the run as failed
   on an engine's failure has a run that was not interrupted but ended, which
   the parent does not ask to resume - and a run lost over a failure of the
   machine rather than of the workflow.

.. feat_arch:: A tool splits into a grants reader, a sandbox and a tool performer beside the runner
   :id: ARCH_TOOLS
   :realises: FEAT_TOOL_PERFORMED, FEAT_TOOL_KEPT_TO_ITS_GRANT, FEAT_TOOL_GRANTED_BY_THE_PERSON, FEAT_TOOL_GRANTS_UNREADABLE_REFUSED, FEAT_TOOL_UNGRANTED_REFUSED, FEAT_TOOL_INTERRUPTED_PERFORMED_AGAIN, FEAT_TOOL_ENGINE_FAILURE_LEAVES_THE_STEP
   :uses: COMP_GRANTS_READER, COMP_SANDBOX, COMP_TOOL_PERFORMER, COMP_PROJECT_READER, COMP_RUNNER, COMP_COMMAND_LINE
   :statement: Agconflo shall allocate performing a tool within its grant to the grants reader, the sandbox, the tool performer, the project reader, the runner and the command line.

   Three new components in ``agconflo-runner``, and three of running from
   documents given one more thing each to do, each answerable for one
   question:

   - The grants reader answers what the person granted: the grants file read
     into the image, folders, network, actions and limits, or refused where
     the fault is.
   - The sandbox answers what a container does: made, locked down, running a
     step's command within its limits, handing back its output and status or
     the engine's own failure, and removed, with what a killed run left
     removed before.
   - The tool performer answers what a tool's step comes to: the step's
     inputs read by the action's parameters, a path refused or passed on, the
     sandbox asked, and the result made into the text the step is answered
     with.
   - The project reader reads the manifest's tools, beside its scripts and
     persons.
   - The runner answers the steps the tool performer performs, stops at those
     the engine could not, and checks a run's tools before it starts.
   - The command line reads the grants file's path and tells the person why a
     step was left awaiting.

   The core and ``agconflo-lua`` are not among them
   (``DEC_TOOL_PERFORMED_BY_THE_RUNNER``). The sandbox is the one component
   that knows a container engine exists; the tool performer knows the three
   actions and nothing of Docker, which is what lets the runner's handling of
   tools be tested against a stand-in for it (``DEC_TESTS_NEED_DOCKER``).

   The decisions this is built against are named here rather than linked:

   - ``DEC_GRANTS_IN_A_FILE_OF_THEIR_OWN``, ``DEC_GRANTS_NARROW_BY_DEFAULT``,
     ``DEC_IMAGE_BY_DIGEST_NEVER_PULLED``: the grants reader.
   - ``DEC_ONE_CONTAINER_PER_CALL``, ``DEC_LEFTOVER_CONTAINERS_REMOVED``,
     ``DEC_CONTAINER_LOCKED_DOWN``, ``DEC_STEP_USER_IS_THE_PERSONS``,
     ``DEC_COMMAND_WRAPPED``, ``DEC_OUTPUT_KEEPS_BOTH_ENDS``,
     ``DEC_ENGINE_THROUGH_ITS_COMMAND``: the sandbox.
   - ``DEC_THREE_TOOL_ACTIONS``, ``DEC_PATHS_AS_ARGUMENTS``,
     ``DEC_PATHS_IN_GRANTED_FOLDERS``, ``DEC_TOOL_FAILURE_IS_OUTPUT``: the tool
     performer.
   - ``DEC_TOOLS_NAMED_IN_THE_MANIFEST``: the project reader.
   - ``DEC_TOOL_PERFORMED_BY_THE_RUNNER``,
     ``DEC_ENGINE_FAILURE_LEAVES_THE_STEP``: the runner.
