======================================
Components of a model choosing a route
======================================

The requirements ``ARCH_ROUTE_BY_MODEL`` allocates to the three components it
uses, and after them those ``ARCH_ROUTING`` allocates to five. Each component
is defined where its first feature put it: the model roster in
``components/models``, the script host in ``components/behaviour``, the model
map in ``components/runner``, the topology reader in ``components/topology``,
the wiring validator in ``components/wiring``, the workflow run in
``components/run`` and the run record in ``components/resume``. Each title is the grammatical subject of the
requirements allocated to it, and the gate in ``scripts/gates`` refuses a
component requirement whose subject is anything else.

A decision is a model call (``DEC_DECISION_IS_A_MODEL_CALL``), so the
requirements every model call is held to hold of it and are not repeated:
``CREQ_ROSTER_ROLE_TO_MODEL``, ``CREQ_ROSTER_CONTEXTS_WHOLE``,
``CREQ_ROSTER_UNMAPPED_ROLE``, ``CREQ_ROSTER_PROVIDER_FAILURE``,
``CREQ_HOST_PROMPT_IS_A_CONTEXT``, ``CREQ_HOST_MODEL_CALL_LIMIT``,
``CREQ_HOST_MODEL_FAILURE``, ``CREQ_HOST_REPORTS_EXCHANGES``,
``CREQ_HOST_ANSWERS_FROM_RECORD`` and ``CREQ_HOST_REPLAY_DIVERGED``. A
decision's offer is empty, which ``CREQ_HOST_OFFERS_DECLARED`` allows only
where the instance declares no calls, and that is where a decision may be
asked for.

.. comp_req:: A script is given the option chosen for each question it asked
   :id: CREQ_HOST_GIVES_CHOICE
   :derived_from: FEAT_ROUTE_CHOSEN_BY_MODEL
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a script asks a model to choose, Script host shall give it the option chosen and its confidence for each question asked, with the answer as a new context of the type the script names.

   ``DEC_DECIDE_GIVES_ANSWER_AND_CHOICES``: the choices are values a script
   branches on, and the answer is the context the model's bytes stay in.

   Failure modes:

   - **The choice given as text to parse**, the shape the feature exists to
     end.
   - **One question's choice given for another's**, answers read by position
     where the model may give them in any order.
   - **The answer's type chosen by the host**, or left out, so that the
     model's bytes have no context of their own.

.. comp_req:: A decision asked with malformed questions fails before it is sent
   :id: CREQ_HOST_REFUSES_BAD_QUESTIONS
   :derived_from: FEAT_ROUTE_CHOSEN_BY_MODEL
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a script asks a model to choose with no question or with a question lacking instructions or a second option or holding a value that is not a string, then Script host shall fail that activation as a script error without sending a request.

   ``DEC_CHOICE_QUESTIONS_ONLY``: a question is a choice among named options,
   and one option is no choice.

   Failure modes:

   - **The request sent and refused by the provider**, spending a call on a
     question the host could see was malformed, and reporting it as the
     provider's failure rather than the script's.
   - **A question with one option answered**, a route that was never a choice.
   - **A number or a table taken as text**, and a question sent that the
     script did not write.

.. comp_req:: A decision asked where the instance declares calls fails before it is sent
   :id: CREQ_HOST_REFUSES_CHOICE_WITH_CALLS
   :derived_from: FEAT_ROUTE_CHOSEN_BY_MODEL
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If a script asks a model to choose in an activation whose instance declares calls, then Script host shall fail that activation as a script error without sending a request.

   ``DEC_DECIDING_WITHOUT_CALLS``. The feature gives a choice to a router's
   script, and a router's instance declares no calls; a decisions model is
   offered nothing, which ``CREQ_HOST_OFFERS_DECLARED`` allows only there.

   Failure modes:

   - **The decision sent without the declared offer**, an exchange whose
     offer differs from what every other call of the activation was offered.
   - **The declared node types sent to a decisions model**, which takes no
     tools and refuses the request, or ignores them.

.. comp_req:: A decision goes to its role's decisions endpoint as its contexts
   :id: CREQ_ROSTER_SENDS_DECISION
   :derived_from: FEAT_ROUTE_CHOSEN_BY_MODEL
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: event
   :statement: When a decision is sent for a role mapped to a decisions model, Model roster shall send it to the decisions path of that role's endpoint with the state's rendering and each question's rendering, whole.

   ``DEC_DECISIONS_THROUGH_THEIR_ENDPOINT``: the path is ``decisions`` joined
   to the endpoint the map gives, and the request is the model, the state and
   the questions (``EVD_DECISIONS_MODEL_SHAPE``). ``DEC_QUESTIONS_ARE_CONTEXTS``
   makes each question a context whose rendering is what is sent for it.

   Failure modes:

   - **The chat path used**, which a decisions model refuses with 400.
   - **A question rebuilt from its parts** rather than sent as its rendering,
     so that the window recorded is not what the model was sent.

.. comp_req:: A decision and a chat call each fail at a role mapped to the other kind
   :id: CREQ_ROSTER_REFUSES_KIND_MISMATCH
   :derived_from: FEAT_ROUTE_CHOSEN_BY_MODEL
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: unwanted
   :statement: If a decision names a role mapped to a chat model or a chat call names a role mapped to a decisions model, then Model roster shall fail that call naming the role without reaching any provider.

   Measured, each kind is refused on the other's endpoint
   (``EVD_DECISIONS_MODEL_SHAPE``, ``EVD_DECISIONS_REFUSALS``), so a mismatch
   sent is a call billed or refused for a fault the mapping already shows.

   Failure modes:

   - **The mismatch sent**, and the provider's 400 reported as though the
     model had failed.
   - **Failed without the role**, where a caller mapping several cannot tell
     which entry to fix.

.. comp_req:: A decisions model's answer that does not answer what was asked fails the call
   :id: CREQ_ROSTER_REFUSES_BAD_ANSWER
   :derived_from: FEAT_ROUTE_CHOSEN_BY_MODEL
   :allocated_to: COMP_MODEL_ROSTER
   :ears_pattern: unwanted
   :statement: If a decisions model's answer lacks a question that was asked or chooses an option that was not, then Model roster shall fail that call naming the question.

   ``DEC_DECISION_ANSWER_CHECKED``. The endpoint is alpha
   (``EVD_DECISIONS_MODEL_SHAPE``), and an answer of another shape is the
   first sign it has moved.

   Failure modes:

   - **An option not asked given to the script**, and a router taking a
     branch that does not exist.
   - **A missing question given as nothing**, and a script branching on an
     absent value.
   - **Failed without the question**, leaving the caller to find which of
     several went wrong.

.. comp_req:: A role mapped to a decisions model reaches it at its endpoint with its key
   :id: CREQ_MODEL_MAP_READS_DECISIONS
   :derived_from: FEAT_ROUTE_CHOSEN_BY_MODEL
   :allocated_to: COMP_MODEL_MAP
   :ears_pattern: event
   :statement: When a model mapping names a decisions model for a role, Model map shall send the role's decisions to that model at the endpoint the entry names, with the key from the variable it names.

   ``DEC_DECISIONS_ROLE_IN_THE_MAP``: a role's entry names ``decisions`` where
   a chat model's names ``model``, in the same table, so a role is looked up
   in one place whatever kind it is.

   Failure modes:

   - **The decisions model's name read as a chat model's**, and refused for
     lacking an adapter's name, or sent to the chat endpoint.
   - **The key taken from anywhere but the variable named.**

.. comp_req:: A decisions entry that is incomplete or also names a chat model is refused
   :id: CREQ_MODEL_MAP_REFUSES_BAD_DECISIONS_ENTRY
   :derived_from: FEAT_ROUTE_CHOSEN_BY_MODEL
   :allocated_to: COMP_MODEL_MAP
   :ears_pattern: unwanted
   :statement: If a model mapping's entry names both a model and a decisions model or names a decisions model without a key variable or without an endpoint ending in a slash, then Model map shall refuse the mapping at that entry.

   ``DEC_DECISIONS_ROLE_IN_THE_MAP``. A decisions model has no default
   endpoint, and a URL joined to an endpoint without its last slash loses the
   endpoint's last segment.

   Failure modes:

   - **Both names accepted**, and one of them used without a word.
   - **An endpoint without its slash accepted**, and every decision sent one
     segment short of the path.
   - **A missing key variable read as an empty key**, and the first decision
     refused by the provider as unauthorised.

.. comp_req:: A node type may declare that it routes
   :id: CREQ_READER_READS_ROUTES
   :derived_from: FEAT_ROUTE_WALKS_CHOSEN_EDGES
   :allocated_to: COMP_TOPOLOGY_READER
   :ears_pattern: event
   :statement: When a node type declares routes = true, Topology reader shall read that type as a router.

   ``DEC_ROUTER_DECLARED``. A value that is not a boolean is refused as any
   value of the wrong type is (``CREQ_READER_FAULT_LOCATED``).

   Failure modes:

   - **The key read past**, and a router's script refused for naming where
     its run goes.
   - **``routes = false`` read as a router.**

.. comp_req:: A binding may take a router's input by name
   :id: CREQ_READER_READS_ROUTED_INPUT
   :derived_from: FEAT_ROUTE_WALKS_CHOSEN_EDGES
   :allocated_to: COMP_TOPOLOGY_READER
   :ears_pattern: event
   :statement: When a binding is written as a table of from and input, Topology reader shall read it as the edge carrying the input so named of the instance so named.

   ``DEC_ROUTED_INPUT_BOUND_BY_TABLE``. A table with a key other than those
   two, or without one of them, is refused as a document that cannot be read
   (``CREQ_READER_FAULT_LOCATED``).

   Failure modes:

   - **The table read as the router's output**, and the branch given the
     router's decision where it was bound to the draft.
   - **``from`` and ``input`` swapped.**

.. comp_req:: A binding taking an input a node does not route is a defect
   :id: CREQ_VALIDATOR_ROUTED_INPUT
   :derived_from: FEAT_ROUTE_WALKS_CHOSEN_EDGES
   :allocated_to: COMP_WIRING_VALIDATOR
   :ears_pattern: unwanted
   :statement: If a binding takes an input of an instance whose node type does not route or does not declare that input, then Wiring validator shall report a defect naming that binding.

   A router passes on what it was given (``DEC_ROUTER_OUTPUT_IS_ITS_DECISION``),
   so an input taken from a node that is not one, or one its type does not
   declare, names an edge that will never carry anything. A context type
   disagreement on such a binding is compared against the input's declared
   type, as any wire's is.

   Failure modes:

   - **The binding passed**, and its consumer waiting for ever on an edge
     nothing walks.
   - **The input's type not compared**, and a note given where a diff is
     declared.

.. comp_req:: A router's script names where its run goes
   :id: CREQ_HOST_ROUTE_NAMED
   :derived_from: FEAT_ROUTE_WALKS_CHOSEN_EDGES
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: event
   :statement: When a router's script names the instances its run goes on to, Script host shall report those names with the activation's output.

   ``DEC_ROUTER_DECLARED``: through ``host.route``, with none or several
   names, once in an activation. Naming none is a router deciding its run goes
   nowhere from here.

   Failure modes:

   - **The names dropped**, and the router's edges walked as any node's are.
   - **The names reported in another order than named**, which a record
     compared on resuming would take for a different route.

.. comp_req:: A route named where none may be, or not named where one must be, fails
   :id: CREQ_HOST_REFUSES_ROUTE
   :derived_from: FEAT_ROUTE_WALKS_CHOSEN_EDGES
   :allocated_to: COMP_SCRIPT_HOST
   :ears_pattern: unwanted
   :statement: If the script of a node type that does not route names instances to go on to or a router's script names them twice or ends without naming them, then Script host shall fail that activation as a script error.

   Failure modes:

   - **A transform's script routing**, and a node that makes content deciding
     where it goes (``STKH_ONE_OUTPUT``).
   - **A router ending without a route read as every edge**, and a branch
     taken nobody chose.
   - **The second naming kept**, and a route a script meant to replace
     treated as its first.

.. comp_req:: A router's output goes along its edges into the instances named
   :id: CREQ_RUN_WALKS_ROUTED
   :derived_from: FEAT_ROUTE_WALKS_CHOSEN_EDGES
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: event
   :statement: When the caller reports a router's output with the instances named, Workflow run shall walk along each edge out of the router into those instances its output or the input the edge names and along no other edge.

   ``DEC_ROUTER_OUTPUT_IS_ITS_DECISION``. An input walked is the context the
   router's activation was given for it, by reference: the router makes none.

   Failure modes:

   - **Every edge walked**, and a branch not chosen run all the same.
   - **An input walked as a copy**, a new context whose bytes the router did
     not write (``STKH_PROVENANCE``).
   - **The input of another pass walked**, one the router was not given in
     this activation.

.. comp_req:: A route the run cannot take is refused
   :id: CREQ_RUN_REFUSES_BAD_ROUTE
   :derived_from: FEAT_ROUTE_WALKS_CHOSEN_EDGES
   :allocated_to: COMP_WORKFLOW_RUN
   :ears_pattern: unwanted
   :statement: If the caller reports a router's output without naming instances or names an instance no edge from it enters or names instances for a node that does not route, then Workflow run shall refuse that output.

   Failure modes:

   - **A name no edge enters ignored**, and a misspelt branch taken as no
     branch.
   - **A transform's output walked only where named**, a caller routing a
     node the workflow does not declare a router.

.. comp_req:: A router's naming is kept and resumed
   :id: CREQ_RECORD_HOLDS_ROUTES
   :derived_from: FEAT_ROUTE_RECORDED
   :allocated_to: COMP_RUN_RECORD
   :ears_pattern: ubiquitous
   :statement: Run record shall write each router's output with the instances its activation named and resume the run walking those names.

   ``DEC_ROUTE_RECORDED``: in the same entry as the output.

   Failure modes:

   - **The output resumed without its names**, and its edges walked as a
     transform's are.
   - **A record naming an instance the workflow no longer has resumed**,
     where resuming it would refuse as a divergence
     (``CREQ_RECORD_REFUSES_DIVERGENCE``).
