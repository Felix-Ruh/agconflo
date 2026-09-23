====================
Behaviour test cases
====================

How each requirement in ``components/behaviour`` is to be checked, with two
feature-level cases where the claim is about the whole rather than a part.
Results are never written here: they are imported from the test runner, and
each case records only what it asserts.

Every failure mode listed in ``components/behaviour`` is caught by a case, and
each case below names the failure modes it catches, so the derivation can be
audited rather than taken on trust. Most of this feature's requirements say what
a script may not do, and a host that refused every script would satisfy nearly
all of them - so the positive cases, and the feature-level property, are the half
that keeps the rest honest.

A case's id is the path of the Rust test that implements it, uppercased. The
crate's modules are named ``behaviours``, ``host`` and ``scripted``, none of
which is a module of ``agconflo-core``: a case id carries no crate name, so a
module name shared across the two crates would give two tests one id.

Limits in these cases are set small - thousands of instructions and a few
mebibytes - so that a case meeting one does so in milliseconds, well inside the
time a test is allowed.

.. test_case:: A node type with no script is refused before anything runs
   :id: TEST_BEHAVIOURS_MISSING_SCRIPT_IS_REFUSED
   :verifies: CREQ_BEHAVIOURS_REFUSE_MISSING
   :test_kind: error_path
   :coverage: partial

   A workflow with two instances of one node type and one of another, given a
   script for the second type only and a script keyed by a misspelling of the
   first. The run is refused, with no activation performed, and the refusal holds
   exactly one fault: the first type, named once.

   Catches: found when the node is reached (nothing is performed); named by
   instance rather than by type (two instances, one fault); a misspelt key taken
   to fill the gap.

.. test_case:: A node type nothing instantiates needs no script
   :id: TEST_BEHAVIOURS_UNINSTANTIATED_TYPE_NEEDS_NO_SCRIPT
   :verifies: CREQ_BEHAVIOURS_REFUSE_MISSING
   :test_kind: positive
   :coverage: partial

   A catalogue declaring a type no instance names, with no script for it, and a
   script supplied for a type the catalogue does not have at all. The run starts
   and completes.

   Catches: a script asked of every declared type rather than every
   instantiated one. And it is the must-pass shape for a caller sharing one set
   of scripts across workflows.

.. test_case:: A script that does not compile is refused with the compiler's account
   :id: TEST_BEHAVIOURS_UNCOMPILABLE_SCRIPT_IS_REFUSED
   :verifies: CREQ_BEHAVIOURS_REFUSE_UNCOMPILABLE
   :test_kind: error_path
   :coverage: partial

   A script with a syntax error on its second line, supplied from a document with
   a name of its own. The run is refused with no activation performed, and the
   fault names the node type and the document and carries the compiler's
   message, which itself names the document and line 2.

   Catches: compiled when first run; the message dropped; the document not
   named.

.. test_case:: A broken script for a type this run never runs passes
   :id: TEST_BEHAVIOURS_UNINSTANTIATED_BROKEN_SCRIPT_PASSES
   :verifies: CREQ_BEHAVIOURS_REFUSE_UNCOMPILABLE
   :test_kind: positive
   :coverage: partial

   A script that does not compile, supplied for a declared type that no instance
   names. The run starts and completes: a script this run never runs is not this
   run's fault.

.. test_case:: A name that resolves to nothing fails when run, not when checked
   :id: TEST_BEHAVIOURS_UNDEFINED_NAME_FAILS_WHEN_RUN
   :verifies: CREQ_BEHAVIOURS_REFUSE_UNCOMPILABLE
   :test_kind: positive
   :coverage: partial

   A script calling a function that exists nowhere. The run starts, since the
   script compiles (``EVD_LUA_COMPILES_WITHOUT_RUNNING``), and the activation
   fails as a script error naming the missing name.

   The control against a check that claims more than compiling can know: a
   refusal here would mean the check ran the script.

.. test_case:: A node type given two scripts is refused
   :id: TEST_BEHAVIOURS_TWO_SCRIPTS_ARE_REFUSED
   :verifies: CREQ_BEHAVIOURS_REFUSE_TWICE
   :test_kind: error_path
   :coverage: partial

   One node type given two scripts from two documents, first with different
   texts and then with identical ones. Both runs are refused, naming the type.

   Catches: the later kept; the first kept; refused only when the texts differ.

.. test_case:: Every fault in the scripts is reported in one refusal
   :id: TEST_BEHAVIOURS_EVERY_FAULT_REPORTED
   :verifies: CREQ_BEHAVIOURS_EVERY_FAULT
   :test_kind: error_path
   :coverage: partial

   Three instantiated types: one with no script, one whose script does not
   compile, and one given two. One refusal carries all three faults, each as a
   value of its own kind.

   Catches: the first fault only; one kind at a time; faults joined into one
   message.

.. test_case:: Checking the scripts runs none of them
   :id: TEST_BEHAVIOURS_CHECKING_RUNS_NOTHING
   :verifies: CREQ_BEHAVIOURS_NOTHING_RUN
   :test_kind: error_path
   :coverage: partial

   A workflow that is refused because one type has no script, where another
   type's script raises an error on its first line. The refusal holds the
   missing script and nothing else: had the check run the other script, it would
   have met that error and reported it.

   Catches: the script run to see whether it fails; its top level run.

.. test_case:: A script reads its inputs by parameter name
   :id: TEST_HOST_INPUTS_BY_PARAMETER_NAME
   :verifies: CREQ_HOST_RUNS_THE_SCRIPT
   :test_kind: positive
   :coverage: partial

   A node type declaring two parameters in an order that is not alphabetical,
   whose script composes them by name in the other order. The output renders
   them in the order the script named them.

   Catches: inputs handed over by position.

.. test_case:: An unbound optional parameter is absent from what a script is given
   :id: TEST_HOST_UNBOUND_OPTIONAL_IS_ABSENT
   :verifies: CREQ_HOST_RUNS_THE_SCRIPT
   :test_kind: positive
   :coverage: partial

   A node type with an optional parameter the workflow leaves unbound. The
   script finds it ``nil``, and says so in its output.

   Catches: an unbound optional parameter handed over as an empty context.

.. test_case:: Each node type runs its own script
   :id: TEST_HOST_EACH_TYPE_RUNS_ITS_OWN_SCRIPT
   :verifies: CREQ_HOST_RUNS_THE_SCRIPT
   :test_kind: positive
   :coverage: partial

   Three node types with three different scripts in a chain. The result shows
   each script's contribution in the order the chain ran them.

   Catches: the script of another node type run.

.. test_case:: A script is told the type its output is declared as
   :id: TEST_HOST_OUTPUT_TYPE_IS_GIVEN
   :verifies: CREQ_HOST_RUNS_THE_SCRIPT
   :test_kind: positive
   :coverage: partial

   A script that makes its output with the type it is given rather than one
   written into it, run for two node types declaring different output types.
   Both outputs are accepted.

   Catches: the output type not given.

.. test_case:: A script returning anything but one context fails, saying what it returned
   :id: TEST_HOST_NOT_ONE_CONTEXT_FAILS
   :verifies: CREQ_HOST_ONE_CONTEXT
   :test_kind: error_path
   :coverage: partial

   Five scripts, one run each: returning ``nil``, returning nothing, returning
   two contexts, returning a string and returning a table. Each run ends with
   that activation failed, naming what was returned - five different answers -
   and no instance after it performed.

   Catches: the first of several taken; nothing taken as success; a string
   turned into a context; failed without saying what came back.

.. test_case:: An output the run refuses fails the activation with the refusal
   :id: TEST_HOST_REFUSED_OUTPUT_FAILS_WITH_THE_REFUSAL
   :verifies: CREQ_HOST_OUTPUT_REFUSAL_CARRIED
   :test_kind: error_path
   :coverage: partial

   A script returning a context of a type its node type does not declare, and
   one returning the input it was given. Each run ends with the activation failed
   carrying the run's own refusal - of the undeclared type, and of the held
   identifier.

   That each run ends at all catches the script being run again, since nothing
   would change its answer and nothing would stop the loop. Carrying the run's
   value catches the refusal dropped for a failure of the host's own.

.. test_case:: Arguments made from another source collide with the first output
   :id: TEST_SCRIPTED_ARGUMENTS_FROM_ANOTHER_SOURCE_FAIL
   :verifies: CREQ_HOST_OUTPUT_REFUSAL_CARRIED
   :test_kind: error_path
   :coverage: partial

   A run's arguments made from one identifier source and the run given another,
   with two scripts. One makes its output first, so the output carries the
   identifier an argument already has; the other composes its input with a word,
   so the word - a part - carries it. The run refuses each, as a held identifier
   and as a shared one, and the activation fails carrying that refusal.

   Not a failure mode of the requirement but of using it: the run and its
   caller have to share one source, and this pins down what a caller who does
   not is told.

   The composing shape is the one that found the gap. The case was first
   written with it, and that run completed: the run asked only about the
   output's own identifier, so the word under the argument's identifier passed
   and one run held two contexts sharing it. The case was narrowed to the direct
   shape until the run refused a second context under a held identifier
   anywhere in an output (``CREQ_RUN_REFUSES_SHARED_OUTPUT_IDENTIFIER``), and
   now holds both.

.. test_case:: Nothing an activation leaves behind reaches the next
   :id: TEST_HOST_NOTHING_SURVIVES_AN_ACTIVATION
   :verifies: CREQ_HOST_FRESH_STATE
   :test_kind: error_path
   :coverage: partial

   Two instances of one node type in a chain, whose script sets a global and a
   field of the string library and renders both. The second instance renders
   neither.

   Catches: one state for the run; one state per node type, which two instances
   of one type share; globals cleared between activations, which the string
   library's field survives.

.. test_case:: A script can reach nothing outside its activation
   :id: TEST_HOST_NOTHING_READS_OUTSIDE
   :verifies: CREQ_HOST_NOTHING_OUTSIDE
   :test_kind: error_path
   :coverage: partial

   A script listing which of ``io``, ``os``, ``require``, ``package``,
   ``dofile``, ``loadfile``, ``load``, ``loadstring``, ``debug``,
   ``math.random`` and ``math.randomseed`` it can see: none. And a script calling
   ``io.open`` fails as a script error, rather than opening anything.

   Catches: the default state; a compiler left in; the random source left in.

.. test_case:: A script can catch no error, so the limits hold
   :id: TEST_HOST_NOTHING_CATCHES_AN_ERROR
   :verifies: CREQ_HOST_NO_CATCHING
   :test_kind: error_path
   :coverage: partial

   ``pcall``, ``xpcall`` and ``coroutine`` are absent. A script that wraps an
   endless loop in whichever of them it can find fails on the instruction limit,
   and one that wraps an oversized allocation the same way fails on the memory
   limit.

   Catches: ``pcall`` removed and ``xpcall`` left; the coroutine library given;
   relying on the hook's flag, which the memory case has none of.

.. test_case:: A script's error fails the activation with its whole message
   :id: TEST_HOST_ERROR_CARRIES_ITS_MESSAGE
   :verifies: CREQ_HOST_ERROR_CARRIED
   :test_kind: error_path
   :coverage: partial

   A script raising an error on its third line with a message of two lines. The
   activation fails as a script error - not as a limit - and the message carries
   both lines, the document's name and line 3.

   Catches: a script error reported as a limit; the message cut to its first
   line or its location dropped.

.. test_case:: An error that is not a string is still a script error
   :id: TEST_HOST_NON_STRING_ERROR_IS_RAISED
   :verifies: CREQ_HOST_ERROR_CARRIED
   :test_kind: error_path
   :coverage: partial

   A script raising a table, and one raising ``nil``. Both activations fail as
   script errors. The message is not asserted on beyond that, since a table's
   rendering is its address and changes from run to run.

   Catches: a non-string error refused or lost.

.. test_case:: An endless script is stopped at its instruction limit
   :id: TEST_HOST_RUNAWAY_SCRIPT_IS_STOPPED
   :verifies: CREQ_HOST_INSTRUCTION_LIMIT
   :test_kind: error_path
   :coverage: partial

   A script that loops for ever. The activation fails as having exceeded the
   instruction limit - not as a script error - and nothing after it is
   performed.

   Catches: no limit; the limit reported as a script error.

.. test_case:: The instruction limit is each activation's own
   :id: TEST_HOST_INSTRUCTION_LIMIT_IS_PER_ACTIVATION
   :verifies: CREQ_HOST_INSTRUCTION_LIMIT
   :test_kind: error_path
   :coverage: partial

   A chain of three instances whose script spends about two thirds of the
   instruction limit. The run completes, which it could not if the count were
   kept across activations.

   Catches: counted per run rather than per activation.

.. test_case:: A script over its memory limit is stopped
   :id: TEST_HOST_MEMORY_BOMB_IS_STOPPED
   :verifies: CREQ_HOST_MEMORY_LIMIT
   :test_kind: error_path
   :coverage: partial

   A script making one string far larger than the memory limit. The activation
   fails as having exceeded the memory limit - not as a script error and not as
   the instruction limit - and nothing after it is performed.

   Catches: no limit; the limit reported as a script error.

.. test_case:: The memory limit is each activation's own
   :id: TEST_HOST_MEMORY_LIMIT_IS_PER_ACTIVATION
   :verifies: CREQ_HOST_MEMORY_LIMIT
   :test_kind: error_path
   :coverage: partial

   A chain of three instances whose script holds about two thirds of the memory
   limit while it runs. The run completes, which it could not if the limit were
   shared across activations.

   Catches: a limit on the whole run.

.. test_case:: A changed script changes the result, with nothing rebuilt
   :id: TEST_SCRIPTED_CHANGED_SCRIPT_CHANGES_THE_RESULT
   :verifies: FEAT_BEHAVIOUR_FROM_SCRIPT
   :test_kind: positive
   :coverage: partial

   One workflow run twice in one test binary, with two different scripts for
   one of its node types. The two results differ exactly as the two scripts do.

   The parent goal made concrete: behaviour changed between two runs, and the
   engine was compiled once.

.. test_case:: Scripted workflows run to completion
   :id: TEST_SCRIPTED_WORKFLOWS_RUN_TO_COMPLETION
   :verifies: FEAT_BEHAVIOUR_ONE_CONTEXT
   :test_kind: property
   :coverage: partial

   For any chain of instances whose scripts each compose their input with a
   piece of text of their own, a run completes, and its result renders as the
   argument followed by each piece in the order the chain ran.

   The counterweight to a file of refusals: a host that failed every activation
   would satisfy almost every other case here.
