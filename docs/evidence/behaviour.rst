=====================================
Evidence about running node behaviour
=====================================

Measurements the choice of a script language and the shape of its sandbox rest
on. All of them were taken on the same day in throwaway crates outside the
repository, against the newest release of everything involved: ``mlua`` 0.12.1
with its ``vendored`` feature, which built Lua 5.5 from ``lua-src`` 551.0.2, Lua
5.4 from the same crate, and Luau 0.736 from ``luau0-src`` 0.21.0. Each finding
says which of the three it was taken on, because a sandbox is exactly what
differs between them.

One crate depended on ``agconflo-core`` by path at ``4dc940b`` and drove its run
with a script per node type: each activation was performed by loading that
type's script into a Lua state, handing it the activation's inputs and two host
functions that make contexts, and reporting what it returned. The others probed
``mlua`` alone. Where a finding names the environment a script was given, that
is one built from the ``string``, ``table``, ``math`` and ``utf8`` libraries
with ``dofile``, ``loadfile``, ``load``, ``require``, ``collectgarbage`` and
``print`` removed - the smallest environment the spike could write useful
scripts in - and it is called the reduced environment below.

.. evd:: Lua's default state gives a script files and the process
   :id: EVD_LUA_DEFAULT_STATE_EXPOSES
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: A state from mlua's Lua::new, and one from its ALL_SAFE library set, gave a script io, os, require, package, dofile, loadfile and load on both Lua 5.4 and Lua 5.5.

   Each name was looked up in the globals from inside a script and reported when
   it was not ``nil``. ``io`` and ``os`` are files, processes and the clock;
   ``require``, ``dofile`` and ``loadfile`` read files; ``load`` compiles text a
   script assembled at run time, which is how anything removed from the
   environment would come back.

   The two gave the same list on both versions, and the ``os`` they gave held
   ``execute``, ``exit``, ``getenv``, ``remove`` and ``rename`` among its
   functions. "Safe" in the set's name is about the host process's memory, not a
   sandbox. The reduced environment exposed none of them.

.. evd:: A shared state carried one activation's writes into the next
   :id: EVD_LUA_SHARED_STATE_LEAKS
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: With one Lua state shared across a run, a global and a field added to the string library by one node's script were read by the next node's script, and with a fresh state per activation both read as nil.

   The first node's script set ``secret`` and ``string.secret``; the second
   rendered both into its output. Shared, the run completed with
   ``from seed / via string lib``; fresh, with ``nil / nil``. Same results on Lua
   5.4 and 5.5. The second node was not wired to the first for either value, so
   in the shared state it read something no wire shows.

   Clearing globals between activations would not close it: the library tables
   are shared too, and the string library is also every string's metatable, so a
   write there reaches every script that touches a string.

.. evd:: A fresh Lua state costs tens of microseconds
   :id: EVD_LUA_FRESH_STATE_COST
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: Creating 1000 states in the reduced environment with an instruction hook and a memory limit took 25.1 ms on Lua 5.5, 26.7 ms on Lua 5.4 and 88.9 ms on Luau, in a release build, and a fresh Lua 5.5 state held 15929 bytes.

   Measured on the same machine in one run each. The whole three-node run with a
   fresh state per activation took 0.29 ms on Lua 5.5 against 0.05 ms with one
   state shared, so the difference is the states and not the scripts.

.. evd:: Luau's sandbox still gives a script the process and a compiler
   :id: EVD_LUAU_SANDBOX_EXPOSES
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: A Luau 0.736 state with mlua's sandbox mode switched on gave a script os, require, loadstring, debug, collectgarbage and print.

   The same probe as for Lua. Luau's ``os`` is a reduced one - it held
   ``clock``, ``date``, ``difftime`` and ``time`` - so what it still carries is
   the clock, and ``loadstring`` compiles text built at run time.

   What the sandbox did do is make the libraries read-only: the script writing
   ``string.secret`` failed with ``attempt to modify a readonly table``. That is
   the property a fresh state per activation gives Lua anyway
   (``EVD_LUA_SHARED_STATE_LEAKS``).

.. evd:: Luau takes about twice as long to build as Lua
   :id: EVD_LUAU_BUILD_COST
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: A clean debug build of a crate depending on mlua 0.12.1 alone took 6.0 s and 5.6 s with vendored Lua 5.5, and 12.3 s both times with vendored Luau.

   Each build started from an empty target directory with the sources already
   fetched, twice per language. A debug build is what the commit hook and
   continuous integration compile.

   An earlier comparison while planning read it as six times; that set an
   incremental Lua build against a release Luau build, and is corrected here.

.. evd:: pcall let a script survive both limits
   :id: EVD_LUA_PCALL_SWALLOWS_LIMITS
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: On Lua 5.5 a script that called a runaway loop through pcall caught the instruction limit's error and returned normally, and one that called string.rep through pcall under an 8 MiB limit caught not enough memory and went on to fill a table.

   The instruction limit was a hook raising an error once a count was passed,
   and the memory limit ``mlua``'s own. Both errors are ordinary Lua errors by
   the time a script sees them, so anything that catches an error catches them:
   ``xpcall`` did the same for the memory error.

   A host reading the script's result would have recorded both activations as
   successful. The hook's own flag showed the instruction limit had been hit;
   for the memory limit the spike had no such flag to read.

.. evd:: math.random differed between processes
   :id: EVD_LUA_RANDOM_PER_PROCESS
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: On Lua 5.5 two states in one process drew the same two numbers from math.random, and three separate processes drew three different first numbers.

   Within one process both states drew 0.0681 then 0.4220. The first draws of
   three processes were 0.0681, 0.4273 and 0.2505. A script using it produces
   output that depends on which process ran it, which is a value that reached
   the node through no wire.

.. evd:: A script compiles without any of it running
   :id: EVD_LUA_COMPILES_WITHOUT_RUNNING
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: On Lua 5.5 compiling a chunk that assigns a global left the global unassigned, a syntax error was reported with the chunk's name and place, and a call to an undefined function compiled.

   The chunk was ``touched = true; return 1``, compiled with ``mlua``'s
   ``into_function`` and never called. The syntax error read
   ``[string "types/echo.lua"]:1: <name> or '...' expected near 'end'`` - the
   name given to the chunk, which is how a document reaches the message.

   The last is the limit of what compiling can find: a name that resolves to
   nothing is a run-time error in Lua, so it surfaces only when the script runs.

.. evd:: A chunk takes its arguments as varargs
   :id: EVD_LUA_CHUNK_TAKES_ARGUMENTS
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: On Lua 5.5 a chunk called with two tables read them through its varargs, and a chunk expected to return a function that returned 42 failed only once it was run.

   ``local given, host = ...`` at the top of the chunk received both tables. The
   second shape is the other way to write a behaviour - a chunk returning the
   function to call - and its failure, ``error converting Lua integer to
   function``, is one no compiler reports, because the chunk is well-formed.

.. evd:: The limits stop a script and say which one
   :id: EVD_LUA_LIMITS_STOP
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: On Lua 5.5 with a one-million instruction limit and an 8 MiB memory limit, an empty loop was stopped by the instruction hook, one 1 GiB string.rep by a memory error, and a table filled in a loop by the hook first.

   The hook ran every 1000 instructions and set a flag once its count passed
   the limit. The memory error came with that flag unset, so the two are told
   apart by the flag and the error's kind rather than by reading a message.

   The empty loop stopped in 3.6 ms. The memory error left its state usable: a
   script run in it afterwards returned normally. On Lua 5.4 a table filled the
   same way, to a larger count, hit the memory limit first, so which limit a
   script meets depends on its shape and on the version, and only a script that
   can meet just one of them tells the two apart.

   The hook costs something on work that stays inside the limit: a loop of three
   million additions took 41 ms without it and 53 ms with it.

.. evd:: A script cannot reach into a context it is given
   :id: EVD_LUA_USERDATA_PROTECTED
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: On Lua 5.5 a context handed to a script as mlua userdata answered getmetatable with false, refused setmetatable, and refused a field assignment.

   So a script can call the methods it is given and nothing else: it cannot
   replace ``render`` to change what a context says, attach a field to carry
   something along, or give a context of its own making the look of one the
   host made.

.. evd:: Table addresses and key order differed between processes
   :id: EVD_LUA_ORDER_PER_PROCESS
   :evd_kind: measurement
   :observed_on: 2026-09-23
   :observation: On Lua 5.5 in the reduced environment, three processes gave three different addresses from tostring of a table and from string.format with percent-p, and visited the same twelve string keys with pairs in three different orders.

   Taken while writing the requirement that a script reads nothing but its
   inputs, to find out whether leaving ``math.random`` out had made a script's
   output a function of its inputs alone. It had not. The order ``pairs`` visits
   string keys in depends on a hash seed chosen per process, and a script that
   joins keys in that order produces different text from the same inputs.

   Neither is a value a script reads from outside, the way it reads a random
   number: both are the order and the naming of values the script already
   holds. They are recorded because nothing here makes a script deterministic,
   and nothing should be read as claiming it.
