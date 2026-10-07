=========================
Working through a browser
=========================

What a person sees of Agconflo and does with it in a web browser: a workflow
as its graph, a run on that graph, and, built on those views, starting runs
and editing workflows. The views come first and the rest is built on them, so
the goals are listed in the order they are meant to be met.

.. stkh_req:: A person works with Agconflo in a web browser
   :id: STKH_WEB_INTERFACE
   :stakeholder: user
   :statement: Agconflo shall give a person a web interface to its workflows and their runs.

   This is the interface the maintainer means to be the main one, which
   ``STKH_RUN_FROM_DOCUMENTS`` leaves to a goal of its own. A command line
   runs a workflow and prints where the run stopped, and that is all a person
   learns from it: a workflow's wiring is read out of its documents, and what
   a run did out of its record, by hand. Both are graphs, and a graph is read
   best as one.

   It starts barebones, as an overview of workflows and their runs, and grows
   into running and editing them; the goals below say what each step adds.
   It does not replace the command line, nor the tool calls an agent makes
   (``STKH_MACHINE_AUTHORING``). Each stays a way in, and this is the one a
   person is meant to reach for first.

.. stkh_req:: The web interface grows by adding views
   :id: STKH_WEB_VIEWS_ADDED
   :stakeholder: maintainer
   :statement: Agconflo shall let a view be added to its web interface without changing the views it already has.

   The interface starts barebones and is meant to grow: a workflow's graph,
   then a run on it, then running and editing, and views nobody has thought
   of yet. Where each addition reaches into what is there, every one costs
   more than the last and puts what already works at risk. Kept apart, a view
   is added beside the others, and they go on working as they did.

   It names no mechanism. What a view is, and what the views share, are for
   the decisions below it to say.

   It is separate from ``STKH_WEB_INTERFACE`` because either can hold without
   the other: an interface can be built as one piece that every change
   reaches into, and views kept apart can belong to something other than a
   web interface.

.. stkh_req:: A workflow is shown as its graph
   :id: STKH_WORKFLOW_SHOWN
   :stakeholder: user
   :statement: Agconflo shall show a person a workflow as a graph whose nodes and connections show their definitions when the person selects them.

   A workflow is a graph (``STKH_TOPOLOGY_AS_DATA``), and its documents list
   it as tables: instances, node types, bindings. Which node feeds which, and
   what each is given, is read by following names from one table to the
   next. Drawn as a graph, the shape is seen at once, and the detail is one
   selection away rather than all on the page.

   A connection is what joins two nodes: an output bound to a parameter,
   with the type of context it carries, or a branch a router may send the
   run along. Selecting a node shows its node type, its parameters, its
   output and what is bound to each; selecting a connection shows what it
   joins and what it carries. Whether a selection is a click or a pointer
   resting on the thing selected is the interface's to choose.

.. stkh_req:: A run is shown on its workflow's graph
   :id: STKH_RUN_SHOWN
   :stakeholder: user
   :statement: Agconflo shall show a person a run, while it is in progress and after it ends, on its workflow's graph with the contexts each connection carried and each node was given and made.

   Provenance is what this project is for (``STKH_PROVENANCE``), and today it
   is kept in a record a person reads as text: every context, and which
   activation was given it and which made it. Laid on the workflow's graph,
   the same record answers a question by looking: what went along this
   connection, what this node saw when it ran, on which pass. A node a model
   performs shows what the model was sent and what it answered, since both
   are contexts.

   A run is found in a list of its workflow's runs. One in progress is shown
   as it goes, so a person watching it sees each output as it is made. A run
   is shown on the definition it ran on, which is the one its record is a
   record of, whatever that workflow has become since.

.. stkh_req:: Watching a run is kept apart from editing its workflow
   :id: STKH_WATCHING_APART_FROM_EDITING
   :stakeholder: user
   :statement: Agconflo shall show a run in a view that cannot change the workflow it is a run of.

   A run is a record of what one definition did. A view of it that also
   edits the workflow invites a change made while reading the evidence, and
   then shows the edited definition beside a run of the old one: something
   that never happened. So a workflow's definition is edited in a view of
   its own (``STKH_WORKFLOW_EDITED``), and a run's view is for watching it.

   It does not stop a person acting on a run. Answering a step the run
   awaits changes the run, not its workflow (``STKH_RUN_FROM_THE_WEB``).

   It is separate from ``STKH_RUN_SHOWN`` because either can hold without
   the other: a run can be shown faithfully in a view that also edits, and a
   view that edits nothing can show a run poorly.

.. stkh_req:: Runs are started and answered in the web interface
   :id: STKH_RUN_FROM_THE_WEB
   :stakeholder: user
   :statement: Agconflo shall let a person start, answer and resume a run of a workflow through its web interface.

   The views show what runs did; this is where a person makes them go.
   Running a workflow means carrying the run to its end
   (``STKH_RUN_FROM_DOCUMENTS``): starting it with a context for each
   parameter nothing in it binds (``STKH_RUN_FROM_ANY_PARAMETER``), answering
   the steps a person performs with the contexts they ask for
   (``STKH_HUMAN_IN_RUN``), and resuming it after an interruption
   (``STKH_RESUMABLE_RUN``). Each is done beside the run's graph, where what
   a step was given can be read before it is answered.

   It is built on the views: a run started here is watched as
   ``STKH_RUN_SHOWN`` says.

.. stkh_req:: Workflows are created and edited in the web interface
   :id: STKH_WORKFLOW_EDITED
   :stakeholder: user
   :statement: Agconflo shall let a person create and edit a workflow through its web interface.

   A workflow is its documents: its topology, its node types and their
   scripts (``STKH_RUN_FROM_DOCUMENTS``). Edited as text, names are kept in
   step across tables and files by hand, and a broken edit is found when the
   workflow is next checked. Edited on its graph, a node is added or wired
   where it is seen, and what an edit broke is shown where it was made
   (``STKH_WIRING_CHECKED``).

   What is edited is the documents themselves, kept as data
   (``STKH_TOPOLOGY_AS_DATA``): a workflow edited here reads and runs as one
   written by hand, and what the editor does not understand in a document is
   kept. An agent changes the same documents through tool calls
   (``STKH_MACHINE_AUTHORING``); the two are ways to the same workflow, not
   two kinds of it.

   Editing has a view of its own, apart from watching runs
   (``STKH_WATCHING_APART_FROM_EDITING``).
