==================
Agconflo on itself
==================

Four goals about Agconflo's relationship with its own development. The first
two pull against each other on purpose: the first is what makes the second worth
stating. The third is about where the reasons for its code are kept, and the
fourth about where an agent working on it looks first.

.. stkh_req:: The process that builds Agconflo runs on Agconflo
   :id: STKH_SELF_HOSTING
   :stakeholder: maintainer
   :statement: Agconflo shall run the development workflows derived from its own process description.

   Derived from, and that is the whole of it. "Use Agconflo to build Agconflo"
   would be satisfied by workflows written out by hand beside the process
   description, and would prove far less: the two would drift, and the
   description would go back to being decoration. The target is that the
   description is the source, which is what makes a process executable rather
   than aspirational.

   The shapes line up for it. A work product is a context, a role is a node type,
   a workflow is a graph, and the role answerable for an output is the node that
   produces it. A role definition naming its exact inputs and outputs is a
   context-wiring specification - this project's core thesis stated in process
   vocabulary rather than in engine vocabulary.

   One constraint follows and is not negotiable: v1 has to be usable before it can
   build itself, so the first pipeline runs from hand-written workflow data over a
   small set of nodes.

.. stkh_req:: Nothing Agconflo ships is privileged
   :id: STKH_NO_PRIVILEGED_TYPES
   :stakeholder: user
   :statement: Agconflo shall give a node type it ships no capability that a user-defined node type lacks.

   Agconflo is a general engine, so other people have to be able to model their
   own domains in it as well as this project models its own. Whatever it ships is
   a default or an example, never a vocabulary the engine knows about.

   The requirement above is exactly what makes this one worth writing down. The
   role, workflow and work-product shapes this project will ship for its own
   pipeline are precisely the ones it would be convenient to teach the engine
   about, and doing so would quietly turn a general engine into one that runs this
   project's process and approximates everyone else's. Self-hosting creates the
   pressure; this refuses it.

.. stkh_req:: The reasons for the code are kept in the graph
   :id: STKH_REASONS_IN_THE_GRAPH
   :stakeholder: maintainer
   :statement: Agconflo shall keep the reasons for its source code, and the relations between that code and its needs, in its requirements graph rather than in the code's comments.

   A reason written in a comment is invisible to every query this project can
   ask. Analysing a decision or a requirement should find all the code shaped by
   it, and a comment naming the decision in prose is found by nobody. The same
   comment also restates what the decision's own body says, so the two drift
   apart the first time either changes.

   What a comment or a docstring says is left to it: what the code does, what
   an item takes and gives back, how it fails. That is the split this goal
   draws - what the code is, in the code; why it is so and what it answers to,
   in the graph - and it is judged case by case rather than by a length.

   When it was written, 3,552 of the source's 19,371 lines were comments, and
   187 of the comments longer than a line named a need in prose
   (``EVD_SOURCE_COMMENTS_COUNTED``).

.. stkh_req:: An agent asks the requirements graph before the files
   :id: STKH_GRAPH_QUERIED_FIRST
   :stakeholder: maintainer
   :statement: Agconflo shall have an agent working on it answer a question its requirements graph can answer by querying that graph before searching the files the graph is built from.

   The graph is where what Agconflo holds about itself meets: every
   requirement, decision and measurement, the links between them, where the
   code meets each requirement, and how each test last ran. A question about
   any of it has one exact answer there, reached by following links. The files
   the graph is built from give the same answer only as text, where a link and
   a mention of an id in prose look alike, and a question across levels is
   joined by hand (``EVD_GRAPH_TELLS_LINK_FROM_MENTION``).

   A question is any about what the graph holds: reading a need, finding the
   needs that say something, and following what joins them.

   An agent working on Agconflo is any of them: one in a chat session beside
   the maintainer, and one that is a node of the development workflows
   Agconflo runs on itself once ``STKH_SELF_HOSTING`` holds. The goal is the
   same for both, so the self-hosted workflows give their agents the graph to
   ask, and not only the files to read.

   Before, and not instead of. What the graph does not hold - the body of a
   function, a file that is not a need - is the files' to answer, and a query
   answering nothing may be wrong rather than right, which the files can show.
   The goal names no tool: which query language and which command are a
   decision's to say, and may change.

   It is separate from ``STKH_REASONS_IN_THE_GRAPH`` because either can hold
   without the other. The reasons for the code can be kept in the graph and
   still be found by searching its files, and an agent can ask a graph that
   holds no reasons.
