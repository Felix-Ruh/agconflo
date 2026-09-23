======================
Running node behaviour
======================

The first feature in which a node does something: each activation a run hands
out is performed by running a script its node type was given, and the script's
result is what the run is told. Every requirement here derives from a goal in
``stakeholder/authoring``, ``stakeholder/context`` or ``stakeholder/execution``,
and each is written against the decisions in ``decisions/behaviour`` rather than
re-opening them.

What a script does not do here is what keeps this a slice. It reaches no
provider, no file and no clock, it activates no node twice
(``DEC_ACTIVATION_ONCE_PER_RUN``), and nothing it does is persisted. A script
performs an activation the way any caller of the run would
(``DEC_RUN_IS_DRIVEN``), so everything the run already guarantees - which node
may activate, what it is given, the four ways a run ends and the outputs it
refuses - holds unchanged, and nothing here restates it.

Each requirement was checked by hand against the question no rule can ask:
could this be false while its parent is true? The body of each says how. Two
parents are judgements rather than readings, and those bodies say why the parent
is the one it is: the instruction limit's and the refusal before a run starts.

Five of the seven statements use the ``unwanted`` pattern and two
``ubiquitous``: what a node runs, and what it can read, are true of every
activation rather than triggered by something. That is the distribution a
feature made mostly of failure modes produces, and ``DEC_EARS_SIX_PATTERNS`` has
already been looked at again for a feature with the same shape.

.. feat_req:: A node does what its script says
   :id: FEAT_BEHAVIOUR_FROM_SCRIPT
   :derived_from: STKH_LIVE_BEHAVIOUR
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall perform each activation by running the script its caller supplied for the node type when starting the run.

   The parent asks that behaviour change without recompiling the engine. This
   says how: the behaviour is text handed over when a run starts
   (``DEC_SCRIPTS_FROM_CALLER``), run as a Lua script (``DEC_BEHAVIOUR_IN_LUA``).

   It can be false while the parent holds. Plugins loaded from a library built
   separately change behaviour without recompiling the engine, and so meet the
   parent word for word, while still needing a compiler to change a node - the
   cost the parent was written to remove. So can a script read once and cached
   for the life of a process, which changes nothing until the process restarts.
   A script supplied with each run is one that a changed script reaches on the
   next run.

   The test is the parent's own claim made concrete: one build, two runs of one
   workflow with two different scripts for one node type, and two different
   results.

.. feat_req:: A script's output is one context the run accepts
   :id: FEAT_BEHAVIOUR_ONE_CONTEXT
   :derived_from: STKH_ONE_OUTPUT
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a node's script returns anything other than exactly one context that the run accepts, then Agconflo shall fail that activation.

   The parent restricts a node to one output. A script is the first thing that
   can return any number of anything, and this is what the restriction means
   for it.

   It can be false while the parent holds. A host that takes the first of two
   values returned has restricted the node to one output, and has discarded the
   second without a word. One that turns a returned string into a context has
   given the node an output, of a type the host chose rather than the one
   declared. Both leave the parent met and the node's author misled.

   "That the run accepts" is what joins this to the run's own refusals: an
   output of the wrong type, or one the run already holds, is refused by the run
   (``CREQ_RUN_REFUSES_UNDECLARED_OUTPUT``, ``CREQ_RUN_REFUSES_HELD_IDENTIFIER``),
   and the script's activation then fails rather than being tried again, since a
   script run twice with the same inputs has no reason to answer differently.

.. feat_req:: A script reads only what its activation carries
   :id: FEAT_BEHAVIOUR_READS_ONLY_ITS_INPUTS
   :derived_from: STKH_EXPLICIT_CONTEXT
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall give a node's script nothing to read but the contexts its activation carries.

   The parent says what the engine gives a node. This says what a node can take,
   which is the half a script makes a question at all: a script is code, and
   code reaches for things.

   It can be false while the parent holds, in each of the ways measured. The run
   gives a node exactly what was wired to it, and a script given Lua's default
   state then reads files, the process's environment and the clock
   (``EVD_LUA_DEFAULT_STATE_EXPOSES``), draws numbers that differ from process to
   process (``EVD_LUA_RANDOM_PER_PROCESS``), or reads what the previous node's
   script left behind in a shared state (``EVD_LUA_SHARED_STATE_LEAKS``). None
   of those is a context, so none is anything the parent speaks of, and each is
   an input the node's output depends on that no wire shows.

   ``DEC_ENVIRONMENT_BY_NAME`` and ``DEC_STATE_PER_ACTIVATION`` are how it is
   met.

.. feat_req:: A script's error fails its activation and says what it was
   :id: FEAT_BEHAVIOUR_ERROR_CARRIED
   :derived_from: STKH_TYPED_FAILURE
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a node's script raises an error, then Agconflo shall fail that activation carrying the error's message.

   The parent says which failure occurred is reported when a node fails. A
   script's error is the commonest way a node will, and this is what reporting
   it means.

   It can be false while the parent holds. A failure that says "the script
   failed" has said which failure occurred at the level of the host - and told
   the person who wrote the script nothing about which line, or why. Carrying
   the message is what lets the script's author act on it.

   The message is all Lua has to carry, and not always that: an error raised
   with a table rather than a string renders as the table's address, which
   changes from run to run. That the failure is a script error rather than a
   limit or a wrong output is what is always carried, as its kind.

.. feat_req:: A script that runs too long is stopped
   :id: FEAT_BEHAVIOUR_INSTRUCTION_LIMIT
   :derived_from: STKH_STUCK_RUN
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a node's script executes more instructions than its limit allows, then Agconflo shall fail that activation.

   This derivation is a judgement, and the reason for the parent is worth
   giving. The parent reports a run in which no node can make further progress.
   A script in an endless loop is exactly that run: nothing is running that will
   ever finish, and because the run is waiting on an activation that never
   returns, it never gets the chance to say so. Nothing reports it at all.

   It can be false while the parent holds. The run's quiescence report
   (``CREQ_RUN_ENDS_QUIESCENT``) meets the parent for every run the engine can
   see into, and cannot see into an activation. ``STKH_STEP_BUDGET`` was the
   other candidate parent and is not true of it: the budget counts activations,
   and an endless script is one activation.

   Counted in instructions rather than time (``DEC_LIMITS_NOT_TIME``), so the
   same script stops at the same point on every machine.

.. feat_req:: A script that takes too much memory is stopped
   :id: FEAT_BEHAVIOUR_MEMORY_LIMIT
   :derived_from: STKH_TYPED_FAILURE
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a node's script allocates more memory than its limit allows, then Agconflo shall fail that activation.

   The parent says which failure occurred is reported when a node fails. A
   script allocating without limit does not fail in a way anything can report:
   the process runs out, and whatever stops it stops everything, the run and its
   record included.

   It can be false while the parent holds. Every failure a script can report
   about itself is reported, and the one that ends the process is not a failure
   anything is left to report. A limit turns it into an activation that failed,
   which is a failure the run carries like any other.

   It is a separate requirement from the instruction limit because the two are
   met separately and fail separately. Which one a script meets first depends on
   its shape and on the Lua version (``EVD_LUA_LIMITS_STOP``), and a host with
   only one limit has a script it cannot stop.

.. feat_req:: A run whose scripts cannot run does not start
   :id: FEAT_BEHAVIOUR_REFUSED_BEFORE_START
   :derived_from: STKH_WIRING_CHECKED
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a node type named by an instance of a workflow has no script or its script does not compile, then Agconflo shall refuse to start the run.

   This derivation is a judgement too, and the reason for the parent is in its
   body: the value of refusing an invalid workflow before any node runs is that a
   mistake costs a rejection rather than half of an expensive run. A node type
   with no script is that mistake. Found when the node is reached, it costs
   every activation before it.

   It can be false while the parent holds. The wiring can be sound, so the
   parent is met and the run starts, and the fifth node then has nothing to run.
   ``STKH_EXPLICIT_CONTEXT`` and ``STKH_LIVE_BEHAVIOUR`` were the other
   candidates, and neither is about when a mistake is found.

   "Does not compile" is the whole of what can be known before a run without
   running any of the script (``EVD_LUA_COMPILES_WITHOUT_RUNNING``). A call to a
   function that does not exist compiles, and fails when the script runs, as a
   script error.
