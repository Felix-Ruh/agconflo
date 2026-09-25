====================================
Components of running from documents
====================================

The components ``ARCH_RUNNER`` divides running a workflow from its documents
between, four in ``agconflo-runner`` and one in ``agconflo-cli``, and the
requirements allocated to them. Each title is the grammatical subject of the
requirements allocated to it, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

.. comp:: Project reader
   :id: COMP_PROJECT_READER
   :crate: agconflo-runner

   Reads a manifest and every file it names - the workflow document, the node
   type documents and the scripts - into what a scripted run is started with:
   the definition, the behaviours, the budget and the limits. Or refuses them,
   saying which file and where.

   Everything it does is a question about files and their text, with no run
   in it, which is what separates it from the runner that uses what it read.

.. comp:: Model map
   :id: COMP_MODEL_MAP
   :crate: agconflo-runner

   Reads a model mapping into the roster a scripted run calls models through:
   for each role, the model, the endpoint when one is given, and the key from
   the variable named. Or refuses it, at the model or the variable at fault.

   It answers where calls would go and with which key, and sends nothing.

.. comp:: Record keeper
   :id: COMP_RECORD_KEEPER
   :crate: agconflo-runner

   Holds a record file for one run: takes it, refusing one another run holds;
   replaces its contents with each record it is handed, whole; remembers a
   record it could not write; and lets it go when the run stops.

   It knows nothing of what a record says. That is the run record's, in the
   core.

.. comp:: Runner
   :id: COMP_RUNNER
   :crate: agconflo-runner

   Starts, resumes, answers or checks a run from what the project reader and
   the model map read, keeping its records through the record keeper, and
   hands back how the run stopped - its ending, the step it awaits, or why it
   was refused - with whether its record was kept.

   Its entry points are futures polled on one thread
   (``DEC_RUNNER_ON_ONE_THREAD``), and it prints nothing.

.. comp:: Command line
   :id: COMP_COMMAND_LINE
   :crate: agconflo-cli

   The ``agconflo`` binary: reads the command a person typed, asks the runner
   for it, prints what the runner handed back and exits with the status for
   it.
