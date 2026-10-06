====================================
Code shared among scripts test cases
====================================

How each requirement in ``components/modules`` and ``features/modules`` is to
be checked. Results are never written here: they are imported from the test
runner.

A case's id is the path of the Rust test that implements it, uppercased. The
cases live in ``agconflo-lua``'s modules ``behaviours`` and ``host``, in
``agconflo-runner``'s module ``project``, and in ``agconflo-cli``'s
integration test file ``command``. No case reaches a provider, a container or
the network.

.. test_case:: A module that does not compile, or two under one name, refuses the run
   :id: TEST_BEHAVIOURS_MODULE_FAULTS_REFUSED
   :verifies: CREQ_BEHAVIOURS_REFUSE_MODULE_FAULTS
   :test_kind: error_path
   :coverage: full

   A module that does not compile is refused naming its name, its document
   and line 2 of it, and no script requires it; two modules under one name
   are refused naming both documents; a broken module beside a node type
   with no script gives both faults in one refusal, the script's first. A
   module whose code raises an error when run passes the check and the run
   starts - the control that checking runs nothing.

   Catches: only the modules a script names checked, a module run while it
   is checked, the message or the document dropped, the second module kept.

.. test_case:: A required module is run once and what it returned given back every time
   :id: TEST_HOST_MODULE_REQUIRED
   :verifies: CREQ_HOST_REQUIRE_RUNS_MODULE
   :test_kind: positive
   :coverage: full

   A script requiring ``help`` twice gets one table, calls its function, and
   finds the module ran once; a module returning nothing gives ``true``; a
   module requiring another gives what that one returned; and a module's
   arguments are its name alone.

   Catches: the module run at every require, the module given the host
   functions.

.. test_case:: A module is held to the activation's limits
   :id: TEST_HOST_MODULE_UNDER_THE_LIMITS
   :verifies: CREQ_HOST_REQUIRE_RUNS_MODULE
   :test_kind: error_path
   :coverage: partial

   A module looping without end when required fails its activation for the
   instruction limit, and one building a string past the memory limit for the
   memory limit, each within the time a script stopped by its limit takes.

   Catches: the module run on another thread, beyond the instruction limit.

.. test_case:: A module's state is not kept from one activation to the next
   :id: TEST_HOST_MODULE_PER_ACTIVATION
   :verifies: CREQ_HOST_REQUIRE_RUNS_MODULE
   :test_kind: positive
   :coverage: partial

   Two activations each require a module counting in a global how often it
   ran, and each finds it ran once.

.. test_case:: Requiring what no module is supplied under fails naming it
   :id: TEST_HOST_UNSUPPLIED_MODULE_FAILS
   :verifies: CREQ_HOST_REQUIRE_UNSUPPLIED
   :test_kind: error_path
   :coverage: full

   Requiring ``absent``, ``io``, ``os`` and the text ``return 1`` each fails
   the activation as a script error naming what was required, ``io``'s table
   never given back and the text never run; requiring ``42`` fails saying a
   module's name is a string. A supplied module required in the same way
   runs - the control.

   Catches: a library or a file reached, text compiled, nothing given back.

.. test_case:: Requiring a module while it runs fails naming it
   :id: TEST_HOST_MODULE_WHILE_RUNNING_FAILS
   :verifies: CREQ_HOST_REQUIRE_WHILE_RUNNING
   :test_kind: error_path
   :coverage: full

   A module requiring itself, and two requiring each other, each fail the
   activation as a script error naming the module required while it ran.

   Catches: recursion until the stack runs out, nothing given back.

.. test_case:: A manifest gives each module it names, read beside it
   :id: TEST_PROJECT_READS_MODULES
   :verifies: CREQ_PROJECT_READS_MODULES
   :test_kind: positive
   :coverage: full

   A manifest naming ``help`` and ``lib.fmt`` under ``modules``, read from
   another working directory, gives both by name with their files as the
   manifest writes them and their texts byte for byte, and the behaviours it
   gives carry them; a manifest naming none gives none.

   Catches: a path resolved against the working directory, the modules left
   out of the behaviours, a module's text changed.

.. test_case:: A module's file that cannot be read is refused naming it
   :id: TEST_PROJECT_MODULE_UNREADABLE
   :verifies: CREQ_PROJECT_REFUSES_UNREADABLE
   :test_kind: error_path
   :coverage: partial

   A module naming a file that is not there is refused naming its path as the
   manifest writes it, and a module given as a number is refused at its key.

.. test_case:: Two scripts share a module the manifest names
   :id: TEST_COMMAND_MODULE_SHARED
   :verifies: FEAT_MODULES_FROM_THE_MANIFEST, FEAT_SCRIPT_REQUIRES_MODULE
   :test_kind: positive
   :coverage: partial

   ``agconflo run`` of a manifest naming a module that two node types'
   scripts require completes, each script's part of the result made by the
   module's helper; ``agconflo check`` of it finds nothing.

.. test_case:: A script requiring a module the manifest does not name fails naming it
   :id: TEST_COMMAND_UNSUPPLIED_MODULE_FAILS
   :verifies: FEAT_UNSUPPLIED_MODULE_FAILS
   :test_kind: error_path
   :coverage: full

   A run whose script requires a name its manifest gives no module exits 5,
   standard error naming the name.

.. test_case:: A module that does not compile refuses the run before it starts
   :id: TEST_COMMAND_MODULE_FAULT_REFUSED
   :verifies: FEAT_MODULE_FAULT_REFUSED
   :test_kind: error_path
   :coverage: full

   ``agconflo check`` and ``agconflo run`` of a manifest naming a module that
   does not compile each exit 4, naming the module and the line of its
   document, and the run leaves no record file.
