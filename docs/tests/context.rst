==================
Context test cases
==================

How each Context requirement is to be checked. Results are never written here:
they are imported from the test runner, and each case records only what it
asserts.

Every failure mode listed in ``components/context`` has an error-path case
asserting which failure occurs and what holds afterwards. Every requirement whose
behaviour can be stated as an invariant has a property case. Degenerate inputs
the requirements say must still be accepted get a positive case naming them,
because a random generator reaches an embedded NUL or a lone CR too rarely to
count on. Where several cases verify one requirement, each covers it only in
part.

A case's id is meant to be the path of the Rust test that implements it,
uppercased, with ``::`` written as ``_``, so a result can be matched to its case
without an annotation that could drift. The paths here are ``<module>::<case>``
in ``agconflo-core``, with the modules ``id``, ``context`` and ``lineage``. The
test runner reports a unit test by its full module path, so these ids hold as
written only for tests that are bare test functions in those modules, not
inside a ``tests`` module - measured, and recorded as
``EVD_NEXTEST_TEST_PATHS``.

The compile-time refusals below must run as ordinary tests rather than as
documentation tests. The test runner does not run documentation tests
(``EVD_NEXTEST_NO_DOCTESTS``), and a case that never produces a result can never
be shown to pass.

.. test_case:: The source never issues an identifier twice
   :id: TEST_ID_NEVER_REPEATS
   :verifies: CREQ_SOURCE_NO_REPEAT
   :test_kind: property
   :coverage: partial

   For any number of requests up to ten thousand, every identifier one source
   issues differs from every other it has issued.

.. test_case:: An exhausted source stays exhausted
   :id: TEST_ID_EXHAUSTION_IS_PERMANENT
   :verifies: CREQ_SOURCE_NO_REPEAT
   :test_kind: error_path
   :coverage: partial

   A source started one identifier short of the end of its space issues that
   last identifier, then refuses the next request with the exhaustion failure
   rather than a generic one, and refuses every request after that in the same
   way. No identifier is issued after exhaustion, so none can repeat an earlier
   one.

   Needs a way to start a source near the end of its space that only tests can
   reach. Counting up to the end is not an option.

.. test_case:: A source cannot be copied
   :id: TEST_ID_SOURCE_CANNOT_BE_COPIED
   :verifies: CREQ_SOURCE_NO_REPEAT
   :test_kind: error_path
   :coverage: partial

   Code that copies or clones a source fails to compile, and the compiler's
   error names the missing capability rather than something incidental. A
   compile failure for any other reason would pass a naive version of this test,
   so the error itself is what is asserted - and the same lines without the
   copy must compile, so that the refusal is about the copy.

.. test_case:: An identifier cannot be forged
   :id: TEST_ID_CANNOT_BE_FORGED
   :verifies: CREQ_SOURCE_SOLE_ISSUER
   :test_kind: error_path
   :coverage: partial

   Code that builds an identifier other than by asking a source fails to
   compile, by each route a derive or an impl could open: the constructor, a
   default, a conversion from a number, and parsing. The constructor's refusal
   names it as private, and each other refusal names what is missing. As above,
   the error is asserted, not merely the failure, and an identifier obtained
   through a context must compile, so that each refusal is about its route.

   Partial since resuming a run's record became the requirement's one exception:
   that its identifiers come back only beside a source past them is
   ``TEST_RECORD_IDENTIFIERS_ONLY_WITH_A_SOURCE``.

.. test_case:: Text reads back as it was given
   :id: TEST_CONTEXT_TEXT_ROUND_TRIPS
   :verifies: CREQ_VALUE_TEXT_EXACT
   :test_kind: property
   :coverage: partial

   For any text, rendering a text context created from it returns the same
   bytes.

.. test_case:: Awkward text reads back as it was given
   :id: TEST_CONTEXT_TEXT_EDGE_CASES
   :verifies: CREQ_VALUE_TEXT_EXACT
   :test_kind: positive
   :coverage: partial

   Each input the requirement names comes back byte for byte: empty text; CRLF;
   a lone CR; leading and trailing whitespace; a letter followed by a combining
   accent, which Unicode normalisation would fold into one character; an
   embedded NUL.

.. test_case:: Composed content is its parts joined
   :id: TEST_CONTEXT_RENDER_IS_JOINED_PARTS
   :verifies: CREQ_VALUE_RENDER_JOINED
   :test_kind: property
   :coverage: partial

   For any tree of compositions, each with its own separator, the rendered
   content equals the parts' rendered content joined by that composition's
   separator, computed independently by the test from the texts it started with.

.. test_case:: Awkward compositions render as specified
   :id: TEST_CONTEXT_RENDER_EDGE_CASES
   :verifies: CREQ_VALUE_RENDER_JOINED
   :test_kind: positive
   :coverage: partial

   No parts renders as empty content; one part renders as that part with no
   separator; an empty separator concatenates; a part given twice renders twice.

.. test_case:: A deeply nested context renders
   :id: TEST_CONTEXT_DEEP_NESTING_RENDERS
   :verifies: CREQ_VALUE_RENDER_JOINED
   :test_kind: positive
   :coverage: partial

   A chain of compositions 100,000 levels deep renders to its expected content
   on a test thread with the default stack. That depth exhausts such a stack
   under recursion at one frame per level, so a recursive renderer fails this by
   aborting.

.. test_case:: A context reads the same twice
   :id: TEST_CONTEXT_READS_ARE_IDENTICAL
   :verifies: FEAT_CONTEXT_STABLE_CONTENT
   :test_kind: property
   :coverage: full

   For any context, text or composed, two reads return identical content.

   The one case written against a feature requirement directly. The cases above
   compare rendered content with what the test computes, which is a different
   assertion: this one catches a read that is correct on average and not twice
   in a row, the failure ``FEAT_CONTEXT_STABLE_CONTENT`` exists to rule out.

.. test_case:: Parts are the contexts that were given
   :id: TEST_CONTEXT_PARTS_ARE_ORIGINALS
   :verifies: CREQ_VALUE_PARTS_BY_REFERENCE
   :test_kind: property
   :coverage: partial

   For any list of contexts, text and composed alike, repeats and mixed types
   included, the parts of their composition carry exactly the given identifiers
   in the given order. Composed contexts must be among them, since the defect
   this rules out flattens a composed part into a copy of its text.
   Identifiers are unique, so an equal identifier is the original and not a
   copy.

.. test_case:: Awkward compositions keep their parts
   :id: TEST_CONTEXT_PARTS_EDGE_CASES
   :verifies: CREQ_VALUE_PARTS_BY_REFERENCE
   :test_kind: positive
   :coverage: partial

   No parts gives an empty list of parts; a context given twice appears at both
   positions and is itself both times; parts of different declared types
   compose.

.. test_case:: A deeply nested context is released
   :id: TEST_CONTEXT_DEEP_NESTING_DROPS
   :verifies: CREQ_VALUE_PARTS_BY_REFERENCE
   :test_kind: positive
   :coverage: partial

   A chain of compositions 100,000 levels deep is released on a test thread with
   the default stack. Releasing a chain one frame per level fails this by
   aborting, and it is the easiest of the three depth cases to get wrong, since
   the release is written by nobody unless someone writes it on purpose.

.. test_case:: The declared type is the one reported
   :id: TEST_CONTEXT_TYPE_IS_DECLARED
   :verifies: CREQ_VALUE_DECLARED_TYPE
   :test_kind: property
   :coverage: partial

   For any non-empty type name and any parts of any types, the composed context
   reports exactly the declared name. The generator must also produce parts
   whose names all differ from the declared one, since that is the case an
   inferred type gets wrong - and must not assume the names always differ,
   because composing two summaries into a summary is ordinary.

.. test_case:: An empty type name is refused
   :id: TEST_CONTEXT_EMPTY_TYPE_REFUSED
   :verifies: CREQ_VALUE_DECLARED_TYPE
   :test_kind: error_path
   :coverage: partial

   An empty type name is refused with the invalid-type-name failure rather than
   a generic one, and nothing carrying that name is created. A context created
   next with a valid name succeeds normally.

.. test_case:: Lineage reaches each ancestor once
   :id: TEST_LINEAGE_REACHES_EACH_ONCE
   :verifies: CREQ_WALKER_EACH_ONCE
   :test_kind: property
   :coverage: partial

   For any graph of compositions, shared parts included, a context's lineage is
   exactly the set of contexts reachable through its parts, computed
   independently by the test: no identifier is reported twice, and the context
   itself is absent. Compared as a set, since the order is not specified.

.. test_case:: Awkward ancestries are reported once
   :id: TEST_LINEAGE_EDGE_CASES
   :verifies: CREQ_WALKER_EACH_ONCE
   :test_kind: positive
   :coverage: partial

   A text context has no ancestors; a part given twice is reported once; an
   ancestor reachable along two paths is reported once.

.. test_case:: A stack of diamonds is walked in linear time
   :id: TEST_LINEAGE_DIAMONDS_ARE_LINEAR
   :verifies: CREQ_WALKER_EACH_ONCE
   :test_kind: positive
   :coverage: partial

   A stack of 64 diamonds - each level two contexts sharing the level below, and
   joined by one above - has two to the 64 paths to its bottom. Its lineage is
   reported as exactly 192 contexts within the test runner's time limit. A
   walker that follows paths instead of contexts fails by never finishing, so
   this case needs that limit set to fail it.

.. test_case:: A deeply nested context is walked
   :id: TEST_LINEAGE_DEEP_NESTING_WALKS
   :verifies: CREQ_WALKER_EACH_ONCE
   :test_kind: positive
   :coverage: partial

   The lineage of a chain 100,000 levels deep is reported, 100,000 contexts
   long, on a test thread with the default stack.
