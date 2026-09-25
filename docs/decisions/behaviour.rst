======================================
Decisions about running node behaviour
======================================

How what a node does is written, where it runs, and what it can reach while it
runs. ``DEC_RUN_IS_DRIVEN`` left all of it open on purpose - the run hands its
caller one activation at a time and takes back a context or a failure - so these
are the choices about that caller, and none of them reopens the run.

Five rest on measurements recorded in ``evidence/behaviour``, taken against the
newest release of every library involved on the day the decisions were made.
Three are judgements, and say so: where the code lives, where a script comes
from, and what a script is given to call. None of them is a measurement dressed
as one, though the last cites one for a property it relies on.

What they do not settle is named here so it is not inferred. Nothing below
decides how a script reaches a provider, whether an activation is ever
asynchronous, or what further functions a script may one day be given - each
addition to what a script can call is a change to what a node can see, and is
decided when it is wanted.

.. dec:: Node behaviour is a Lua 5.5 script
   :id: DEC_BEHAVIOUR_IN_LUA
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_LUAU_SANDBOX_EXPOSES, EVD_LUAU_BUILD_COST, EVD_LUA_FRESH_STATE_COST
   :statement: Agconflo shall run node behaviour as scripts in Lua 5.5.

   ``STKH_LIVE_BEHAVIOUR`` asks for behaviour that changes without recompiling
   the engine, and a script loaded when a run starts is the shape that needs no
   toolchain at all. Lua embeds through ``mlua``, which builds it from source, and
   a person or a model can write it.

   Luau was the strongest alternative, because it has a sandbox mode, and it lost
   on three measurements. Its sandbox still gave a script ``os``, ``require``,
   ``loadstring`` and ``debug`` (``EVD_LUAU_SANDBOX_EXPOSES``), so the
   environment would have to be built by hand anyway. What the mode does add,
   read-only libraries, a fresh state per activation gives Lua without it. And it
   costs more on both counts measured: a state three and a half times as long to
   create (``EVD_LUA_FRESH_STATE_COST``), a build twice as long
   (``EVD_LUAU_BUILD_COST``).

   Lua 5.4 behaved identically in every probe taken. 5.5 is the newest release,
   and nothing measured argues against it.

   WebAssembly and native plugins were set aside rather than measured. Both need
   a compiler to change a node, which is the cost the stakeholder goal exists to
   remove, and a plugin loaded into the process has whatever the process has.

.. dec:: Node behaviour runs outside agconflo-core
   :id: DEC_BEHAVIOUR_OWN_CRATE
   :dec_status: accepted
   :decided_on: 2026-09-23
   :statement: Agconflo shall run node behaviour in a crate of its own rather than in agconflo-core.

   A judgement, and the argument is the one ``DEC_RUN_IS_DRIVEN`` already made:
   the core decides what may run and what it is given, and performing it is the
   caller's. A crate that runs scripts is such a caller. Putting it in the core
   would bring a C build and a script language into the crate every other caller
   depends on, for callers that perform activations some other way.

   A behaviour trait in the core with Lua as one implementation was the
   alternative. It abstracts over a single backend, and what a script is given
   to call is the context API the core already has
   (``DEC_HOST_FUNCTIONS_CONTEXT_API``), so a second backend would bind the same
   functions without a trait standing between them.

.. dec:: Every activation gets a state of its own
   :id: DEC_STATE_PER_ACTIVATION
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_LUA_SHARED_STATE_LEAKS, EVD_LUA_FRESH_STATE_COST
   :statement: Agconflo shall run each activation's script in a Lua state created for that activation alone.

   One state shared across a run was measured carrying a global and a write to
   the string library from one node's script into the next's
   (``EVD_LUA_SHARED_STATE_LEAKS``). The second node read something no wire
   shows, which is exactly what ``STKH_EXPLICIT_CONTEXT`` rules out.

   Clearing globals between activations was the alternative, and does not close
   it: the libraries are shared tables, and the string library is every string's
   metatable. A read-only environment in the manner of Luau's sandbox would, and
   costs a design for something a new state gives for nothing.

   The cost is 25 microseconds a state on Lua 5.5 (``EVD_LUA_FRESH_STATE_COST``),
   paid once per activation - a unit that, for a node calling a model, is
   measured in seconds.

.. dec:: A script's environment is built from named parts
   :id: DEC_ENVIRONMENT_BY_NAME
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_LUA_DEFAULT_STATE_EXPOSES, EVD_LUA_PCALL_SWALLOWS_LIMITS, EVD_LUA_RANDOM_PER_PROCESS, EVD_LUA_ORDER_PER_PROCESS
   :statement: Agconflo shall build a script's environment from named libraries, leaving out every function that reads outside the activation or catches an error.

   The default is the wrong starting point, measured: ``mlua``'s default state,
   and its set named "safe", gave a script files, the process and a compiler
   (``EVD_LUA_DEFAULT_STATE_EXPOSES``). Removing what is known to be dangerous
   from a default leaves whatever nobody thought of; naming what goes in leaves
   out everything else by construction.

   Two kinds of function are left out of what does go in - the named libraries
   and the base library every state has - and each was found by measuring
   rather than by reading a list.

   Those that read outside the activation. ``math.random`` drew different
   numbers in different processes (``EVD_LUA_RANDOM_PER_PROCESS``), so a script
   using it produces output depending on something no wire carries. The clock
   and the environment are out already, with ``os``.

   Leaving it out does not make a script deterministic, and is not claimed to.
   The order ``pairs`` visits string keys in, and the address ``tostring`` gives
   a table, both differed between processes (``EVD_LUA_ORDER_PER_PROCESS``).
   Those are the order and naming of values a script already holds rather than
   values it reads from outside, and they are part of the base library every
   state has. Making a script's output reproducible is a question for the day a
   run has to be replayed.

   Those that catch an error. ``pcall`` let a script survive both of its limits
   and return normally (``EVD_LUA_PCALL_SWALLOWS_LIMITS``), and ``xpcall`` did the
   same, so as long as a script can catch an error, neither limit is a limit.
   The cost is real and accepted: a script cannot recover from an error of its
   own, and its activation fails instead, carrying what went wrong. A caller
   with a better idea of recovery is a caller, and has the run's failure to act
   on.

.. dec:: A script is handed over by the caller as text
   :id: DEC_SCRIPTS_FROM_CALLER
   :dec_status: accepted
   :decided_on: 2026-09-23
   :statement: Agconflo shall take each node type's script as text from the caller starting a run rather than reading it from a file or from a node type document.

   Which documents are read is already the caller's to say
   (``DEC_TYPES_IN_OWN_DOCUMENTS``), and nothing in this project's libraries
   opens a file but ``agconflo-runner``, which is such a caller
   (``DEC_RUNNER_CRATES``). A script is the same: the caller reads it from wherever it
   keeps it, and hands over the text with the name of the document it came from,
   which is what a fault in it is reported against.

   A key in the node type document naming a file was the first alternative, and
   it would make the library resolve paths - relative to what, and on which
   machine - for a question the caller can answer without it. Writing the script
   into the node type document was the second. Topology is data that is
   validated without executing it (``STKH_TOPOLOGY_AS_DATA``), and a document
   holding code as well as declarations is one a machine editing either has to
   be careful with both.

   What this leaves unrecorded is the link between a node type and its script,
   which lives in the caller. A command-line caller will want a convention for
   it, and that is decided when there is one: ``DEC_RUN_FROM_A_MANIFEST``.

.. dec:: A script is the body of its node's behaviour
   :id: DEC_SCRIPT_IS_THE_BODY
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_LUA_CHUNK_TAKES_ARGUMENTS, EVD_LUA_COMPILES_WITHOUT_RUNNING
   :statement: Agconflo shall run a script as the body of its node's behaviour, passing the activation's inputs and the host functions as its arguments.

   The script is what runs, top to bottom, once per activation: it reads its
   arguments with ``local given, host = ...`` and returns its output. Measured
   working (``EVD_LUA_CHUNK_TAKES_ARGUMENTS``).

   A script that returns a function for the host to call was the alternative,
   and it adds a way to be wrong that nothing can catch early. A chunk returning
   ``42`` instead of a function is well-formed, so compiling it reports nothing,
   and it fails only when run (``EVD_LUA_CHUNK_TAKES_ARGUMENTS``). With the
   script as the body, compiling it is the whole of what can be checked before a
   run starts (``EVD_LUA_COMPILES_WITHOUT_RUNNING``), and that is a check that
   runs none of it.

.. dec:: A script is limited by what it does, not by how long it takes
   :id: DEC_LIMITS_NOT_TIME
   :dec_status: accepted
   :decided_on: 2026-09-23
   :supported_by: EVD_LUA_LIMITS_STOP
   :statement: Agconflo shall limit a script by the instructions it executes and the memory it allocates rather than by elapsed time.

   The run's budget counts activations and cannot see inside one: a script that
   never returns is one activation that never ends. So a script needs limits of
   its own, and ``DEC_BUDGET_COUNTS_ACTIVATIONS`` has already made the argument
   against time. It is not reproducible, so nothing can assert on it, and it is
   wrong about what it guards - a slow machine is not a runaway script.

   Both limits were measured stopping a script, each on the shape it exists for,
   and saying which one it was (``EVD_LUA_LIMITS_STOP``). Which one a script meets
   first depends on its shape and on the Lua version, so each is reported as
   itself rather than as a general failure.

   The instruction count costs something on every activation that stays inside
   it: a hook every thousand instructions took a loop from 41 ms to 53 ms. That
   is accepted, because an activation with no limit is one that can hold a run
   for ever.

.. dec:: A script calls the context API and nothing else
   :id: DEC_HOST_FUNCTIONS_CONTEXT_API
   :dec_status: accepted
   :decided_on: 2026-09-23
   :statement: Agconflo shall give a script the context API as its only host functions.

   A judgement. What a script is given is what a node can do, and the smallest
   set that makes a node useful is the one ``DEC_CONTEXT_API`` already defines:
   make a context from text, compose one from others, and read a context's
   content, type and parts. The type its output is declared as comes with the
   activation, so a script need not repeat it.

   A script-shaped interface - helpers that exist because Lua lacks them - was
   the alternative, and it is what would tie the engine to one language. Keeping
   to the context API is what lets a second backend bind the same functions.

   Contexts reach a script as values it can call methods on and nothing else,
   which was measured rather than assumed: a script could not read or replace
   their metatable, nor add a field to one (``EVD_LUA_USERDATA_PROTECTED``).

.. dec:: The coroutine library is taken back out once the host functions exist
   :id: DEC_COROUTINES_CLOSED_AFTER_HOST
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_MLUA_ASYNC_LOADS_COROUTINE
   :statement: Agconflo shall remove the coroutine library from a script's environment after the host functions for its activation are made.

   ``DEC_ENVIRONMENT_BY_NAME`` never loads ``coroutine``, since resuming a
   coroutine hands back its error as a value, which is catching a limit under
   another name (``CREQ_HOST_NO_CATCHING``). Measured, ``mlua`` loads the library
   anyway when the first asynchronous function is made, and reads
   ``coroutine.yield`` into its own poller at that moment
   (``EVD_MLUA_ASYNC_LOADS_COROUTINE``). Removing the global afterwards takes it
   from the script and leaves the poller what it took.

   Removing it before the host functions are made was the alternative, and
   ``mlua`` would load it again on the first asynchronous function. Giving up
   asynchronous host functions was the other, and ``DEC_BEHAVIOUR_ASYNC`` needs
   them.

.. dec:: A script is given neither print nor collectgarbage
   :id: DEC_NO_PRINT_OR_COLLECTOR
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_LUA_DEFAULT_STATE_EXPOSES
   :statement: Agconflo shall leave print and collectgarbage out of a script's environment.

   Two functions of the base library that ``DEC_ENVIRONMENT_BY_NAME``'s two kinds
   do not cover, since neither reads outside the activation nor catches an
   error. ``print`` writes to the host process's output, where no wire carries
   it and no record holds it: a script's one way out is its output context.
   ``collectgarbage`` controls the state's collector, which is the host's to run
   under the memory limit rather than the script's.
