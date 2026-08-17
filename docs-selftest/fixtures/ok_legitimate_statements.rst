=================================
Fixture: ok_legitimate_statements
=================================

.. The counterweight to every wording rule, and the fixture whose emptiness IS
   the assertion. Each statement below is one a regex is plausibly tempted to eat,
   and all of them must stay green.

   A rule that rejects good requirements is worse than no rule: it teaches an
   author to write round the checker rather than to write well, and there is no
   per-need waiver to escape with. So the near misses are kept here deliberately
   rather than being discovered later by someone whose correct requirement was
   refused.

   What each case guards:

   - a decimal, against a singular rule that banned every internal period
   - an apostrophe in the response, against an over-tight subject slot
   - a hyphenated subject, which is why the slot allows a hyphen at all
   - simplex, steadfast and atypically, against smell words matching inside a
     longer word
   - "state of the run", against the phrase "state of the art"
   - a correct complex and a correct unwanted statement, the two EARS grammars
     with two clauses, which are the easiest to get wrong in the rule rather than
     in the requirement

.. stkh_req:: A quantified statement with a decimal
   :id: STKH_OK_DECIMAL
   :stakeholder: user
   :statement: Agconflo shall render a run log within 0.5 seconds.

.. stkh_req:: An apostrophe in the response
   :id: STKH_OK_APOSTROPHE
   :stakeholder: user
   :statement: Agconflo shall bind a node's parameters before it runs.

.. stkh_req:: A hyphenated subject
   :id: STKH_OK_HYPHEN
   :stakeholder: maintainer
   :statement: Run-log writer shall append one record per activation.

.. stkh_req:: A smell word inside a longer word
   :id: STKH_OK_SUBSTRINGS
   :stakeholder: user
   :statement: Agconflo shall hold a steadfast simplex channel atypically open.

.. stkh_req:: A phrase that merely starts like a smell
   :id: STKH_OK_PHRASE
   :stakeholder: user
   :statement: Agconflo shall record the state of the run in its log.

.. feat_req:: A correct complex statement
   :id: FEAT_OK_COMPLEX
   :derived_from: STKH_OK_DECIMAL
   :ears_pattern: complex
   :verification_method: test
   :statement: While a loop is active, when an exit router fires, Agconflo shall end the iteration.

.. feat_req:: A correct unwanted statement
   :id: FEAT_OK_UNWANTED
   :derived_from: STKH_OK_DECIMAL
   :ears_pattern: unwanted
   :verification_method: test
   :statement: If a join receives contexts from two activations, then Agconflo shall withhold the join.
