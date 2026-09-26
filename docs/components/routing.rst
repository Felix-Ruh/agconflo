======================================
Components of a model choosing a route
======================================

The requirements ``ARCH_ROUTE_BY_MODEL`` allocates to the three components it
uses, each defined where its first feature put it: the model roster in
``components/models``, the script host in ``components/behaviour`` and the
model map in ``components/runner``. Each title is the grammatical subject of the
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
