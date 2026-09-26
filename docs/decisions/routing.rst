========================================
Decisions about a model choosing a route
========================================

How a node's script asks a model to choose, what reaches the model, what
comes back, and where the choice may be asked for. They are written for
``FEAT_ROUTE_CHOSEN_BY_MODEL`` against the measurements in
``evidence/routing``.

Five rest on those measurements, or on those ``decisions/models`` rested on.
The other three are judgements between the alternatives each names. One supersedes a decision in ``decisions/models``.

What they do not settle is named here: how a router takes the branch it
chose, which is the routing mechanism ``DEC_ONE_GRAPH`` describes; and the
score and yes-or-no questions the decisions model also answers.

.. dec:: A decision is a model call, recorded as an exchange offering nothing
   :id: DEC_DECISION_IS_A_MODEL_CALL
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_EXCHANGE_WITHOUT_OFFER_REPLAYS
   :statement: Agconflo shall treat a decision a script asks for as one of its model calls, recorded as an exchange whose window is the state and the questions, whose offer is empty and whose answer is the model's.

   A decision is something sent to a model by a role and an answer back, and
   everything the project requires of such a thing already holds of model
   calls: the role mapping, exactly the contexts sent, the answer as a
   context, the call limit, a failure carrying the role and the provider's
   answer, the record and the replay. The engine already records and replays
   an exchange that offered nothing and made no call
   (``EVD_EXCHANGE_WITHOUT_OFFER_REPLAYS``), so the core is unchanged.

   A kind of request of its own was the alternative: each of those
   requirements written again for it, and a second counter a script could
   spend beside the call limit.

.. dec:: A decision's questions are contexts in its window
   :id: DEC_QUESTIONS_ARE_CONTEXTS
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall make each question a script asks a model a context of the state's type, compose them after the state as the decision's window, and send each question as its rendering.

   A script writes its questions as strings, as it writes the text it gives
   ``host.text``, and the host makes each a context, as it makes the tools it
   offers from their declarations. Each question's rendering is the question
   as the decisions endpoint reads it: its type, its instructions and a
   record from each option to what it means.

   A model is shown exactly the contexts a call is made with
   (``FEAT_MODEL_WINDOW_IS_THE_PROMPT``), and a question is shown to it, so a
   question outside every context was ruled out: it would be text the record
   does not hold and replay does not compare. Asking the script to make each
   string a context itself was the other alternative, and gives the same
   window for more of the script's code.

.. dec:: A decision asks choice questions only
   :id: DEC_CHOICE_QUESTIONS_ONLY
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_DECISIONS_MODEL_SHAPE, EVD_DECISIONS_REPEAT
   :statement: Agconflo shall let a script ask a decisions model choice questions of two or more named options, and no other kind.

   The goal is choosing a branch, and a choice among named options is that:
   a yes-or-no is a choice of two. The decisions model also answers scores
   and yes-or-no values as numbers (``EVD_DECISIONS_MODEL_SHAPE``), and those
   moved between repeats where its choices did not (``EVD_DECISIONS_REPEAT``);
   they wait for a goal that needs a number.

.. dec:: A decision may be asked only where the instance declares no calls
   :id: DEC_DECIDING_WITHOUT_CALLS
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall let a script ask a model to choose only in an activation whose instance declares no calls.

   ``CREQ_HOST_OFFERS_DECLARED`` offers every model call the node types its
   instance declares, and a decisions model takes no tools. Where the
   instance declares none, a decision's empty offer is what every call is
   offered, and nothing is held back from the model or sent that it cannot
   use. A router makes no content of its own (``STKH_ROUTING``), so its
   instance has nothing to declare.

   Changing that requirement to let a decision offer nothing beside calls
   that offer tools was the alternative. It would be a change raised by this
   work rather than by the requirement's own parents, and a node that both
   calls node types and chooses a route is two steps, each a node of its own.

.. dec:: host.decide gives the answer as a context and each choice as a value
   :id: DEC_DECIDE_GIVES_ANSWER_AND_CHOICES
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall give a script asking for a decision the model's answer as a new context of a type the script must name, and a table from each question's name to the option chosen and its confidence.

   ``host.decide(role, state, questions, answer_type)`` returns the two. The
   table is what a router branches on - ``chosen.verdict.choice`` - and the
   context is where the model's bytes stay the model's
   (``FEAT_MODEL_ANSWER_IS_A_CONTEXT``). The type is required rather than
   defaulted, as a node's inputs are (``DEC_EVERY_INPUT_REQUIRED``).

   A table alone was the alternative, and leaves the answer with no
   provenance; a context alone makes every router parse it.

.. dec:: A role's entry names a decisions model with its endpoint and key
   :id: DEC_DECISIONS_ROLE_IN_THE_MAP
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_DECISIONS_MODEL_SHAPE
   :statement: Agconflo shall read a decisions model from a role's entry in the model mapping naming it under decisions, with an endpoint ending in a slash and a key variable, both required.

   One table of roles, whatever kind of model plays each, so a role is looked
   up in one place. The name goes under its own key because a chat model's
   is refused without a ``genai`` adapter's name before its double colon, and
   ``~typesafe/jev-latest`` has none. The endpoint is required because a
   decisions model has no provider ``genai`` knows a default for, and it ends
   in a slash because ``decisions`` is joined to it.

   A table of decisions models apart from the roles was the alternative: two
   places to look a role up, and a role that could be in both.

.. dec:: A chat model is reached through genai and a decisions model through its own endpoint
   :id: DEC_DECISIONS_THROUGH_THEIR_ENDPOINT
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supersedes: DEC_MODELS_THROUGH_GENAI
   :supported_by: EVD_GENAI_TWO_FORMATS_EXACT, EVD_GENAI_ERROR_STATUS, EVD_GENAI_BUILD_COST, EVD_DECISIONS_MODEL_SHAPE, EVD_REQWEST_THROUGH_GENAI
   :statement: Agconflo shall reach chat models through genai, and decisions models by posting to the decisions endpoint with the HTTP client genai already builds.

   Everything ``DEC_MODELS_THROUGH_GENAI`` rested on still holds for chat
   models, and they are still reached that way. A decisions model is not
   one: it answers only on its own endpoint, refusing the chat endpoint with
   400 (``EVD_DECISIONS_MODEL_SHAPE``), and ``genai`` speaks chat formats and
   nothing else. ``reqwest`` is already in the build as ``genai``'s
   dependency (``EVD_REQWEST_THROUGH_GENAI``), so a direct dependency on it
   adds no crate.

   Wrapping the decisions model as a chat model was the alternative, and it
   is not one: ``genai`` would send it a chat request, which it refuses.

.. dec:: A decisions model's answer is checked against the questions asked
   :id: DEC_DECISION_ANSWER_CHECKED
   :dec_status: accepted
   :decided_on: 2026-09-26
   :supported_by: EVD_DECISIONS_MODEL_SHAPE
   :statement: Agconflo shall fail a decision whose answer lacks a question asked or chooses an option that was not asked, naming the question.

   The endpoint is alpha, and its answer's shape is the day's
   (``EVD_DECISIONS_MODEL_SHAPE``). A router given an option nobody asked
   takes a branch that does not exist, and one given nothing branches on an
   absent value; either is a failure the script cannot see from the answer.
