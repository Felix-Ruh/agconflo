====================================
Components of running node behaviour
====================================

The two parts ``ARCH_BEHAVIOUR`` divides running node behaviour into, and the
requirements allocated to each. Each title below is the grammatical subject of
the requirements allocated to it, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

Both live in ``agconflo-lua``, which depends on ``agconflo-core`` and adds
nothing to it (``DEC_BEHAVIOUR_OWN_CRATE``).

.. comp:: Behaviour set
   :id: COMP_BEHAVIOUR_SET
   :crate: agconflo-lua

   The scripts a run is started with, one per node type, each with the name of
   the document it came from. It answers, before the run starts, whether every
   node type the workflow instantiates has exactly one script and whether each
   compiles - and it answers without running any of them.

   That is a question about the scripts and the definition together, and it has
   no history in it, which is what separates it from the host that runs them.

.. comp:: Script host
   :id: COMP_SCRIPT_HOST
   :crate: agconflo-lua

   Performs one activation: creates a state for it, gives the script what it may
   reach and nothing more, runs it under its limits, and turns what it returns -
   or the way it failed - into what the run is told.

   Everything it does is true of one activation taken alone. That is what a fresh
   state for each makes possible (``DEC_STATE_PER_ACTIVATION``), and what makes
   each requirement below one a single activation can be tested against.

.. comp_req:: A node type with no script stops the run starting
   :id: CREQ_BEHAVIOURS_REFUSE_MISSING
   :derived_from: FEAT_BEHAVIOUR_REFUSED_BEFORE_START
   :allocated_to: COMP_BEHAVIOUR_SET
   :ears_pattern: unwanted
   :statement: If a node type named by an instance of a workflow has no script and is not named as performed by a person, then Behaviour set shall refuse to start the run naming that node type.

   Only the node types an instance names. A workflow's definition carries every
   declaration in its catalogue, and a type nothing instantiates is never run, so
   asking for its script refuses a run for nothing.

   Failure modes:

   - **Found when the node is reached.** Every activation before it has been
     spent, which is the cost the parent refuses.
   - **Asked of every declared type.** A catalogue shared between workflows then
     forces a script for every type in it on each of them.
   - **Named by instance rather than by node type.** Five instances of one type
     are one missing script, and a refusal listing five sends its reader looking
     for five.
   - **A script supplied for a name no type has, taken to fill the gap.** A
     script keyed by a misspelt type fills nothing, and the real type is still
     without one.

   Must pass unreported: a script for every instantiated type and none for a
   type no instance names, and scripts supplied for types the workflow does not
   have at all, which a caller sharing one set of scripts across workflows will
   do. And a type the caller names as performed by a person, which has no script
   because none runs for it (``DEC_PERSON_NAMED_BY_CALLER``).

.. comp_req:: A script that does not compile stops the run starting
   :id: CREQ_BEHAVIOURS_REFUSE_UNCOMPILABLE
   :derived_from: FEAT_BEHAVIOUR_REFUSED_BEFORE_START
   :allocated_to: COMP_BEHAVIOUR_SET
   :ears_pattern: unwanted
   :statement: If the script supplied for a node type does not compile, then Behaviour set shall refuse to start the run naming that node type, its document and the compiler's message.

   Compiling is the whole of what can be checked without running anything
   (``EVD_LUA_COMPILES_WITHOUT_RUNNING``), and the compiler's message already
   carries the document's name and the line, because the document's name is
   what the chunk is compiled as.

   Failure modes:

   - **Compiled when first run.** The run starts and fails at the node, having
     spent what came before it.
   - **The message dropped.** "Does not compile" without the compiler's account
     sends the author to read the whole script for one character.
   - **The document not named.** A caller that reads scripts from files needs to
     know which file, and two node types may share a script's text but not its
     document.

   Only the scripts of node types an instance names are compiled. A broken
   script for a type this run never runs is not this run's fault - the same
   choice as for a missing one - so it must pass unreported, and a check that
   compiled every script supplied would refuse a run for nothing.

.. comp_req:: A node type given two scripts stops the run starting
   :id: CREQ_BEHAVIOURS_REFUSE_TWICE
   :derived_from: FEAT_BEHAVIOUR_FROM_SCRIPT
   :allocated_to: COMP_BEHAVIOUR_SET
   :ears_pattern: unwanted
   :statement: If more than one script is supplied for a node type, then Behaviour set shall refuse to start the run naming that node type.

   The parent says an activation runs the script supplied for its node type,
   which presumes there is one. Two is the same fault as a run argument supplied
   twice, and gets the same answer for the same reason: choosing either would be
   a choice nobody made.

   Failure modes:

   - **The later one kept.** Which script runs is then decided by the order the
     caller happened to add them in, and a caller that loaded a directory twice
     never finds out.
   - **The first one kept.** The same, the other way round.
   - **Refused only when the texts differ.** Two identical scripts from two
     documents are still two documents claiming one type, and the next edit to
     one of them makes the texts differ without anyone noticing which ran.

.. comp_req:: Every fault in the scripts is reported at once
   :id: CREQ_BEHAVIOURS_EVERY_FAULT
   :derived_from: FEAT_BEHAVIOUR_REFUSED_BEFORE_START
   :allocated_to: COMP_BEHAVIOUR_SET
   :ears_pattern: ubiquitous
   :statement: Behaviour set shall carry every fault of the scripts supplied for a run in its refusal rather than the first.

   The same obligation the validator has for wiring defects, and for the reason
   ``STKH_MACHINE_AUTHORING`` gives: an author learning faults one refusal at a
   time pays a round trip per fault, and an agent correcting its own scripts pays
   it in calls.

   Failure modes:

   - **The first fault only.** Three missing scripts become three refusals.
   - **One kind at a time.** Missing scripts reported, then on the next attempt
     the one that does not compile.
   - **Faults joined into one message.** A caller then parses prose to find out
     which type to fix.

.. comp_req:: Checking a script runs none of it
   :id: CREQ_BEHAVIOURS_NOTHING_RUN
   :derived_from: FEAT_BEHAVIOUR_REFUSED_BEFORE_START
   :allocated_to: COMP_BEHAVIOUR_SET
   :ears_pattern: ubiquitous
   :statement: Behaviour set shall check the scripts supplied for a run without running any part of them.

   The check happens before the run starts, and a script is a node's behaviour:
   running any of it would be running a node before the run that was supposed to
   decide whether one may.

   Failure modes:

   - **The script run to see whether it fails.** Whatever it does is done, once
     for the check and again for the activation, and a script that would fail on
     its inputs fails on none instead.
   - **Its top level run to collect something from it.** The shape a script that
     returns a function would need (``DEC_SCRIPT_IS_THE_BODY``), and why that
     shape was not chosen.

.. comp_req:: An activation is performed by running its node type's script
   :id: CREQ_HOST_RUNS_THE_SCRIPT
   :derived_from: FEAT_BEHAVIOUR_FROM_SCRIPT
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: ubiquitous
   :statement: Script host shall perform an activation of a node type given a script by running that script given the activation's inputs by parameter name, its declared output type and the host functions.

   What a script receives is the activation and the context API
   (``DEC_HOST_FUNCTIONS_CONTEXT_API``): its inputs, each under the name of the
   parameter it fills; the type its output is declared as; and functions to make
   a context from text and to compose one from others. A context it is given can
   be rendered, asked its type, and asked its parts.

   Failure modes:

   - **Inputs by position rather than by name.** A script then depends on the
     order its node type declares its parameters, and reordering a declaration
     silently swaps what each name means.
   - **An optional parameter left unbound handed over as an empty context.** The
     activation leaves it absent (``CREQ_SCHEDULER_ACTIVATION_CARRIES``), and a
     script must be able to tell absent from empty the same way.
   - **The script of another node type run.** A workflow whose node types have
     different scripts finds it; one where every type's script happens to agree
     passes against a host that runs any of them.
   - **The output type not given.** A script then writes its type out by hand,
     and a node type whose declaration changes is a script that fails at the
     run's refusal instead of following it.

.. comp_req:: A script that returns anything but one context fails
   :id: CREQ_HOST_ONE_CONTEXT
   :derived_from: FEAT_BEHAVIOUR_ONE_CONTEXT
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a script returns anything other than exactly one context, then Script host shall fail that activation naming what was returned.

   Failure modes, each one a script's author would otherwise not hear about:

   - **The first of several values taken.** The rest are discarded without a
     word.
   - **Nothing returned, taken as success.** The run is told nothing, and the
     activation neither produced nor failed.
   - **A string turned into a context.** Of which type? Whatever the host
     picked, which is not a decision the script made.
   - **Failed without saying what came back.** ``nil``, a table and two contexts
     are three different mistakes, and "not a context" sends the author to guess
     which.

.. comp_req:: An output the run refuses fails the activation
   :id: CREQ_HOST_OUTPUT_REFUSAL_CARRIED
   :derived_from: FEAT_BEHAVIOUR_ONE_CONTEXT
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If the run refuses the context a script returned, then Script host shall fail that activation carrying the run's refusal.

   The run refuses an output of an undeclared type and one it already holds, and
   leaves the activation outstanding for its caller to answer again or fail
   (``DEC_REFUSED_OUTPUT_OUTSTANDING``). The host is that caller, and answering
   again means running the same script on the same inputs.

   Failure modes:

   - **The script run again.** Nothing has changed that would make it answer
     differently, and a refusal is not counted against the budget, so nothing
     ends the loop.
   - **The refusal dropped for a failure of the host's own.** The run said
     precisely what was wrong, as a value, and a failure that says less has
     thrown that away.

.. comp_req:: Each activation runs in a state of its own
   :id: CREQ_HOST_FRESH_STATE
   :derived_from: FEAT_BEHAVIOUR_READS_ONLY_ITS_INPUTS
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: ubiquitous
   :statement: Script host shall run each activation's script in a Lua state that no other activation has used.

   Measured necessary (``EVD_LUA_SHARED_STATE_LEAKS``): a shared state carried a
   global and a write to the string library from one node's script to the next.

   Failure modes:

   - **One state for the run.** The measured leak.
   - **One state per node type.** Two instances of one type then share one, and
     the second reads what the first left behind.
   - **Globals cleared between activations.** The libraries are shared tables
     and survive a clearing of globals, as the measured write to ``string`` does.

.. comp_req:: A script is given nothing that reads outside its activation
   :id: CREQ_HOST_NOTHING_OUTSIDE
   :derived_from: FEAT_BEHAVIOUR_READS_ONLY_ITS_INPUTS
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: ubiquitous
   :statement: Script host shall give a script no function that reads a file, the process environment, the clock or a random source.

   Built by naming what goes in rather than by removing from a default
   (``DEC_ENVIRONMENT_BY_NAME``), so that what nobody thought of is out by
   construction.

   Failure modes:

   - **The default state.** Measured giving files, the process and a compiler
     (``EVD_LUA_DEFAULT_STATE_EXPOSES``).
   - **A compiler left in.** ``load`` builds a function from text a script
     assembled, and whatever else was removed comes back through it.
   - **The random source left in.** ``math`` is otherwise harmless and brings
     ``math.random`` with it, which differs between processes
     (``EVD_LUA_RANDOM_PER_PROCESS``).

   Not a claim that a script's output is determined by its inputs, for the
   reason ``EVD_LUA_ORDER_PER_PROCESS`` records.

.. comp_req:: A script is given nothing that catches an error
   :id: CREQ_HOST_NO_CATCHING
   :derived_from: FEAT_BEHAVIOUR_INSTRUCTION_LIMIT, FEAT_BEHAVIOUR_MEMORY_LIMIT
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: ubiquitous
   :statement: Script host shall give a script no function that catches an error.

   Both limits are Lua errors by the time a script meets them, and ``pcall`` was
   measured letting a script survive each (``EVD_LUA_PCALL_SWALLOWS_LIMITS``). So
   this is what makes either limit a limit.

   Failure modes:

   - **``pcall`` removed and ``xpcall`` left.** The measurement caught the memory
     error with ``xpcall`` too.
   - **The coroutine library given.** Resuming a coroutine returns its error as a
     value, which is catching it under another name.
   - **Relying on the instruction hook's flag instead.** It shows the
     instruction limit was hit, after the script has caught it and carried on,
     and nothing comparable shows the memory limit.

.. comp_req:: A script's error fails the activation with its message
   :id: CREQ_HOST_ERROR_CARRIED
   :derived_from: FEAT_BEHAVIOUR_ERROR_CARRIED
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a script raises an error, then Script host shall fail that activation carrying the error's message.

   Failure modes:

   - **A limit reported as a script error.** Both limits arrive as Lua errors,
     and a host that looks no further tells the author their script is wrong
     when it ran out of room.
   - **A script error reported as a limit.** The mirror, and as misleading.
   - **The message cut to its first line, or its location dropped.** The line
     number is the part the author needs.
   - **A non-string error refused or lost.** A table raised as an error has no
     message worth keeping, and is still a script error rather than nothing.

.. comp_req:: A script over its instruction limit fails
   :id: CREQ_HOST_INSTRUCTION_LIMIT
   :derived_from: FEAT_BEHAVIOUR_INSTRUCTION_LIMIT
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a script executes more instructions than its limit, then Script host shall fail that activation as having exceeded its instruction limit.

   Counted by a hook every thousand instructions (``EVD_LUA_LIMITS_STOP``), so a
   script is stopped within a thousand instructions of its limit rather than on
   it.

   Failure modes:

   - **No limit.** An endless script holds the run for ever, and nothing reports
     it.
   - **Counted per run rather than per activation.** One busy node exhausts a
     limit the rest of the run was relying on.
   - **Reported as a script error.** See ``CREQ_HOST_ERROR_CARRIED``.

.. comp_req:: A script over its memory limit fails
   :id: CREQ_HOST_MEMORY_LIMIT
   :derived_from: FEAT_BEHAVIOUR_MEMORY_LIMIT
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a script allocates more memory than its limit, then Script host shall fail that activation as having exceeded its memory limit.

   A single allocation over the limit is refused as it is made, which is the
   shape measured (``EVD_LUA_LIMITS_STOP``): one ``string.rep`` of a gigabyte
   against a limit of eight mebibytes.

   Failure modes:

   - **No limit.** The process runs out, and everything with it.
   - **A limit on the whole run.** Activations share nothing else
     (``DEC_STATE_PER_ACTIVATION``), and a limit shared across them would make
     one node's failure depend on how much another allocated.
   - **Reported as a script error.** See ``CREQ_HOST_ERROR_CARRIED``.
