===================
Context and lineage
===================

The first feature: the immutable, addressable value a node consumes and produces,
and the record of what it was made from. Every requirement here derives from one
of the three context goals in ``stakeholder/context``, and each is written
against the decisions in ``decisions/context`` rather than re-opening them.

Each one was checked by hand against the question that no rule can ask: could
this be false while its parent is true? The body of each says how, because a
feature requirement that cannot fail independently of its parent is a
restatement, and nothing in the toolchain catches one.

The feature's architecture closes the file. It realises all five requirements
and names the components they are divided between, which are defined in
``components/context``.

.. feat_req:: A context is identifiable
   :id: FEAT_CONTEXT_IDENTITY
   :derived_from: STKH_PROVENANCE
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall give every context an identifier that no other context in the same run shares.

   Provenance recorded against an ambiguous name is not provenance. A run that
   reports a byte as coming from "the summary" while three contexts answer to
   that description has recorded something, and it has not recorded where the
   byte came from.

   So this can be false while its parent holds: every byte is attributed, and
   the attribution does not resolve to one value. Uniqueness within the run is
   what makes the recorded answer usable.

   ``DEC_IDENTITY_PER_ACTIVATION`` goes further than this requirement and says
   the identifier is fresh per activation, which is what keeps one pass's inputs
   apart from the same node's inputs on a later pass. This is the observable
   half. ``DEC_NO_CONTENT_ADDRESSING`` is the other boundary: the identifier is
   the whole of identity for now, so nothing may assume two contexts holding
   equal content are the same context.

.. feat_req:: A composed context keeps its parts
   :id: FEAT_CONTEXT_PARTS
   :derived_from: STKH_PROVENANCE
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall expose the parts of a composed context individually and in their composed order.

   ``DEC_COMPOSITION_BY_REFERENCE`` made observable, and the requirement the
   whole context model is built to keep.

   It can be false while its parent holds, and the way it can is the reason it
   is written down: a flattened string carrying a side table of byte ranges
   answers "where did this byte come from" perfectly well, and has thrown the
   parts away. Provenance survives; composition does not. Nothing further can
   then be built on the parts - not reordering for a provider's prompt cache,
   not a fuller fragment graph - because they no longer exist as values.

   Order is named because content is order-dependent. A set of parts cannot
   reproduce what was rendered, so a requirement that exposed the parts without
   their arrangement would leave the rendered bytes underivable from them.

.. feat_req:: Lineage reaches the whole ancestry
   :id: FEAT_CONTEXT_LINEAGE
   :derived_from: STKH_PROVENANCE
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall report every context in a composed context's ancestry, direct or indirect.

   Composition nests, so the parts of a composed context are frequently composed
   themselves. The question this project exists to answer is asked of the whole
   chain rather than of one level of it.

   It can be false while the requirement above holds: the direct parts are
   exposed, individually and in order, and the grandparents are unreachable
   because each part reports only what it was joined from. One level of an
   answer looks exactly like the answer until something is nested twice.

   Left deliberately unsaid here, because it belongs to the component level: how
   the traversal behaves on a graph that is not a tree. Two consumers sharing a
   part is ordinary rather than exceptional, so ancestry is a set reached by
   traversal and not a count of paths.

.. feat_req:: A context reads the same every time
   :id: FEAT_CONTEXT_STABLE_CONTENT
   :derived_from: STKH_IMMUTABLE_CONTEXT
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall return identical content for a context on every read of that context.

   The parent forbids altering a context. This is the property a test can
   actually assert, and the two come apart in a way that matters.

   It can be false while the parent holds, and this is the failure to expect
   rather than a contrived one. Nothing alters the stored context; the content
   is recomputed on each read, and something in the join is not deterministic -
   an iteration order over a map, a locale, a clock. The stored value was never
   touched and the second read returns different bytes. The parent's wording
   does not catch that, because nothing was altered.

   ``DEC_CONTEXT_API`` is what makes it a live risk rather than a theoretical
   one. Content is reached through a call rather than read from a field, so
   there is real computation behind every read, and the decision exists in order
   to keep lazy rendering addable later.

.. feat_req:: A context carries a readable type
   :id: FEAT_CONTEXT_DECLARED_TYPE
   :derived_from: STKH_EXPLICIT_CONTEXT
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall make a context's declared type readable by node code at run time.

   ``DEC_CONTEXT_TYPING``, and the half of it that is easy to lose. A node
   assembles its own inputs, interleaving them according to what each one is, so
   it has to be able to ask.

   It can be false while its parent holds, and this is the ordinary way to build
   it wrong: the wiring is checked before the run, the types are erased once
   they have served that purpose, and the node is handed exactly what was wired
   to it with no way to tell one input from another. The parent is satisfied in
   full. The node still cannot do its job.

   Checking the wiring itself is not this feature's work. That obligation
   belongs to ``STKH_WIRING_CHECKED`` and to a feature that has a workflow to
   validate, which this one does not.

.. feat_arch:: Context splits into a value, an identifier source and a walker
   :id: ARCH_CONTEXT
   :realises: FEAT_CONTEXT_IDENTITY, FEAT_CONTEXT_PARTS, FEAT_CONTEXT_LINEAGE, FEAT_CONTEXT_STABLE_CONTENT, FEAT_CONTEXT_DECLARED_TYPE
   :uses: COMP_CONTEXT_VALUE, COMP_IDENTIFIER_SOURCE, COMP_LINEAGE_WALKER
   :statement: Agconflo shall allocate context handling to the context value, the identifier source and the lineage walker.

   Three components, each answerable for requirements the other two cannot
   guarantee:

   - The context value answers for ``FEAT_CONTEXT_PARTS``,
     ``FEAT_CONTEXT_STABLE_CONTENT`` and ``FEAT_CONTEXT_DECLARED_TYPE``. All
     three are true or false of one value taken alone.
   - The identifier source answers for ``FEAT_CONTEXT_IDENTITY``. Uniqueness is
     a relation between contexts, and no value can guarantee it about itself.
   - The lineage walker answers for ``FEAT_CONTEXT_LINEAGE``. Ancestry is a
     traversal over many values that share parts, and it fails in ways that a
     single value does not.

   Folding the walker into the value was the obvious alternative, and it lost on
   allocation rather than on taste. The walker's component requirements - a
   shared ancestor reported once, nesting of any depth - have failure modes that
   belong to no single value, and a component requirement needs one subject that
   owns its behaviour.

   The decisions this is built against are named here rather than linked:

   - ``DEC_CONTEXT_API``: content is reached only through operations - the
     rendered content, the parts, the ancestry - and never through a public text
     field. That keeps rendering free to become lazy later, and it is why stable
     content is a requirement on a computation rather than on a stored string.
   - ``DEC_COMPOSITION_BY_REFERENCE``: a composed value holds ordered references
     to its parts and the join that combines them, never a flattened copy.
   - ``DEC_IDENTITY_PER_ACTIVATION`` is why identifiers come from a separate
     source: they are issued when an activation produces a context, and an
     activation belongs to a run rather than to any value. Until the engine
     exists, one source stands for one run.
   - ``DEC_NO_CONTENT_ADDRESSING``: the source generates identifiers and never
     derives one from content, and nothing compares content to decide identity.
   - ``DEC_CONTEXT_TYPING``: the declared type travels on the value, so it cannot
     be erased once the wiring has been checked.
   - ``DEC_METADATA_TRANSFORMED`` is recorded as an absence. No component here
     carries metadata, because it has no requirement to answer to until Transform
     nodes exist, and behind the API it can be added later without breaking any
     caller.
