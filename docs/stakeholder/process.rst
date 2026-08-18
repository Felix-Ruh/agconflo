==================
Agconflo on itself
==================

Two goals about Agconflo's relationship with its own development, and they pull
against each other on purpose: the first is what makes the second worth stating.

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
