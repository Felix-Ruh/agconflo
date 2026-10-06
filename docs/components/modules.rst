=======================================
Components of code shared among scripts
=======================================

The requirements ``ARCH_SHARED_MODULES`` allocates to the three components it
uses: the behaviour set and the script host, defined in
``components/behaviour``, and the project reader, defined in
``components/runner``. Each title is the grammatical subject of the
requirements allocated to it, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

.. comp_req:: A module that does not compile, or a name given two modules, is a fault
   :id: CREQ_BEHAVIOURS_REFUSE_MODULE_FAULTS
   :derived_from: FEAT_MODULE_FAULT_REFUSED
   :allocated_to: COMP_BEHAVIOUR_SET
   :ears_pattern: unwanted
   :statement: If a module supplied does not compile or two modules are supplied under one name, then Behaviour set shall report a fault naming that name, the documents and for one that does not compile the compiler's message.

   ``DEC_MODULES_CHECKED_BEFORE_A_RUN``: every module supplied is compiled,
   whether a script requires it or not, and none is run. Its faults come in
   the same refusal as the scripts', after them, in the order the modules
   were supplied.

   Failure modes:

   - **Only the modules a script names checked**, by reading the scripts for
     calls to ``require``: a name a script builds is missed, and its broken
     module found when the run reaches it.
   - **A module run while it is checked**, its code acting before any node.
   - **The compiler's message dropped**, or the module's document not named.
   - **The second of two modules under one name kept without a word**, and a
     require given whichever came last.

.. comp_req:: A required module is run once in the activation, and what it returned given back
   :id: CREQ_HOST_REQUIRE_RUNS_MODULE
   :derived_from: FEAT_SCRIPT_REQUIRES_MODULE
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a script requires a name a module is supplied under, Script host shall run that module on the script's thread the first time and give back what it returned or true when it returned nothing every time.

   ``DEC_MODULES_REQUIRED_BY_NAME``. A module is given its name as its one
   argument and the activation's globals as its own; it may require another
   module. ``true`` stands for a module that returned nothing, as in Lua's own
   ``require``, so that a second require knows the first ran.

   Failure modes:

   - **The module run on another thread**, beyond the instruction limit,
     which is set on the script's thread (``EVD_LUA_HOOK_PER_THREAD``): an
     endless loop in a module runs until the process is killed.
   - **The module run at every require**, and each helper's table a new one.
   - **The module given the host functions**, so that code no script hands
     them to can call a model.

.. comp_req:: Requiring what no module is supplied under raises an error naming it
   :id: CREQ_HOST_REQUIRE_UNSUPPLIED
   :derived_from: FEAT_UNSUPPLIED_MODULE_FAILS
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a script requires a name no module is supplied under or requires anything but a string, then Script host shall raise an error in the script naming what was required.

   No script can catch it (``CREQ_HOST_NO_CATCHING``), so its activation
   fails carrying the message.

   Failure modes:

   - **A library or a file reached** for a name no module has: ``io``'s
     table given back for ``require('io')``, or a file of that name read.
   - **Text compiled**: Lua a script assembled and passed to ``require``
     run, which is the compiler ``CREQ_HOST_NOTHING_OUTSIDE`` keeps out.
   - **Nothing given back** instead of an error, and the script failing
     later, where the module was to be used, naming no module.

.. comp_req:: Requiring a module while it runs raises an error naming it
   :id: CREQ_HOST_REQUIRE_WHILE_RUNNING
   :derived_from: FEAT_SCRIPT_REQUIRES_MODULE
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a module is required while it is running, then Script host shall raise an error in the script naming that module.

   A module requiring itself, or two requiring each other, has nothing to
   give back yet: what it returns is what is being waited for.

   Failure modes:

   - **Recursion until the stack runs out**, and an error about the stack
     naming no module.
   - **Nothing given back** while the module runs, and the script failing
     where it uses that, naming no module.

.. comp_req:: A manifest gives each module it names under its name
   :id: CREQ_PROJECT_READS_MODULES
   :derived_from: FEAT_MODULES_FROM_THE_MANIFEST
   :allocated_to: COMP_PROJECT_READER
   :ears_pattern: event
   :statement: When a manifest names modules, Project reader shall give each under the name its key gives with its file as the manifest writes it and its text read from its path relative to the manifest's directory.

   ``DEC_MODULES_IN_THE_MANIFEST``. A module's file is read as a script's is,
   and one that cannot be read is refused as a script's is
   (``CREQ_PROJECT_REFUSES_UNREADABLE``). The behaviours the project gives a
   run carry the modules beside the scripts.

   Failure modes:

   - **A path resolved against the working directory**, and a module read
     from wherever the person stood.
   - **The modules left out of the behaviours** a run is started with, and
     every require failing as unsupplied.
   - **A module's text changed on the way**, and a fault reported at a line
     the file does not have.
