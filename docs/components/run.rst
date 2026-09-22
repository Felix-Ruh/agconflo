================================
Components of running a workflow
================================

The two parts ``ARCH_RUN`` divides running a workflow into, and the requirements
allocated to each. A component is an object rather than a level: nothing derives
from it, and it exists so that a requirement has one subject answerable for it -
which is also what lets a requirement's grammatical subject be checked against
the title of the component it is allocated to.

Each title below is that subject, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

.. comp:: Workflow run
   :id: COMP_WORKFLOW_RUN
   :crate: agconflo-core

   One run of one workflow definition, from what it was started with to the one
   way it ended. It holds what has been produced, how many activations have been
   spent, and which activation is outstanding.

   Everything about a run's history belongs here: the two refusals that stop one
   before it starts, the count the budget is measured against, and each of the
   four endings. None of them can be decided by reading a definition, which is
   what separates this from the scheduler it asks.

   It performs no activation (``DEC_RUN_IS_DRIVEN``). It hands one out, takes
   back a context or a failure, and that is the whole of its contact with node
   behaviour.

.. comp:: Run scheduler
   :id: COMP_RUN_SCHEDULER
   :crate: agconflo-core

   The walk that answers, from a definition and the outputs produced so far,
   which instance may activate next and what that activation is given. It has no
   history in it: the same definition and the same produced outputs give the same
   answer, however the run reached them.

   Its requirements are about readiness and about what an activation carries,
   and they are true or false of that answer taken alone - which is what
   separates it from the run that keeps asking.

   This is where the one measured failure of the slice lives
   (``EVD_RUN_OPTIONAL_BY_ORDER``): a walk that asks readiness of the required
   parameters alone lets an instance activate before an optional parameter's
   source has produced, and which instance that is depends on the order the
   definition carries them in.
