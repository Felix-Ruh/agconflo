==================
Context components
==================

The components the Context feature decomposes into, all in ``agconflo-core``.

A component is an object rather than a level: nothing is derived from it, and
component requirements are allocated to it. Each title is written to be used as
the grammatical subject of those requirements, which is why none of them carries
an article.

The requirements allocated to each component follow the components. Every one
lists its failure modes in its body - what is reported, and what holds
afterwards - because the error-path test cases are derived from that list, and a
failure mode nobody wrote down is one nobody tests. Where an operation cannot
fail, the body says so and names the degenerate inputs it must still accept:
being infallible is a design choice, and it gets asserted like any other.

.. comp:: Context value
   :id: COMP_CONTEXT_VALUE
   :crate: agconflo-core

   The immutable value itself: its identifier, its declared type, and its
   content, which is either text or an ordered list of references to other
   values together with the join that combines them.

   It owns everything that is true of one value taken alone - that its content
   reads the same every time, that its parts stay addressable and in order, that
   its type can be asked for. It does not issue its own identifier and it does
   not walk its own ancestry, because both of those are questions about other
   values.

.. comp:: Identifier source
   :id: COMP_IDENTIFIER_SOURCE
   :crate: agconflo-core

   Issues the identifier each new context carries, and is the only thing that
   does. One source spans one run, so uniqueness within a run is a property of
   the source rather than a hope about the values.

   Kept apart from the value because uniqueness is a relation between contexts.
   A value can hold an identifier; it cannot know that no other value in the run
   holds the same one. Keeping issuance in one place is also what lets the
   scheduler take it over once activations exist, without the value changing.

.. comp:: Lineage walker
   :id: COMP_LINEAGE_WALKER
   :crate: agconflo-core

   Reports the ancestry of a context: every value reachable through its parts,
   at any depth, each one once.

   Kept apart from the value because ancestry is a traversal over many values,
   and it fails in ways no single value does. Parts are shared, so one ancestor
   can be reached along several paths and must still be reported once; nesting
   has no fixed depth, so the traversal cannot assume a shallow one.

.. comp_req:: The source never repeats itself
   :id: CREQ_SOURCE_NO_REPEAT
   :derived_from: FEAT_CONTEXT_IDENTITY
   :allocated_to: COMP_IDENTIFIER_SOURCE
   :ears_pattern: ubiquitous
   :statement: Identifier source shall never issue the same identifier twice.

   The parent is about contexts and this is about issuance, and the two part
   company in both directions. An identifier issued and then never used for a
   context would break this without breaking the parent. An identifier made
   without the source would break the parent without breaking this, which is
   what the requirement below exists for.

   Failure modes:

   - **The identifier space runs out.** Reported as exhaustion, a failure of its
     own rather than a generic one, and the source stays exhausted: every later
     request is refused the same way. Wrapping round to the first identifier is
     the defect this rules out, since it would reissue without a word.
   - **The source is copied.** Two copies would issue the same sequence, each
     correct on its own and repeating jointly. So a source cannot be copied, and
     an attempt to copy one is refused when the code is compiled rather than when
     it runs.

   Not this component's: two separate sources created for one run would each
   keep this requirement and jointly break the parent. Nothing in this feature
   has a run to prevent that, so it belongs to whatever owns a run once the
   engine exists.

.. comp_req:: Only the source makes identifiers
   :id: CREQ_SOURCE_SOLE_ISSUER
   :derived_from: FEAT_CONTEXT_IDENTITY
   :allocated_to: COMP_IDENTIFIER_SOURCE
   :ears_pattern: ubiquitous
   :statement: Identifier source shall be the only means of obtaining a context identifier, except by resuming a run's record together with a source positioned past every identifier the record holds.

   A source that never repeats guarantees nothing if an identifier can be made
   without it. This closes the gap the requirement above leaves open.

   The exception was added with ``STKH_RESUMABLE_RUN``, which needs identifiers
   read back from storage: a resumed run holds the contexts it was recorded with
   (``FEAT_RESUME_KEEPS_CONTEXTS``). It keeps the promise rather than breaking
   it, because a record's identifiers come back only beside a source that will
   never issue them (``CREQ_RECORD_SOURCE_CONTINUES``), and a record holding
   one at or past that source's position is refused. Code outside the core
   still has no route to an identifier but these two.

   Failure modes:

   - **Code constructs an identifier directly.** Refused when the code is
     compiled, because an identifier has no constructor outside the source.
   - **A record's identifiers resumed without a source past them.** A record
     naming an identifier its source had not reached yet is resumed, and the
     source then issues it a second time.

.. comp_req:: Text reads back exactly
   :id: CREQ_VALUE_TEXT_EXACT
   :derived_from: FEAT_CONTEXT_STABLE_CONTENT
   :allocated_to: COMP_CONTEXT_VALUE
   :ears_pattern: ubiquitous
   :statement: Context value shall render a text context as exactly the text it was created with.

   The base case of stable content. The defect it rules out is normalisation:
   processing meant to help that changes bytes - line endings converted, text
   trimmed, Unicode normalised. Each of those returns identical content on every
   read, so the parent holds, and each breaks provenance anyway, because the
   bytes read back are not the bytes supplied.

   Failure modes: none. Rendering a text context cannot fail.

   Must still accept, and return byte for byte rather than refuse: empty text;
   CRLF and a lone CR; leading and trailing whitespace; non-ASCII text, including
   combining sequences; an embedded NUL.

   A context is created from text rather than from raw bytes. Turning a
   command's output into text, and refusing what is not valid text, belongs to
   the node that reads it.

.. comp_req:: Composed content is its parts, joined
   :id: CREQ_VALUE_RENDER_JOINED
   :derived_from: FEAT_CONTEXT_STABLE_CONTENT
   :allocated_to: COMP_CONTEXT_VALUE
   :ears_pattern: ubiquitous
   :statement: Context value shall render a composed context as its parts' rendered content in order, joined by the composition's separator.

   This defines composed content as a function of values that cannot change,
   which makes stable content hold by construction rather than by care. It is
   stricter than the parent: a render that reversed the parts every time would
   be perfectly stable and wrong.

   The separator belongs to the composition. Two compositions of the same parts
   with different separators are different contexts with different content, and
   nothing global decides it.

   Failure modes: none. Rendering a composed context cannot fail.

   Must still accept: no parts, which renders as empty content; one part, which
   renders with no separator; an empty separator, which concatenates; the same
   part given twice, which renders twice.

   The defect to guard against is exhausting the stack. Nesting has no fixed
   depth - a loop that composes its own previous output nests one level deeper
   on every pass - so rendering must not spend a stack frame per level. A
   process abort is not a failure anyone can assert on.

.. comp_req:: Parts are the contexts themselves
   :id: CREQ_VALUE_PARTS_BY_REFERENCE
   :derived_from: FEAT_CONTEXT_PARTS
   :allocated_to: COMP_CONTEXT_VALUE
   :ears_pattern: ubiquitous
   :statement: Context value shall return a composed context's parts in the order given, each as the original context rather than a copy.

   ``DEC_COMPOSITION_BY_REFERENCE`` at the level of one component. The parent is
   satisfied by copies that keep their order: copies with fresh identifiers are
   individually addressable and correctly ordered, and every one of them is cut
   off from the context it came from. Identity is what makes the difference
   observable - a part is the original exactly when it carries the original's
   identifier.

   Failure modes: none of its own. Composing needs an identifier and a declared
   type, and the ways obtaining those can fail belong to the requirements that
   own them.

   Must still accept: no parts at all; the same context given twice, which
   appears at both positions and is itself both times; parts of different
   declared types.

   The defect to guard against is the stack again, from the other end. Holding
   parts by reference is what builds a deep chain, and releasing one must not
   spend a stack frame per level either.

.. comp_req:: The declared type is the type
   :id: CREQ_VALUE_DECLARED_TYPE
   :derived_from: FEAT_CONTEXT_DECLARED_TYPE
   :allocated_to: COMP_CONTEXT_VALUE
   :ears_pattern: ubiquitous
   :statement: Context value shall report the type a context was declared with, whatever the types of its parts.

   ``DEC_CONTEXT_TYPING`` is nominal: a type is what a context was declared to
   be, and nothing infers it. A composition of a summary and a transcript is
   whatever it was declared as, which is neither of those. The parent could be
   met by a type inferred from the parts or from the content, and this rules
   both out.

   Failure modes:

   - **An empty type name.** Refused when the name itself is made, before any
     context can be declared with it, and reported as an invalid type name
     rather than as a generic failure. Nothing carrying the name exists
     afterwards, context or otherwise. A type keys global contexts and validates
     wiring, and an empty key does neither.

   Deliberately not restricted further. Which characters a type name may hold
   is a question for the workflow format, and deciding it here would be guessing
   before there is anything to judge it against.

.. comp_req:: Every ancestor is reported once
   :id: CREQ_WALKER_EACH_ONCE
   :derived_from: FEAT_CONTEXT_LINEAGE
   :allocated_to: COMP_LINEAGE_WALKER
   :ears_pattern: ubiquitous
   :statement: Lineage walker shall report every context reachable through a context's parts exactly once.

   The parent asks for the whole ancestry. This adds that it is a set, and the
   addition is not cosmetic. Parts are shared, so an ancestor is often reachable
   along several paths, and counting paths is exponential: a stack of n diamonds
   has two to the n paths to its bottom. A walker that reports per path is
   complete, correct in content and unusable.

   The context being walked is not its own ancestor and is not reported. The
   order of the report is not part of this requirement, so a test compares it as
   a set.

   Failure modes: none. Walking cannot fail, and it always ends, because a
   context can only be composed from contexts that already exist, so no context
   can be its own ancestor.

   Must still accept: a text context, which has no ancestors; a part given
   twice, reported once; a shared ancestor reachable along two paths, reported
   once; a stack of diamonds, walked in time proportional to the contexts in it
   rather than to the paths through them.

   The defect to guard against is the stack, as for rendering: the walk must not
   spend a frame per level of nesting.
