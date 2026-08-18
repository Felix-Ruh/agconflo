==========================================
Decisions about the requirements toolchain
==========================================

How the requirements project itself is built and validated. Two of these look
like metamodel decisions rather than tooling ones, and they are filed here
because both were forced by measured tool behaviour: the obligation moved out of
the body because the tool cannot see a body, and every rule needs a fixture
because the tool ignores a malformed one.

.. dec:: The toolchain is ubc alone
   :id: DEC_NO_PYTHON
   :dec_status: accepted
   :decided_on: 2026-08-16
   :supported_by: EVD_UBC_REPLACES_STACK
   :statement: Agconflo's requirements project shall be built with ubc alone and no Python toolchain.

   The configuration format is sphinx-needs' own, so nothing is invented and the
   data stays portable. That is the whole mitigation for depending on a single
   closed-source pre-release tool for the entire process layer: re-adding the
   Python stack later would be an install rather than a rewrite.

   The known cost is accepted rather than hidden. There is no equivalent of the
   test-report importer, so importing test results will mean writing one.

.. dec:: Requirements are authored in reStructuredText
   :id: DEC_MARKUP_RST
   :dec_status: accepted
   :decided_on: 2026-08-16
   :supported_by: EVD_FORMATTER_RST_ONLY
   :statement: Agconflo's requirements project shall be authored in reStructuredText.

   The formatter and the RST linter are a gate that Markdown would forfeit
   entirely. The cost is that these documents are more verbose to write than the
   equivalent Markdown, which is a real cost and a small one.

   Not one-way: mixed projects are supported, so a later Markdown section would be
   a change rather than a migration.

.. dec:: The obligation lived in the need's body
   :id: DEC_OBLIGATION_IN_CONTENT
   :dec_status: superseded
   :decided_on: 2026-08-14
   :statement: Agconflo's requirements project shall carry a requirement's obligation in the need's body.

   Kept rather than deleted, and marked rather than edited into its replacement. It
   was a reasonable design: the body is a field anyway, it is the main attraction
   of a need, and it is where a reader looks first.

   What it did not survive was measurement. It is recorded here so that anyone
   arriving at the same reasonable idea finds out that it was tried.

.. dec:: The obligation lives in a declared field
   :id: DEC_OBLIGATION_IN_STATEMENT
   :dec_status: accepted
   :decided_on: 2026-08-17
   :supported_by: EVD_CONTENT_INVISIBLE
   :supersedes: DEC_OBLIGATION_IN_CONTENT
   :statement: Agconflo's requirements project shall carry a requirement's obligation in a declared statement field.

   Forced rather than chosen, which is why the decision it replaces is worth
   keeping. A body cannot be validated at all, so every wording rule written
   against one would have matched nothing while appearing to work - the worst
   available outcome, since the checks would have looked green.

   The body keeps a real job: rationale and derivation, the prose a person writes
   around a requirement, and it remains readable from a query. Only the obligation
   moved.

.. dec:: Every rule is guarded by a fixture
   :id: DEC_FIXTURE_PER_RULE
   :dec_status: accepted
   :decided_on: 2026-08-17
   :supported_by: EVD_SILENT_RULE_DROP
   :statement: Agconflo's requirements project shall guard every validation rule with a fixture that fails without it.

   The most load-bearing decision in this file. A rule that has never been seen to
   fail cannot be assumed to run, because a malformed rule is ignored rather than
   rejected and the project goes green either way.

   So every rule gets a document that violates it and a golden file recording the
   exact diagnostics that violation must produce, and the harness refuses to pass
   a rule that no golden file mentions. The cost is real and worth naming: changing
   a rule means re-blessing golden files and reading the resulting diff carefully,
   because blessing without reading turns a broken rule into an expectation.
