===============================
Components of performing a tool
===============================

The components ``ARCH_TOOLS`` divides performing a tool within its grant
between, three new ones in ``agconflo-runner`` defined here and three of
running from documents defined in ``components/runner``, and the requirements
allocated to all six. Each title is the grammatical subject of the
requirements allocated to it, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

.. comp:: Grants reader
   :id: COMP_GRANTS_READER
   :crate: agconflo-runner

   Reads a grants file into what a run's tools may do: the image, each folder
   by name with whether it may be written, whether the network is granted,
   the actions allowed, and the limits on a command's time and output. Or
   refuses it, at the key or the folder at fault.

   Like the project reader, it answers a question about a file and its text,
   and nothing it reads is acted on until the runner asks.

.. comp:: Sandbox
   :id: COMP_SANDBOX
   :crate: agconflo-runner

   The container a run's tool steps are performed in, reached through the
   ``docker`` command: finds the image present, removes what a killed run
   left, makes the container locked down with the granted folders mounted,
   runs one step's command in it within its limits, hands back its output and
   exit status or the engine's own failure, and removes the container.

   It knows what a container is and nothing of what a step means.

.. comp:: Tool performer
   :id: COMP_TOOL_PERFORMER
   :crate: agconflo-runner

   Performs one tool step: reads the step's inputs by the parameters its
   action takes, refuses a path the action may not be given, asks the sandbox
   for what the action does, and makes the text the step is answered with -
   the file read, the write done, the command's status and output, or what
   went wrong.

   It knows the three actions and asks the sandbox for everything else, so
   the runner's handling of tools is tested against a stand-in for the
   sandbox.
