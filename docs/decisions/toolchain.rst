==========================================
Decisions about the requirements toolchain
==========================================

How the requirements project itself is built and validated, and how code and
test results reach it. Two of these look like metamodel decisions rather than
tooling ones, and they are filed here because both were forced by measured tool
behaviour: the obligation moved out of the body because the tool cannot see a
body, and every rule needs a fixture because the tool ignores a malformed one.

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

.. dec:: Tests run under nextest
   :id: DEC_TESTS_UNDER_NEXTEST
   :dec_status: accepted
   :decided_on: 2026-09-19
   :supported_by: EVD_STABLE_NO_JUNIT, EVD_NEXTEST_TEST_PATHS, EVD_NEXTEST_TIMEOUT
   :statement: Agconflo's tests shall run under one pinned version of cargo-nextest in both the commit hook and continuous integration.

   The ascending half of the V needs every test's outcome in the graph, and a
   runner's report is where outcomes come from. Stable ``cargo test`` will not
   write one. nextest writes JUnit naming each test by its module path, which is
   what lets a test case's identifier be its test's path with no annotation to
   drift.

   Its time limit is part of the decision rather than a setting beside it. A
   defect that makes a test never finish would otherwise hang the run instead
   of failing it, and one test case here can only fail that way.

   Two costs are accepted. nextest does not run documentation tests
   (``EVD_NEXTEST_NO_DOCTESTS``), so a compile-time refusal is an ordinary test
   and no rustdoc example is written as a test. And the runner is one more
   pinned download, fetched and verified by ``scripts/get-nextest.sh`` as ubc is
   by its own script.

   Where nextest is not installed, the commit hook runs the tests under
   ``cargo test`` instead and says so, as it stands down for a missing ubc: the
   hook is a convenience and must not block a clone that has not run setup. CI
   has no such fallback, and CI is the authority.

.. dec:: Code is traced by one-line markers
   :id: DEC_IMPL_FROM_MARKERS
   :dec_status: accepted
   :decided_on: 2026-09-19
   :supported_by: EVD_NEED_ID_REFS_KEEP_FIRST, EVD_CODELINKS_CUSTOM_LINK
   :statement: Agconflo's requirements project shall produce every implementation need from a one-line marker in the Rust source.

   A one-line marker makes a need of its own for each place that meets a
   requirement, so a requirement met in two places has two implementations,
   each with its own location. The alternative, a reference in the source to a
   need written in a document, gives that need the first place alone and says
   nothing about the second. A trace that silently loses a place is worse than
   none, because it looks complete.

   The marker fills a link of its own, ``implements``, where ubc's default is
   ``links``: that one belongs to no level here, and a marker was measured
   able to fill another.

   Two costs are accepted. A marker is a comment, which the compiler never
   reads: it can stay behind when the code beneath it moves, and only review
   catches that. And its identifier is typed by hand to a convention - the
   module, then what the code does there - that nothing enforces beyond its
   prefix.

   A requirement with no marker naming it is not an error, since code is
   written after its requirement. Which ones have none is a review report,
   ``scripts/reports/unimplemented.cypher``, and never a gate: a gate there
   would refuse every commit from the one that writes a requirement to the one
   that writes its code.

.. dec:: Test results are committed, and verified rather than rewritten
   :id: DEC_RUNS_COMMITTED
   :dec_status: accepted
   :decided_on: 2026-09-19
   :supported_by: EVD_EXTERNAL_ZERO_NEEDS, EVD_EXTERNAL_MISSING_WARNS
   :statement: Agconflo's requirements project shall keep its test results in a file committed to the repository.

   Forced rather than preferred. A wired external needs file that is absent is
   reported, and every gate here checks at a level that makes that report fail
   the build, so results produced only when the project is checked would leave
   a fresh clone unable to check its documentation until it had built and run
   the tests. An empty file is legal, which is what let the import be wired
   before there was anything to import.

   The cost is a file that must be kept current, and it is paid by a gate that
   compares and never writes: ``scripts/import-test-runs.sh --check`` runs in
   the commit hook and in continuous integration, and a stale file fails both.
   Rewriting is a person's command, and the diff is read like the golden files
   are, for the same reason - a gate that repaired its own subject would report
   success over something nobody had looked at.

   What keeps this bearable is that the file changes only when an outcome or an
   identity changes, which is the decision below.

.. dec:: Only the latest run of each test case is kept
   :id: DEC_LATEST_RUN_ONLY
   :dec_status: accepted
   :decided_on: 2026-09-19
   :supported_by: EVD_NEXTEST_TEST_PATHS, EVD_NEXTEST_IGNORED_ABSENT
   :statement: Agconflo's requirements project shall record the latest run of each test case and no earlier one.

   The graph answers "does this requirement's test pass", which is a question
   about now. A history would answer it too, and at a price: a report carries a
   run identifier, a timestamp per test and a duration per test, all different
   between two runs of the same tests, so a file recording them would change on
   every commit and its diff would say nothing. Git already holds the history,
   with better tooling than a needs file could offer.

   So one run per test case, rewritten in place, holding only the case it ran
   and the outcome. The metamodel enforces the shape - a run executes at most
   one test case - and the importer emits nothing else.

   A test that did not run has no run at all rather than one saying so, because
   the runner writes nothing for a test it skipped. A renamed test is louder:
   its run names a test case that does not exist, and the check fails on that
   dead link until the case is renamed to match. A deleted test is quieter - its
   case simply has no run - which is a question for a query rather than an
   error, for the same reason coverage is never gated here.

.. dec:: Code names the decisions it follows in its marker
   :id: DEC_CODE_FOLLOWS_BY_MARKER
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_CODELINKS_SECOND_LIST, EVD_NEED_ID_REFS_KEEP_FIRST
   :statement: Agconflo's requirements project shall link code to each decision or note it follows by a list in that code's one-line marker.

   ``STKH_REASONS_IN_THE_GRAPH`` puts the reason for code in the graph, and a
   link is what makes the code findable from the reason. The link is
   ``follows``, the marker's fifth field, after ``implements``:
   ``// @<title>,<id>,<type>,[<requirements>],[<decisions or notes>]``. Measured,
   both lists fill their own links, ``[]`` leaves the first empty, a trailing
   list may be left out, and a dead id fails the check
   (``EVD_CODELINKS_SECOND_LIST``).

   A need-id reference was the alternative, and keeps only the first place that
   names a need (``EVD_NEED_ID_REFS_KEEP_FIRST``). An id written in a comment's
   prose was the state of things, and no query reaches it.

.. dec:: Code that implements no requirement is traced by a trace marker
   :id: DEC_TRACE_MARKERS
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_CODELINKS_SECOND_TYPE
   :statement: Agconflo's requirements project shall produce a trace need from a one-line marker for code that follows a decision or a note and implements no component requirement.

   Code shaped by a decision is not always where a requirement is met: a test
   helper, a parser's error path, a pinned constant. An implementation must
   implement a component requirement, and loosening that rule to a requirement
   or a decision would be a shape of rule never measured here. A second type
   leaves the implementation's rule as it is and gives the new one its own:
   ``trace``, prefix ``TRACE_``, following at least one decision or note.
   Measured, a marker can make a need of any declared type
   (``EVD_CODELINKS_SECOND_TYPE``); the name ``code`` collides with a built-in
   directive, which is why it is not the name.

.. dec:: An explanation local to code is a note in its crate's code document
   :id: DEC_NOTES_BESIDE_THE_CRATE
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_CODELINKS_MARKED_RST_INERT
   :statement: Agconflo's requirements project shall hold an explanation local to one piece of code as a note need in that crate's code document.

   Some code needs more explaining than a comment should hold, and the
   explanation belongs to that code alone: no decision chose it and no
   requirement asks for it. The place meant for it is a need written inside
   the comment, next to the code, and on the newest ubc such a need is never
   read (``EVD_CODELINKS_MARKED_RST_INERT``). So the note is written in
   ``docs/code/<crate>.rst``, the document that already brings the crate's code
   into the graph, and the code names it in the ``follows`` list of its marker.
   The note, its links and its body would move into the source unchanged once
   ubc reads needs there. Its directive is ``code_note``: ``note`` is
   reStructuredText's own admonition, and a need type registered under it lost
   its title in the metamodel's self-test.

   An explanation of how code relates to something else is not a note: it
   belongs to the need that owns the relation, a decision's or a requirement's
   body. A choice between alternatives is not a note either, but a decision.

.. dec:: Comments and docstrings say what the code does
   :id: DEC_COMMENTS_SAY_WHAT
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_SOURCE_COMMENTS_COUNTED
   :statement: Agconflo's comments and docstrings shall describe what its code does, and not why it does so or which needs it relates to.

   The split ``STKH_REASONS_IN_THE_GRAPH`` draws, applied to each comment. A
   docstring says what its item does, what it takes, what it gives back and how
   it fails, and may take several lines to; it says nothing of how the item
   does it, which is what the item hides, nor why, which is the graph's. A
   comment inside a body clarifies the code at that point, in a line or two.

   It is judged case by case, against the guidelines in ``AGENTS.md``, because
   whether a sentence is a what or a why is not something text shows. A length
   limit was the alternative, and the stakeholder turned it down: an argument
   list is long for a good reason.

.. dec:: The commit gate refuses a need id in a comment and a malformed marker
   :id: DEC_COMMENT_CHECKS
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_CODELINKS_COMMA_DROPS_MARKER, EVD_SOURCE_COMMENTS_COUNTED
   :statement: Agconflo's commit gate shall refuse a need id in a Rust comment that is not a code marker, and a code marker line that does not have the marker's shape.

   The two parts of ``DEC_COMMENTS_SAY_WHAT`` a machine can hold without a
   false alarm. A need id in a comment is a relation written as prose, which is
   what the marker's lists exist for, so it is never right. A marker that is
   not of the marker's shape is dropped by ubc without a word
   (``EVD_CODELINKS_COMMA_DROPS_MARKER``), so only a check reading the source
   itself can see it.

   The length of a comment is reported for review, not gated: whether a long
   block holds a reason is a question whose honest answer is sometimes no. Each
   crate is held once its comments have been brought to the decisions above,
   so the gate is green on every commit of the way there.

.. dec:: The test results file carries an empty version
   :id: DEC_RUNS_FILE_VERSION_EMPTY
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_EXTERNAL_VERSION_FREE
   :statement: Agconflo's test result importer shall write its needs file with an empty version.

   ``DEC_RUNS_COMMITTED`` commits the file and ``DEC_LATEST_RUN_ONLY`` keeps it
   changing only when an outcome does. A version following the project's would
   change it on every release whatever the tests did. ubc imports the needs
   under any version, the empty one included, and refuses a file with none
   (``EVD_EXTERNAL_VERSION_FREE``), so the version is written, and empty.

.. dec:: A report is imported whole or refused
   :id: DEC_IMPORT_WHOLE_OR_REFUSED
   :dec_status: accepted
   :decided_on: 2026-09-25
   :supported_by: EVD_NEXTEST_TEST_PATHS, EVD_NEXTEST_IGNORED_ABSENT
   :statement: Agconflo's test result importer shall import every run of the crates it is given from a report, or import none and say why.

   A file holding some of a report's runs reads exactly like one holding all of
   them, so an importer that skipped what it could not read would commit a
   file that looks complete and is not. Each way a report falls short is
   therefore a refusal of the whole of it, naming the test:

   - A test with no classname has no crate.
   - A test whose path is not ASCII letters, digits and underscores makes no
     id, such as a binary target's ``bin/<name>`` (``EVD_NEXTEST_IGNORED_ABSENT``)
     or a hyphenated file. An id built from it would fail the graph's id
     pattern naming only the id, where the refusal names the test.
   - Two tests making one id, a skipped test, and a retried one each leave
     which run counts undecided.
   - A crate named with no test in the report is refused, so a misspelt name
     cannot import an empty file that looks clean.

   Only the named crates' tests are read. nextest writes one report for the
   whole workspace, and a crate whose tests trace to no test case, the importer
   among them, would contribute runs of cases that do not exist. A crate name
   is matched whole, so ``agconflo-core-extra`` is not ``agconflo-core``, and a
   name holding ``::`` is a test binary's id rather than a crate
   (``EVD_NEXTEST_TEST_PATHS``). Taken for a crate, it would match that binary
   and drop the file from every id made from it: ``TEST_REACHES_EACH_ONCE`` for
   a test whose case is ``TEST_LINEAGE_REACHES_EACH_ONCE``, wrong and looking
   right. Matched as nothing, it is refused as a crate with no tests.
