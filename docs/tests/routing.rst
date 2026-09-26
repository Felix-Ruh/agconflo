======================================
Test cases of a model choosing a route
======================================

How each requirement in ``components/routing`` is to be checked, and how the
requirements every model call is held to are checked for a decision. Results
are never written here: they are imported from the test runner.

No case reaches a provider. Each points the roster at a stub on the loopback
interface that answers in the shape the decisions endpoint was measured
answering (``EVD_DECISIONS_MODEL_SHAPE``, ``EVD_DECISIONS_REFUSALS``) and
records what it was sent. That the endpoint keeps that shape is outside what a
test here can show, and no key is needed or held.

A case's id is the path of the Rust test that implements it, uppercased: the
host's in the module ``host``, the roster's in ``models``, a resumed script's in
``scripted`` and the model map's in ``model_map``; and, for taking a branch,
the reader's in ``reader``, the validator's in ``wiring``, the run's in ``run``
and the record's in ``record``, all in ``agconflo-core``. Every failure mode listed in
``components/routing`` is named by the case that catches it.

.. test_case:: A script is given each question's choice and the answer as a context
   :id: TEST_HOST_DECISION_CHOSEN
   :verifies: CREQ_HOST_GIVES_CHOICE
   :test_kind: positive
   :coverage: full

   A script asking two questions of a stub that answers them in the reverse
   order. The table gives each question's option and confidence under its own
   name; the answer is a new context of the type the script named, holding the
   answers as the stub sent them.

   Catches: the choice given as text to parse; one question's choice given for
   another's; the answer's type chosen by the host.

.. test_case:: A decision with malformed questions fails and sends nothing
   :id: TEST_HOST_BAD_QUESTIONS_REFUSED
   :verifies: CREQ_HOST_REFUSES_BAD_QUESTIONS
   :test_kind: error_path
   :coverage: full

   Scripts asking with no question, with a question lacking instructions, with
   one option, with a number where an option's meaning goes, and with a key
   that is neither instructions nor options. Each fails as a script error
   naming its fault, and the stub receives nothing.

   Catches: the request sent and refused by the provider; a question with one
   option answered; a number taken as text.

.. test_case:: A decision asked where the instance declares calls fails and sends nothing
   :id: TEST_HOST_DECISION_WITH_CALLS_REFUSED
   :verifies: CREQ_HOST_REFUSES_CHOICE_WITH_CALLS
   :test_kind: error_path
   :coverage: full

   A script asking for a decision in an activation whose instance declares a
   call fails as a script error, and the stub receives nothing. The same
   script in an instance declaring none is answered: the control.

   Catches: the decision sent without the declared offer; the declared node
   types sent to a decisions model.

.. test_case:: Decisions count against the model call limit
   :id: TEST_HOST_DECISIONS_COUNTED
   :verifies: CREQ_HOST_MODEL_CALL_LIMIT
   :test_kind: error_path
   :coverage: partial

   Under a limit of two, a script making a chat call and two decisions fails as
   having exceeded its limit at the second decision, and the stub receives two
   requests.

   Catches: a decision a script could make without limit, spending beside the
   calls the limit bounds.

.. test_case:: A resumed decision is answered from its record, and a changed one diverges
   :id: TEST_SCRIPTED_DECISION_REPLAYED
   :verifies: CREQ_HOST_REPLAY_DIVERGED
   :test_kind: error_path
   :coverage: partial

   A run whose router decided, recorded and resumed: the decision is answered
   from the record and the stub receives nothing. Resumed with a script whose
   question's instructions differ by one word, it fails as diverged before any
   request.

   Catches: a question outside the window, which replay does not compare, and
   a resumed router given an answer to a question it did not ask.

.. test_case:: A decision reaches its endpoint's decisions path with its contexts whole
   :id: TEST_MODELS_DECISION_SENT_TO_ITS_ENDPOINT
   :verifies: CREQ_ROSTER_SENDS_DECISION
   :test_kind: positive
   :coverage: full

   A decision for a role mapped to a decisions model at the stub reaches the
   path ``decisions`` under the endpoint, carrying the model's name, the
   state's rendering as the state, and each question's rendering byte for byte
   under its name.

   Catches: the chat path used; a question rebuilt from its parts.

.. test_case:: A decision and a chat call each fail at a role mapped to the other kind
   :id: TEST_MODELS_KIND_MISMATCH_FAILS
   :verifies: CREQ_ROSTER_REFUSES_KIND_MISMATCH
   :test_kind: error_path
   :coverage: full

   A decision for a role mapped to a chat model, and a chat call for a role
   mapped to a decisions model: each fails naming its role, and the stub
   receives nothing.

   Catches: the mismatch sent; failed without the role.

.. test_case:: A decisions answer that does not answer what was asked fails the call
   :id: TEST_MODELS_BAD_DECISION_ANSWER_FAILS
   :verifies: CREQ_ROSTER_REFUSES_BAD_ANSWER
   :test_kind: error_path
   :coverage: full

   A stub answering one of two questions, and one choosing an option that was
   not asked: each call fails naming the question.

   Catches: an option not asked given to the script; a missing question given
   as nothing; failed without the question.

.. test_case:: A decisions refusal is carried with its role and status
   :id: TEST_MODELS_DECISION_REFUSAL_CARRIED
   :verifies: CREQ_ROSTER_PROVIDER_FAILURE
   :test_kind: error_path
   :coverage: partial

   A stub refusing a decision with 400 and a list of issues, 401, and 404 with
   a guardrail's reason, each as measured (``EVD_DECISIONS_REFUSALS``): each
   call fails carrying the role, the status and the message whole.

   Catches: a refusal from the decisions endpoint read as an answer, or its
   message cut to the status.

.. test_case:: A role mapped to a decisions model reaches it with its key
   :id: TEST_MODEL_MAP_DECISIONS_REACH_THEIR_MODEL
   :verifies: CREQ_MODEL_MAP_READS_DECISIONS
   :test_kind: positive
   :coverage: full

   A mapping giving one role a chat model and another a decisions model at the
   stub, each with its own key variable: a decision for the second reaches the
   stub's decisions path with that model's name and that key, and a chat call
   for the first reaches its own path.

   Catches: the decisions model's name read as a chat model's; the key taken
   from anywhere but the variable named.

.. test_case:: A decisions entry that is incomplete or also names a chat model is refused
   :id: TEST_MODEL_MAP_BAD_DECISIONS_ENTRY_REFUSED
   :verifies: CREQ_MODEL_MAP_REFUSES_BAD_DECISIONS_ENTRY
   :test_kind: error_path
   :coverage: full

   Entries naming both ``model`` and ``decisions``, a decisions model with no
   ``key_env``, with no endpoint, and with an endpoint lacking its last slash:
   each refuses the mapping at that entry's line.

   Catches: both names accepted; an endpoint without its slash accepted; a
   missing key variable read as an empty key.

.. test_case:: A node type's routes declaration is read
   :id: TEST_READER_ROUTES_READ
   :verifies: CREQ_READER_READS_ROUTES
   :test_kind: positive
   :coverage: full

   Types declaring ``routes = true``, ``routes = false`` and neither read as a
   router, not a router and not a router; ``routes = 1`` is refused at its
   value.

   Catches: the key read past; ``routes = false`` read as a router.

.. test_case:: A binding written as a table takes a router's input
   :id: TEST_READER_ROUTED_INPUT_READ
   :verifies: CREQ_READER_READS_ROUTED_INPUT
   :test_kind: positive
   :coverage: full

   ``draft = { from = "router", input = "draft" }`` reads as the edge carrying
   ``router``'s input ``draft``, and ``verdict = "router"`` beside it as the
   edge carrying its output; a table missing ``input``, and one holding a key
   besides the two, are each refused at their place.

   Catches: the table read as the router's output; ``from`` and ``input``
   swapped.

.. test_case:: A binding taking an input a node does not route is reported
   :id: TEST_WIRING_ROUTED_INPUT_CHECKED
   :verifies: CREQ_VALIDATOR_ROUTED_INPUT
   :test_kind: error_path
   :coverage: full

   Bindings taking an input of a node that does not route, and an input a
   router does not declare, are each reported naming the binding; one taking
   a ``note`` input into a ``diff`` parameter is reported as a disagreement;
   one taking a declared input of a router into a parameter of its type passes.

   Catches: the binding passed; the input's type not compared.

.. test_case:: A router's script names where its run goes
   :id: TEST_HOST_ROUTE_NAMED
   :verifies: CREQ_HOST_ROUTE_NAMED
   :test_kind: positive
   :coverage: full

   A router's script naming two instances, and one naming none: each
   activation's output is reported with those names in the order named, and
   with none.

   Catches: the names dropped; the names reported in another order.

.. test_case:: A route named where none may be, or not named where one must be, fails
   :id: TEST_HOST_ROUTE_REFUSED
   :verifies: CREQ_HOST_REFUSES_ROUTE
   :test_kind: error_path
   :coverage: full

   A script of a node type that does not route calling ``host.route``; a
   router's script calling it twice; and one returning without calling it:
   each fails as a script error, and nothing is reported to the run.

   Catches: a transform's script routing; a router ending without a route read
   as every edge; the second naming kept.

.. test_case:: A router's output goes only into the instances named
   :id: TEST_RUN_ROUTED_WALKS_CHOSEN
   :verifies: CREQ_RUN_WALKS_ROUTED
   :test_kind: positive
   :coverage: full

   A router given a draft and a review, bound by two branches - one taking its
   output and its input ``draft``, the other its input ``review`` - and named
   for the first: that branch is given the router's output and the very draft
   context the router was given, by identifier, and the other is given
   nothing. Named for both, both are given theirs.

   Catches: every edge walked; an input walked as a copy; the input of another
   pass walked.

.. test_case:: A route the run cannot take is refused
   :id: TEST_RUN_BAD_ROUTE_REFUSED
   :verifies: CREQ_RUN_REFUSES_BAD_ROUTE
   :test_kind: error_path
   :coverage: full

   A router's output reported without names, one naming an instance no edge
   from it enters, and a transform's output reported with names: each is
   refused, and nothing is walked.

   Catches: a name no edge enters ignored; a transform's output walked only
   where named.

.. test_case:: A router's naming is kept and resumed
   :id: TEST_RECORD_ROUTES_KEPT
   :verifies: CREQ_RECORD_HOLDS_ROUTES
   :test_kind: positive
   :coverage: full

   A run whose router named one of two branches, written after the router's
   output and resumed: the resumed run offers the branch named and not the
   other, and a record naming an instance the workflow no longer has is
   refused as diverging.

   Catches: the output resumed without its names; a record naming an instance
   the workflow no longer has resumed.
