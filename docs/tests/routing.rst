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
the reader's in ``reader``, the writer's in ``writer``, the validator's in
``wiring``, the run's in ``run`` and the record's in ``record``, all in
``agconflo-core``. Every failure mode listed in
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

   ``input = { from = "router", input = "draft" }`` reads as the edge
   carrying ``router``'s input ``draft``, and ``hint = "writer"`` beside it as
   the edge carrying ``writer``'s output; a table missing ``input``, and one
   holding a key besides the two, are each refused at their place.

   Catches: the table read as an output of the instance it names; ``from``
   and ``input`` swapped.

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

.. test_case:: A router's node type declaring an output is refused
   :id: TEST_READER_ROUTER_OUTPUT_REFUSED
   :verifies: CREQ_READER_ROUTER_DECLARES_NO_OUTPUT
   :test_kind: error_path
   :coverage: full

   A node type declaring ``routes = true`` and an output is refused at the
   ``output`` key's line and column, its message saying a router makes none;
   one declaring ``routes = false`` and no output is refused for the missing
   key. The controls: a router declaring none reads with no output, and a
   node type that does not route reads the output it declares.

   Catches: the output read and ignored; a router read as needing an output;
   a node type that does not route read without one.

.. test_case:: Taking from a router anything but its inputs is a defect
   :id: TEST_WIRING_ROUTER_GIVES_ONLY_INPUTS
   :verifies: CREQ_VALIDATOR_ROUTER_GIVES_ONLY_INPUTS
   :test_kind: error_path
   :coverage: full

   One definition holding all three faults: a binding of a router's output,
   the router designated as the definition's output, and an instance
   declaring a call to the router's node type. All three are reported at
   once, each naming its place - the binding's instance and parameter, the
   call's instance and node type, the definition - and the binding's message
   says the router makes no output. The control: the router's input taken
   and another instance designated, no defect.

   Catches: a binding of a router's output passed; a router passed as the
   designated output; a call to a router passed; only the first such fault
   reported.

.. test_case:: A router's script names where its run goes
   :id: TEST_HOST_ROUTE_NAMED
   :verifies: CREQ_HOST_ROUTE_NAMED
   :test_kind: positive
   :coverage: full

   A router's script naming two instances and returning nothing, and one
   naming none: each activation is reported as the names in the order named,
   or none, and the run accepts it, which it would not with an output beside
   them. Named for both, both branches run, each given the input the router
   was given; named for none, the run goes nowhere from it.

   Catches: the names dropped; the names reported in another order; a
   context reported for the router all the same.

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

.. test_case:: A router's inputs go only into the instances named
   :id: TEST_RUN_ROUTED_WALKS_CHOSEN
   :verifies: CREQ_RUN_WALKS_ROUTED
   :test_kind: positive
   :coverage: full

   A router given a draft and a review, bound by two branches - one taking its
   inputs ``review`` and ``draft``, the other its input ``review`` - and
   routed to the first: that branch is given the very review and draft
   contexts the router was given, by identifier, and the other is given
   nothing. Routed to both, both are given theirs.

   Catches: every edge walked; an input walked as a copy; the input of another
   pass walked.

.. test_case:: A route the run cannot take is refused
   :id: TEST_RUN_BAD_ROUTE_REFUSED
   :verifies: CREQ_RUN_REFUSES_BAD_ROUTE
   :test_kind: error_path
   :coverage: full

   An output reported for a router, a route naming an instance no edge from
   the router enters, and a route reported for a transform: each is refused
   naming the instance, nothing is walked, and the same activation is still
   outstanding. A route naming nothing is taken, and the run goes nowhere.

   Catches: a context taken for a router; a name no edge enters ignored; a
   transform's output walked only where named.

.. test_case:: A router's naming is kept and resumed
   :id: TEST_RECORD_ROUTES_KEPT
   :verifies: CREQ_RECORD_HOLDS_ROUTES, CREQ_RECORD_REFUSES_DIVERGENCE
   :test_kind: positive
   :coverage: full

   A run whose router named one of two branches, written once it has: the
   router's entry holds its instance, its route and its inputs, and no
   output. Resumed, the run offers the branch named and not the other. A
   record whose router's entry holds an output beside its route is refused
   as unreadable, at the entry; one naming an instance the workflow no longer
   has is refused as diverging at the router's report; and one recording one
   activation spent for its output and its route is refused, counting two
   reports.

   Catches: the entry resumed without its names; an output written for the
   router; a record naming an instance the workflow no longer has resumed;
   a route left out of the reports the activations spent are held to.

.. test_case:: A router's instance declares its branches
   :id: TEST_READER_BRANCHES_READ
   :verifies: CREQ_READER_READS_BRANCHES
   :test_kind: positive
   :coverage: full

   A router's instance declaring three branches, one of them empty, written
   as an inline table and as a table of its own: each reads the branches in
   the order written, each with its instances in the order written. An
   instance declaring none has none. A ``branches`` that is an array, a branch
   that is a string, and a branch holding a number are each refused at the
   value.

   Catches: the key read past; a branch's instances read out of order or one
   dropped.

.. test_case:: A router's branches are written back as declared
   :id: TEST_WRITER_BRANCHES_WRITTEN
   :verifies: CREQ_WRITER_WRITES
   :test_kind: positive
   :coverage: partial

   A document whose router's branches are a table of their own, with a
   comment and its own spacing, written back unchanged is the same text. With
   the branches changed and a new router added, both are written as the
   definition declares them and read back as the definition; with none
   declared, the key goes.

.. test_case:: Branches that are not the branches after a router are reported
   :id: TEST_WIRING_BRANCHES_CHECKED
   :verifies: CREQ_VALIDATOR_BRANCHES
   :test_kind: error_path
   :coverage: full

   A router whose edges enter two instances, each through its input,
   declaring a branch for one, a branch for both and an empty branch: no
   defect. The same router with one entered instance in no
   branch, a branch naming an instance no edge enters twice, a second branch
   naming the same instances in another order, and branches on an instance
   that does not route: each reported once, naming the router and what is at
   fault, in the definition's order.

   Catches: an entered instance left in no branch passed; a branch naming an
   instance no edge enters passed; a repeated branch passed; branches on a
   node that does not route passed.

.. test_case:: A router's naming that is no branch is refused
   :id: TEST_RUN_ROUTE_NOT_A_BRANCH_REFUSED
   :verifies: CREQ_RUN_REFUSES_UNDECLARED_BRANCH
   :test_kind: error_path
   :coverage: full

   A router whose edges enter two instances, with branches for one and for
   both, naming the other alone: refused, naming what was named and both
   branches, the router still the activation outstanding. Then naming both
   in another order, one twice: accepted, and each is given its own.

   Catches: a set of instances that is no branch walked; a branch named in
   another order refused.

.. test_case:: A router naming no branch fails, and a person naming none answers again
   :id: TEST_SCRIPTED_ROUTE_NOT_A_BRANCH_REFUSED
   :verifies: CREQ_HOST_REFUSES_PERSON_ROUTE_NOT_A_BRANCH, FEAT_ROUTE_IS_A_DECLARED_BRANCH
   :test_kind: error_path
   :coverage: full

   The review loop of ``TEST_SCRIPTED_REVIEW_LOOP``, its router's branches
   sending the draft back or on. A router's script naming both instances
   fails the router's activation with the run's refusal. A person answering
   the router's step with both is refused, nothing run and no record handed
   over, and answering again with one branch is taken.

   Catches: the whole run failed for what the person can answer again; the
   answer taken as given.
