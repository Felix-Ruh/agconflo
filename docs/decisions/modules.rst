=======================
Decisions about modules
=======================

How the scripts of a workflow share code written once
(``STKH_SHARED_SCRIPT_CODE``): a module, supplied with the run as a script
is, and reached from a script by its name.

.. dec:: A script reaches shared code by requiring a module supplied with its run
   :id: DEC_MODULES_REQUIRED_BY_NAME
   :dec_status: accepted
   :decided_on: 2026-10-06
   :statement: Agconflo shall give a script a require function that runs the module supplied with the run under the name it is given once in the activation and returns what that module returned.

   A module is Lua text supplied with the run under a name, as a script is
   supplied for a node type (``DEC_SCRIPTS_FROM_CALLER``). A script writes
   ``local helpers = require('helpers')`` and gets whatever the module's text
   returned, usually a table of functions. That is how Lua code is shared
   everywhere else, so a script still reads as Lua and nothing new is learnt.

   It is not Lua's ``require``, which is left out of every state
   (``DEC_ENVIRONMENT_BY_NAME``) because it reads files and loads libraries
   built for the machine. This one reaches only the texts the run was
   handed, by the name each was handed under: a name no module is supplied
   under is an error raised in the script, and so is any text a script
   assembles, which is never a name. So it is no compiler a script can feed,
   which is the second failure mode ``CREQ_HOST_NOTHING_OUTSIDE`` names.

   A module runs in the activation's own state and on the script's own
   thread, so the limits on instructions and memory hold over it as over the
   script (``DEC_HOOK_ON_THE_THREAD``). It runs at most once in an
   activation: requiring it again gives back what it returned the first
   time. A state belongs to one activation (``DEC_STATE_PER_ACTIVATION``), so
   nothing a module keeps in one activation reaches another.

   A module is given only its name, not the host functions: a helper that
   needs them is handed them by the script calling it, so a module can do no
   more than a script lets it.

   Requiring a module while it is still running - a module requiring itself,
   or two requiring each other - is an error naming the module, where Lua's
   own ``require`` would recurse until it ran out of room.

   Not adopted:

   - **A prelude run before every script**: shared code a script does not
     say it uses, and every helper a global of every script.
   - **Generating each script from shared parts**, as #71's generator did:
     the behaviour's real source sits outside the workflow's documents, in a
     program of another toolchain (``DEC_NO_PYTHON_IN_THE_REPOSITORY``).
   - **Lua's own require, its searchers replaced**: it comes with the package
     library, whose ``loadlib`` reaches the machine, and taking that out of a
     default is what ``DEC_ENVIRONMENT_BY_NAME`` rejected.

.. dec:: Every module supplied with a run is compiled before it starts
   :id: DEC_MODULES_CHECKED_BEFORE_A_RUN
   :dec_status: accepted
   :decided_on: 2026-10-06
   :statement: Agconflo shall refuse to start a run supplied with a module that does not compile or with two modules under one name.

   What a script requires is decided as it runs - a name is a string, and a
   script could make one - so nothing short of running the scripts says
   which modules a run uses. A script is compiled only when an instance names
   its node type (``CREQ_BEHAVIOURS_REFUSE_UNCOMPILABLE``), since a
   definition says which node types it uses; nothing says it of a module.
   So every module supplied is compiled, and one that does not compile
   refuses the run before anything runs (``STKH_WIRING_CHECKED``), rather
   than failing the first activation that requires it after every activation
   before it was paid for.

   The cost is that a broken module nothing requires refuses the run. A
   module is supplied with this run, by its manifest, so it is this run's to
   fix.

   Two modules under one name are refused alike: which of them a require
   would get is a guess either way.
