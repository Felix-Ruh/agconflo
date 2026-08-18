=============================
Fixture: rule_statement_ascii
=============================

.. Statements carrying a character that is not plain ASCII. Every case here reads
   as correct on screen, which is the whole reason the rule exists: a lookalike
   defeats the anchored grammar rules while leaving nothing visible to fix.

   What each case guards:

   - a non-breaking space before the response, the worst of them. The statement
     below is a valid ubiquitous requirement in every respect a reader can see,
     and the grammar rejects it. Without this rule the only diagnostic is one
     that shows a correct-looking sentence failing a pattern it appears to match.
   - a typographic apostrophe in the subject, the same trap one step milder. The
     subject slot admits the ASCII apostrophe, so the message would otherwise
     show a pattern that visibly contains an apostrophe rejecting a statement
     that visibly contains an apostrophe.
   - a typographic apostrophe in the RESPONSE, which no grammar rule touches at
     all - the response slot is `.+`, so this one fires this rule and nothing
     else. It is what proves the rule reaches beyond the subject.

.. stkh_req:: A non-breaking space before the response
   :id: STKH_ASCII_NBSP
   :stakeholder: user
   :statement: Agconflo shall record every context that a node consumed.

.. stkh_req:: A typographic apostrophe in the subject
   :id: STKH_ASCII_SUBJECT
   :stakeholder: user
   :statement: Agconflo’s run log shall record every context that a node consumed.

.. stkh_req:: A typographic apostrophe in the response
   :id: STKH_ASCII_RESPONSE
   :stakeholder: user
   :statement: Agconflo shall record every context that a node’s parameters bound.
