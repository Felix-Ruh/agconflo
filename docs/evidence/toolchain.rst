============================
Evidence about the toolchain
============================

Measurements the decisions beside these rest on, and the ones later decisions
will. Each says how it was obtained, so it can be re-run rather than believed,
and each is dated because evidence goes stale when the pin moves. The first four
date from the ubc 0.31.2b1 pin; every later one names the version it was taken
against.

.. evd:: ubc covers the documentation stack natively
   :id: EVD_UBC_REPLACES_STACK
   :evd_kind: vendor_doc
   :observed_on: 2026-08-16
   :observation: ubc provides the needs model, schema validation, code links and diagram rendering with no Python packages.

   Read from the tool's own documentation and changelog rather than run, which is
   why this is the weakest of the first four and labelled as such. It replaces the
   build engine, the needs model, schema validation, the code-link extractor and
   diagram rendering, the last of those without a Java runtime.

   One capability has no equivalent and the decision beside this one accepts it:
   there is nothing corresponding to the test-report importer, so bringing test
   results into the graph will need a small tool written here.

.. evd:: The formatter skips Markdown
   :id: EVD_FORMATTER_RST_ONLY
   :evd_kind: measurement
   :observed_on: 2026-08-18
   :observation: ubc format processed the six reStructuredText files of this project and skipped a Markdown file in the same source set.

   To re-run it: widen the source include pattern to cover Markdown as well, put a
   Markdown file beside the requirements, and run the formatter in check mode.

   Dated later than the decision it supports, deliberately. The markup choice was
   made on exactly this basis, but the method was never written down, so this is
   the first time it has actually been run here. An observation is dated when it
   was made rather than when it would be convenient for the arrow to point the
   other way.

.. evd:: A need's body is invisible to schema validation
   :id: EVD_CONTENT_INVISIBLE
   :evd_kind: measurement
   :observed_on: 2026-08-17
   :observation: An impossible pattern required of every decision's body matched nothing and reported nothing.

   Method: add a rule demanding a pattern that cannot occur in the body of every
   need of one type, then check the project. Six decisions, all of which have
   bodies, produced no violation whatsoever. The only sign that anything was wrong
   was a configuration note saying the property was not found in the field
   resolver.

.. evd:: A wrongly shaped rule is ignored rather than rejected
   :id: EVD_SILENT_RULE_DROP
   :evd_kind: measurement
   :observed_on: 2026-08-17
   :observation: A not keyword directly under validate.local reported nothing, where the same keyword inside allOf reported six violations.

   The sharpest of the first four, because it is an exact A and B. The rule forbade
   every decision from having an identifier beginning with its own prefix, which
   all six of them violate. Placed one way it found nothing; wrapped in a
   composition keyword it found all six.

   One wrapper is the entire difference between a rule that enforces and a rule
   that is decoration, and nothing reports the difference. This is the failure the
   fixture harness exists to catch.

.. evd:: Two misshapen rule forms still pass silently on 0.35.0
   :id: EVD_UBC035_RULE_SHAPES
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: On ubc 0.35.0 a misspelled keyword and a keyword of the wrong kind still pass silently, while a composite keyword under validate.local now fails the check.

   Taken when the pin moved from 0.31.2b1 to 0.35.0, as an A and B: the same
   probe rules against both binaries, each appended alone to the real rules and
   checked over the real project, with a control rule that fired on all twelve
   decisions under both versions.

   A misspelled keyword and a keyword of the wrong kind for the field - a
   minimum on a string - pass silently on both, exit 0 and nothing printed. A
   composite keyword placed directly under ``validate.local``, silent on
   0.31.2b1, is now reported as an unknown key that fails the check, and with it
   ubc stops applying every rule in the file: the control placed beside it fired
   on nothing. A rule about a need's body matches nothing on either version and
   is reported on both as a configuration warning that fails the check.

   That last result corrects the observation of ``EVD_CONTENT_INVISIBLE``, which
   says such a rule reported nothing. It reported no violation, and the body is
   still invisible to validation, so the decision resting on it stands - but the
   gate did not stay green.

   The fixture harness is still needed for the two shapes that pass silently,
   and for any rule of the right shape that matches the wrong thing.

.. evd:: nextest names a unit test by its full module path
   :id: EVD_NEXTEST_TEST_PATHS
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: cargo-nextest 0.9.145 names a unit test by its module path, including any tests module, and an integration test by its function alone, with the file in classname.

   Taken in a throwaway workspace shaped like the planned ``agconflo-core``,
   with JUnit output switched on in ``.config/nextest.toml``. The file lands at
   ``target/nextest/default/junit.xml``, one ``testsuite`` per test binary:

   - inside ``mod tests`` in ``src/id.rs``: classname ``agconflo-core``, name
     ``id::tests::never_repeats``;
   - a bare ``#[test]`` in ``src/context/mod.rs``: classname ``agconflo-core``,
     name ``context::text_round_trips``;
   - in ``tests/lineage.rs``: classname ``agconflo-core::lineage``, name
     ``reaches_each_once``.

   So the test case ids written for Context hold only if their tests are bare
   test functions in the ``id``, ``context`` and ``lineage`` modules, or if the
   importer drops a ``tests`` segment. A ``tests`` module is otherwise part of
   the path.

   Three attributes change on every run - the run's ``uuid``, and every
   ``timestamp`` and ``time`` - and a failure's ``message`` carries the thread
   number and a platform-specific source path. None of them can be recorded in a
   file that should change only when an outcome does.

   A deliberately failing property was shrunk and reported in the failure's text
   as ``minimal failing input: n = 10``, and its seed was written to
   ``proptest-regressions/context/render.txt`` beside the crate, mirroring the
   module path. The run exited 100.

   The download named ``windows-x86`` is a 32-bit build; ``windows`` is the
   64-bit one, and it is what this was taken with.

.. evd:: nextest does not run doctests
   :id: EVD_NEXTEST_NO_DOCTESTS
   :evd_kind: vendor_doc
   :observed_on: 2026-09-19
   :observation: The nextest documentation states that doctests are not supported by nextest and must be run separately with cargo test --doc.

   Read in the limitations section of its documentation rather than run, hence
   the weaker kind. It matters because a test that nextest never runs produces
   no result for the importer to read: a compile-time refusal written as a
   ``compile_fail`` doctest could never be shown to pass.

.. evd:: proptest adds seconds, not minutes, to a cold build
   :id: EVD_PROPTEST_COLD_BUILD
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: With proptest 1.11.0 in a one-crate workspace, a cold clippy took 4.8 s and a cold nextest build 6.9 s; after a one-line change, 0.2 s and 0.8 s.

   Taken on the development machine with cargo 1.96.0 and nextest 0.9.145,
   emptying the target directory before each cold run and fetching the
   dependencies beforehand so the network was not counted. The timer was checked
   against a one-second sleep, which read 1.0 s. A first attempt read 0.0 s for
   everything because the arithmetic tool it relied on is not installed here,
   and was discarded rather than reported.

   The pre-commit hook runs both, so its worst case with ``proptest`` in the tree
   is about twelve seconds.

.. evd:: A codelinks include glob is not relative to src_dir
   :id: EVD_CODELINKS_INCLUDE
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: On ubc 0.35.0 a codelinks include glob agconflo-core/**/*.rs matched no file under src_dir ../crates, while **/agconflo-core/src/**/*.rs matched exactly that crate.

   Taken with three marked Rust files under ``crates/``: one in
   ``agconflo-core/src``, one in the importer crate, one under a crate-local
   ``target/``. An empty ``include``, or ``**/*.rs``, collected all three - so a
   stray build directory and the importer would both be traced. A pattern
   written relative to ``src_dir``, or relative to the project, collected
   nothing, and nothing said so: the check stayed green with no code traced at
   all. Only a pattern beginning ``**/`` scoped correctly.

   A marker naming a requirement that does not exist was reported by the check
   run without a path argument, as ``needs.dead_link`` located twice: at the
   ``src-trace`` directive, and in the Rust file by an absolute Windows path.
   Naming an unrelated document as the path argument dropped it and exited 0.

.. evd:: A code marker can populate a custom link
   :id: EVD_CODELINKS_CUSTOM_LINK
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: On ubc 0.35.0 a oneline marker whose last field is declared as a custom list link produced an implements relationship to the comp_req it named.

   ``needs_fields`` declared as title, id, type and ``implements`` - a
   ``list[str]`` in the place of the default ``links``. The marker
   ``@Probe in core,IMPL_PROBE_CORE,impl,[CREQ_SOURCE_NO_REPEAT]`` became an
   ``impl`` need with an ``implements`` relationship to that component
   requirement, queryable in Cypher; the built-in ``links`` carried nothing.

.. evd:: A code location filled in by codelinks is visible to schema rules
   :id: EVD_CODELINKS_URL_VISIBLE
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: On ubc 0.35.0 an implementation produced from a code marker satisfied a rule requiring code_url while set_local_url filled it, and failed the same rule once set_local_url was off.

   Taken in a two-file project - one document and one Rust file, inside the
   unlicensed free tier - with ``local_url_field`` naming ``code_url``. A
   hand-written implementation with no ``code_url`` failed the rule under both
   settings, which shows the rule was live. A second rule requiring a field no
   implementation had fired on the code-derived one under both settings, which
   shows rules reach needs produced from code at all.

   It matters because a need's body is invisible to the same validation
   (``EVD_CONTENT_INVISIBLE``). Had a filled-in location been invisible too, a
   rule requiring it would have failed every real implementation.

.. evd:: A remote code URL pairs the checked-out commit with the working tree
   :id: EVD_CODELINKS_REMOTE_URL
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: On ubc 0.35.0 a remote_url_pattern of blob/{commit}/{path}#L{line} gave HEAD's full commit, the path from the git root and the marker's line in the working tree, so an uncommitted edit moved the line against an unchanged commit.

   Taken in a scratch git repository shaped like this one, with the URL
   revealed by a rule designed to fail and print it. A marker on line 3 of a
   committed file gave that file's path from the repository root and ``#L3``.
   After two lines were added above it without committing, the same commit was
   paired with ``#L5``, which on the host names a different line; a new,
   untracked file was given a link into a commit that does not contain it.

   In CI the checkout is the commit itself, so the two agree, and the value is
   recomputed at every index and never committed.

.. evd:: Traced Rust files do not count toward the unlicensed tier
   :id: EVD_CODELINKS_FREE_TIER
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: On ubc 0.35.0 a project with four documents and six traced Rust files checked without a licence, six documents with the same Rust files did not, and a project extending the configuration indexed no code needs without a src-trace directive.

   Taken outside this repository, where the open-source grant does not apply.
   The refusal named six files to index, which is the documents alone. The
   extending project was one fixture checked the way ``docs-selftest`` checks
   each of its own: it exited 0 and printed nothing, since only a src-trace
   directive brings code needs into a project.

.. evd:: Need-id references keep only the first call site
   :id: EVD_NEED_ID_REFS_KEEP_FIRST
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: On ubc 0.35.0 two need-id references to one requirement, in two Rust files, gave it the location of the first alone, with no diagnostic about the second.

   The location was written to the requirement's URL field and revealed by a
   rule designed to fail and print it. As a control, removing the first reference
   gave the second file's location, so the second had been read and dropped
   rather than never found.

   A one-line marker makes a need of its own for each place instead, so two
   places are two needs, each with its location.

.. evd:: An empty external needs file is legal and a missing one is not
   :id: EVD_EXTERNAL_ZERO_NEEDS
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: On ubc 0.35.0 an external needs file holding zero needs checked clean, and a missing one failed the check; a test_case imported from one linked into the graph.

   Wired as one ``[[needs.external_needs]]`` entry with a ``base_url``. The
   envelope is the one ``ubc build needs`` writes: ``current_version``, then
   ``versions``, then the needs keyed by id. With no needs in it, the check
   passed with nothing printed. With one ``test_case`` verifying a real
   component requirement, the case appeared in Cypher with its ``verifies``
   relationship across the boundary. With the file absent, the check failed
   with ``needs.external``, saying the source contributed no needs.

   So an import can be wired before the first test has run, but its file has to
   exist from then on.

.. evd:: Stable cargo test cannot write a JUnit report
   :id: EVD_STABLE_NO_JUNIT
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: On cargo 1.96.0 stable, cargo test refused the junit output format as accepted only on the nightly compiler with unstable options, and ran no test.

   Run in this workspace as ``cargo test --lib -- --format junit``. It exited
   101 before running anything, with the error that the format "is only
   accepted on the nightly compiler with -Z unstable-options". Test results
   reach the requirements graph through a report of this kind, so on stable
   Rust the built-in runner cannot supply them.

.. evd:: nextest fails a test that never finishes
   :id: EVD_NEXTEST_TIMEOUT
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: Under a slow-timeout of 10 s terminating after two periods, nextest 0.9.145 killed a test that could not finish at 20.0 s, exited 100, and recorded a failure of type test timeout in its JUnit report.

   Taken with the configuration in ``.config/nextest.toml``, after disabling the
   lineage walker's record of contexts already visited, so that the stack of 64
   diamonds is walked along every one of its two to the 64 paths. Without a
   limit that case never ends; the same walk run outside nextest was still
   running when it was killed after 45 s.

   With ``fail-fast`` off, the rest of the run was still reported: 16 passed and
   2 failed, the other two lineage cases catching the same defect as repeated
   ancestors.

.. evd:: nextest writes no test case for an ignored test
   :id: EVD_NEXTEST_IGNORED_ABSENT
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: cargo-nextest 0.9.145 counted an ignored test as skipped in its run summary and wrote no testcase element for it in the JUnit report, whose own skipped count was zero.

   Taken in a throwaway workspace whose one crate held a passing test, a failing
   one, an ignored one, a ``should_panic`` test that did not panic, a test
   inside a ``tests`` module, an integration test and a test in a binary
   target. Both failures were written as ``failure``; neither was an ``error``.

   So a run read from this report was never skipped: a test that did not run is
   absent from the report rather than recorded as not having run. An importer
   therefore has two outcomes to record, not three.

   The binary target's test was named with ``classname``
   ``<crate>::bin/<binary>``, which carries a slash - a character no identifier
   in this project may hold. The two naming rules of
   ``EVD_NEXTEST_TEST_PATHS`` held again: the test in the ``tests`` module kept
   that segment, and the integration test was named by its file.

.. evd:: The version in an external needs file is not checked
   :id: EVD_EXTERNAL_VERSION_FREE
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: On ubc 0.35.0 an external needs file imported the same need under a current_version of 0.0.0, 9.9.9 or the empty string, and a file with no current_version key failed the check.

   Wired as in ``EVD_EXTERNAL_ZERO_NEEDS``, with one ``test_case`` verifying a
   real component requirement, and this project's own version being 0.0.0. Under
   each of the first three the case appeared in Cypher with its ``verifies``
   relationship. Without the key the check failed with ``needs.external``,
   saying the file is not valid needs JSON and that the source contributed no
   needs.

   So a file written by a tool can carry one fixed version instead of following
   the project's, which would otherwise change the file on every release.
