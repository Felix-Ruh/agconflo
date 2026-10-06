=========================
Code shared among scripts
=========================

A module: Lua text supplied with a run under a name, which a script reaches
by requiring that name. Every requirement here derives from
``STKH_SHARED_SCRIPT_CODE``, some from others beside it, and each is written
against the decisions in ``decisions/modules`` and
``DEC_MODULES_IN_THE_MANIFEST``.

.. feat_req:: A script that requires a module is given what the module returned
   :id: FEAT_SCRIPT_REQUIRES_MODULE
   :derived_from: STKH_SHARED_SCRIPT_CODE
   :ears_pattern: event
   :verification_method: test
   :statement: When a script requires a module supplied with its run, Agconflo shall give it what that module's code returned, running that code once in the activation.

   The parent at the level of one activation: code written once, in one
   module, reaches every script that requires it.

   It can be false while the parent holds. A host that pasted a module's
   text into every script before running it shares code written once, in the
   letter of the parent, and gives each script its own copy of the module's
   state: a table the module keeps, filled through one helper, is empty to
   another. Running it once in the activation is what makes it one module
   rather than several copies.

   It claims no more than the parent: how a script requires a module, and
   what a module may reach, are decisions (``DEC_MODULES_REQUIRED_BY_NAME``).

.. feat_req:: Requiring a name no module is supplied under fails the activation naming it
   :id: FEAT_UNSUPPLIED_MODULE_FAILS
   :derived_from: STKH_SHARED_SCRIPT_CODE, STKH_TYPED_FAILURE
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a script requires a name no module is supplied with its run under, then Agconflo shall fail that activation naming the name.

   ``STKH_TYPED_FAILURE`` asks which failure occurred. A module misspelt, or
   left out of the manifest, is the commonest way a require goes wrong, and
   an error saying only that something was missing names neither which name
   nor that it was a module.

   It can be false while both parents hold. A host that looked a name up on
   disk when no module had it would share code written once and report
   whatever failure it met there - and would read a file no wire shows, which
   ``FEAT_BEHAVIOUR_READS_ONLY_ITS_INPUTS`` refuses.

.. feat_req:: A module that cannot be used refuses the run before it starts
   :id: FEAT_MODULE_FAULT_REFUSED
   :derived_from: STKH_SHARED_SCRIPT_CODE, STKH_WIRING_CHECKED
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a module supplied with a run does not compile or two modules are supplied under one name, then Agconflo shall refuse to start the run naming the name.

   ``STKH_WIRING_CHECKED`` refuses a workflow that cannot run as written
   before any node runs, and a module that does not compile makes one: the
   first script requiring it fails, after every activation before it was
   paid for. Which modules a run uses is not known before it runs, so every
   module supplied is checked (``DEC_MODULES_CHECKED_BEFORE_A_RUN``).

   It can be false while both parents hold, as
   ``FEAT_BEHAVIOUR_REFUSED_BEFORE_START`` can for a script: the wiring is
   sound, the run starts, and the fault is found where the module is first
   required.

.. feat_req:: A manifest supplies its run with the modules it names
   :id: FEAT_MODULES_FROM_THE_MANIFEST
   :derived_from: STKH_SHARED_SCRIPT_CODE, STKH_RUN_FROM_DOCUMENTS
   :ears_pattern: event
   :verification_method: test
   :statement: When a run is started from a manifest naming modules, Agconflo shall supply the run with each module under the name the manifest gives it.

   ``STKH_RUN_FROM_DOCUMENTS`` has a person run a workflow from its documents
   with no program written to host it. Shared code is one of those documents
   now, and a host that could not be told of it would leave a person writing
   a program to supply it, or copying it into every script.

.. feat_arch:: Shared code is held by the behaviour set, reached through the script host and read by the project reader
   :id: ARCH_SHARED_MODULES
   :realises: FEAT_SCRIPT_REQUIRES_MODULE, FEAT_UNSUPPLIED_MODULE_FAILS, FEAT_MODULE_FAULT_REFUSED, FEAT_MODULES_FROM_THE_MANIFEST
   :uses: COMP_BEHAVIOUR_SET, COMP_SCRIPT_HOST, COMP_PROJECT_READER
   :statement: Agconflo shall allocate sharing code among a run's scripts to the behaviour set, the script host and the project reader.

   - The behaviour set holds each module supplied beside the scripts, and
     checks that every one compiles and that no name is given two.
   - The script host gives every script ``require``, which runs a module in
     the activation and raises an error for a name it does not know.
   - The project reader reads the modules a manifest names.

   The runner and the command line are not among them: a manifest's modules
   reach a run in the behaviours the project reader gives, and a module's
   fault is refused and reported as a script's is, through the same values.
