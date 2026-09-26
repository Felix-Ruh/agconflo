===============================
A model choosing a node's route
===============================

A node's script asking a model to choose among named options, and being given
the option chosen. It derives from ``STKH_ROUTING``, and is written against
the decisions in ``decisions/routing``.

What it does not do is what keeps this a slice. It gives a script a model's
choice; how a router then takes the branch it chose is the routing mechanism
``DEC_ONE_GRAPH`` describes, and is not specified here. A choice is the one
kind of question: a score or a yes-or-no, which the decisions model measured
also answers (``EVD_DECISIONS_MODEL_SHAPE``), waits for a goal that needs one.

A decision is a model call (``DEC_DECISION_IS_A_MODEL_CALL``), so what is
already required of every model call a script makes holds of it unchanged:
the model its role is mapped to answers it, it is shown exactly the contexts it
is made with, its answer is a context, it counts against the call limit, a
failure ends the activation carrying the role and the provider's answer, and
the run's record holds it. What is added is the choice itself.

The requirement was checked by hand against the two questions every body here
answers: could it be false while its parent holds, and could the parent hold
while it is false? The statement is ``event``. The feature's architecture
closes the file.

.. feat_req:: A router's script is given the option a model chose
   :id: FEAT_ROUTE_CHOSEN_BY_MODEL
   :derived_from: STKH_ROUTING
   :ears_pattern: event
   :verification_method: test
   :statement: When the script of a node whose instance declares no calls asks a model to choose among named options, Agconflo shall give the script the option the model chose.

   The parent lets a node decide which branch a run takes, and names a model
   among what may make that decision. A model asked in words answers in words,
   and a script then reads a branch out of prose. Given the option as a value,
   a router branches on what the model chose rather than on what a script made
   of its text.

   It can be false while the parent holds. A host offering only
   ``host.complete`` meets the parent - a node can still decide, from a
   model's text - and leaves every model-made route resting on a parse.

   It claims no more than the parent. The node is the parent's router, which
   "passes on the contexts it was given rather than making new ones": an
   instance declaring calls has its model call node types, whose outputs are
   new contexts, and is a transform, not a router. Which options there are,
   and what they mean, is the script's to say.

.. feat_arch:: A model's choice splits between the model roster, the script host and the model map
   :id: ARCH_ROUTE_BY_MODEL
   :realises: FEAT_ROUTE_CHOSEN_BY_MODEL
   :uses: COMP_MODEL_ROSTER, COMP_SCRIPT_HOST, COMP_MODEL_MAP
   :statement: Agconflo shall allocate a model's choice to the model roster, the script host and the model map.

   Each answers for what the others cannot:

   - The model map answers for which decisions model a role reaches, read
     from the person's mapping, as it does for chat models.
   - The model roster answers for a decision on the wire: where it is sent,
     that it reaches a decisions model and no chat model, and that the answer
     chose among the options asked.
   - The script host answers for what a script may ask and what it is given:
     the questions it may put, where it may put them, and the choice and the
     answer it gets back.

   The decisions it is built against are named here rather than linked:
   ``DEC_DECISION_IS_A_MODEL_CALL``, ``DEC_QUESTIONS_ARE_CONTEXTS``,
   ``DEC_CHOICE_QUESTIONS_ONLY``, ``DEC_DECIDING_WITHOUT_CALLS``,
   ``DEC_DECIDE_GIVES_ANSWER_AND_CHOICES``, ``DEC_DECISIONS_ROLE_IN_THE_MAP``,
   ``DEC_DECISIONS_THROUGH_THEIR_ENDPOINT`` and ``DEC_DECISION_ANSWER_CHECKED``.
