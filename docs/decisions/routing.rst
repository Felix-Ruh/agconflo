========================================
Decisions about a model choosing a route
========================================

How a node's script asks a model to choose, what reaches the model, what
comes back, and where the choice may be asked for; then how a router takes
the branch it chose. The first eight are written for
``FEAT_ROUTE_CHOSEN_BY_MODEL`` against the measurements in
``evidence/routing``, the last four for ``FEAT_ROUTE_WALKS_CHOSEN_EDGES`` and
``FEAT_ROUTE_RECORDED`` on the model ``DEC_ONE_GRAPH`` recorded.

Five rest on those measurements, or on those ``decisions/models`` rested on.
The other seven are judgements between the alternatives each names. One
supersedes a decision in ``decisions/models``.

What they do not settle is named here: the score and yes-or-no questions the
decisions model also answers, and a stuck run's report telling an instance on
a branch not taken from one that cannot proceed.

The last, taken on 2026-10-04, makes a router's branches part of its
instance's declaration, so that the passes each branch runs on are known
before a run starts.

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

.. dec:: A node type routes by declaring it
   :id: DEC_ROUTER_DECLARED
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall treat a node type as a router only when its declaration says routes = true, and let only a router's script name the instances its run goes on to.

   Declared in the node type, a router is known before anything runs: the
   validator can check what is bound to its inputs, and a rendering of the
   graph can draw its edges as the branches they are. Letting any node's
   script route was the alternative, and leaves every edge in a workflow
   possibly not walked, which nothing but a run could tell.

   Nothing makes a router a type Agconflo ships (``STKH_NO_PRIVILEGED_TYPES``):
   any workflow's node type declares it.

.. dec:: A router's one output is its decision, and its inputs go on by name
   :id: DEC_ROUTER_OUTPUT_IS_ITS_DECISION
   :dec_status: superseded
   :decided_on: 2026-09-26
   :statement: Agconflo shall take a router's output as its decision, and walk along each of its edges either that output or the one of its inputs the edge names.

   ``STKH_ONE_OUTPUT`` keeps deciding apart from producing, and one output is
   one context with one identity. A router's output is therefore the context
   its decision is held in - a decisions model's answer, or text its script
   wrote - and not new content for what comes after. What the branch needs is
   what the router was given: a draft sent back to be revised is the draft,
   not a copy of it. So an edge out of a router carries its output, bound as
   any output is, or one of its inputs, bound by name
   (``DEC_ROUTED_INPUT_BOUND_BY_TABLE``).

   Bundling its inputs into one composed output was the alternative, and
   hands each consumer a list to take apart by position.

.. dec:: A binding takes a router's input by naming the router and the input in a table
   :id: DEC_ROUTED_INPUT_BOUND_BY_TABLE
   :dec_status: accepted
   :decided_on: 2026-09-26
   :statement: Agconflo shall read a binding written as a table of from and input as the edge carrying the named input of the named router, and a binding written as a name as the edge carrying that instance's output.

   ``draft = { from = "router", input = "draft" }`` rather than
   ``draft = "router.draft"``: an instance's name may hold a dot, so a dotted
   name cannot tell an instance called ``router.draft`` from the input
   ``draft`` of ``router``. Refusing dots in instance names was the other way,
   and would refuse workflows that are sound today for a syntax.

.. dec:: A router's naming is held in the record beside its output
   :id: DEC_ROUTE_RECORDED
   :dec_status: superseded
   :decided_on: 2026-09-26
   :statement: Agconflo shall write the instances a router's activation named beside that activation's output in the run's record, and resume a run walking the recorded names.

   Resuming takes a finished activation's output from the record
   (``CREQ_RECORD_CONTINUES_THE_RUN``) and runs no script again, so which
   edges a router walked has to be there too. Beside its output, in the same
   entry, because the two are one act of the router's and a record holding
   one without the other is not a record of it.

.. dec:: A router's instance declares its branches, and a route names one of them
   :id: DEC_ROUTER_BRANCHES_DECLARED
   :dec_status: accepted
   :decided_on: 2026-10-04
   :supported_by: EVD_RECORD_GOALS_NEEDS_BRANCHES
   :statement: Agconflo shall have a router's instance declare its branches, each a named set of the instances its edges enter, and refuse a route naming instances that are not one of those sets.

   Which instances a router names together decides which of them run on the
   same passes (``DEC_PASS_CLOCKS``), and a script naming them while the run
   goes says it too late for the run to be checked before it starts. Taken
   one instance at a time, every instance a router names would run on passes
   of its own, and a node reading two of them could be given no pass: the
   first development workflow is refused that way, and accepted with its
   router's two branches as its script names them
   (``EVD_RECORD_GOALS_NEEDS_BRANCHES``).

   They are declared in the workflow document, on the router's instance,
   because the instances a branch names are that workflow's:
   ``branches = { again = ["head", "judge"], done = ["approval"] }``. Every
   instance an edge out of the router enters is in a branch, a branch names
   no instance no edge out of it enters, and no two branches name the same
   instances, so a route picks out one branch. Naming nothing is not a
   branch, and stays allowed: the run goes nowhere from there.

   A route still names instances, as ``FEAT_ROUTE_WALKS_CHOSEN_EDGES`` and
   ``FEAT_PERSON_ROUTES`` have it, and the record keeps the names it always
   did; the run refuses a set that is not a declared branch, as it refuses a
   name no edge enters. Naming the branch instead, ``host.route('again')``,
   was the alternative: shorter to write, and a change to two requirements
   that are right for their own parents, to the record and to every script
   and answer that routes, for a name the run can find from the instances.

.. dec:: A router makes no context, and its edges carry what it was given
   :id: DEC_ROUTER_PASSES_ON_ITS_INPUTS
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supersedes: DEC_ROUTER_OUTPUT_IS_ITS_DECISION
   :statement: Agconflo shall take a router's activation as the instances it names and no context, and walk along each edge out of the router into an instance named the one of its inputs the edge names.

   ``STKH_ROUTING`` says a router "passes on the contexts it was given rather
   than making new ones", and ``DEC_ONE_GRAPH`` that "a router creates no
   content". ``DEC_ROUTER_OUTPUT_IS_ITS_DECISION`` made a router's output the
   context its decision is held in - "a decisions model's answer, or text its
   script wrote" - and that is a context a router made: the judge of the
   workflow proposed in #71 wrote its loop's whole state into it, which is
   producing content in a router. The maintainer's word of 2026-10-06 is the one the
   goal already says: a route gives no text, and routes its inputs onward.

   So a router's one output is the decision itself, the instances named
   (``STKH_ONE_OUTPUT``), and every edge out of it carries one of its inputs,
   bound by name (``DEC_ROUTED_INPUT_BOUND_BY_TABLE``). A context a router
   would have written - a verdict, a state, a note on why - is a node of its
   own before the router, whose output the router is given and passes on.
   What a decisions model answered stays where every model call's answer is,
   in the activation's exchange, and the script still reads its choices.

   Following from it: the run refuses an output reported for a router's
   activation; a router's node type declares no output
   (``DEC_ROUTER_DECLARES_NO_OUTPUT``); a workflow binding a router's output,
   designating a router as its output or declaring a call to a router is
   refused before it runs, since each takes from a router something it does
   not make; and a person performing a router answers with the route alone
   (``DEC_PERSON_ROUTE_IS_THE_ANSWER``).

.. dec:: A router's node type declares no output
   :id: DEC_ROUTER_DECLARES_NO_OUTPUT
   :dec_status: accepted
   :decided_on: 2026-10-06
   :statement: Agconflo shall refuse a node type document declaring an output for a node type that routes or none for any other node type.

   A declared output type says what a node of the type produces, and a router
   produces no context (``DEC_ROUTER_PASSES_ON_ITS_INPUTS``). Declared anyway,
   it would be a type nothing can ever be of, and a reader of the document
   would take the router for a node that writes one. Refused at reading, the
   mistake is found where it is written, with its line.

   Leaving it optional for a router and ignoring it was the other way, and
   keeps a declaration that says something untrue.

.. dec:: A router's naming is the whole of its entry in the record
   :id: DEC_ROUTE_RECORDED_ALONE
   :dec_status: accepted
   :decided_on: 2026-10-06
   :supersedes: DEC_ROUTE_RECORDED
   :statement: Agconflo shall write the instances a router's activation named as that activation's entry in the run's record, and resume a run walking the recorded names.

   What ``DEC_ROUTE_RECORDED`` decided, without the output it was written
   beside: a router's activation produces none
   (``DEC_ROUTER_PASSES_ON_ITS_INPUTS``). Resuming runs no script again
   (``CREQ_RECORD_CONTINUES_THE_RUN``), so the names are what a resumed run
   walks. The record's format changes with it, and its version with the
   format.
