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
``scripted`` and the model map's in ``model_map``. Every failure mode listed in
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
   one option, and with a number where an option's meaning goes. Each fails as
   a script error and the stub receives nothing.

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
