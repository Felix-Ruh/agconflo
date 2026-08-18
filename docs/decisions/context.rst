=======================
Decisions about context
=======================

The choices the context model rests on. These are decisions rather than
requirements: each is a *how*, chosen to serve a goal recorded among the
stakeholder requirements. The first feature is written against them, and
re-opening one means superseding it rather than quietly disagreeing with it.

.. dec:: A composed context keeps its parts
   :id: DEC_COMPOSITION_BY_REFERENCE
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall hold a composed context as ordered references to its parts rather than as copied text.

   Two reasons, and one that is explicitly not a reason.

   Provenance is the product here, and "which bytes came from where" needs
   structure to answer. A flattened string plus a note naming its parents cannot
   answer it, because the correspondence between the two is exactly what was
   thrown away.

   Reordering for a provider's prompt cache needs fragment boundaries to reorder.
   Caching is an exact byte-prefix match - measured at 14,848 cached tokens of
   14,902 on a repeat call - so a flat string cannot be rearranged to maximise a
   shared prefix. That measurement is deliberately not linked as evidence here: it
   is about caching, which this project has deferred, and it travels with the
   caching decision rather than this one.

   Deduplication is *not* a reason, and was dismissed when it was raised. A third
   argument, that metadata would otherwise be lost, was raised and then answered
   better by the decision below, so it is no longer an argument for references
   either.

   This is the minimal form of a fragment graph: no deduplication, no cached lazy
   rendering, no budget logic. All of those remain addable precisely because the
   parts survive.

.. dec:: Metadata is transformed, never inherited
   :id: DEC_METADATA_TRANSFORMED
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall derive a composed context's metadata from a declared transform rather than by inheritance.

   A transform declares how its parents' metadata combine into the output's own,
   as part of the same specification that says how their content combines.

   Deliberately simpler than answering metadata questions by walking the reference
   graph: every context carries its own directly readable metadata at every level
   of composition, so nobody has to ask what the metadata of the fragment at some
   offset was.

   Left open on purpose: the *default* join, so that an author need not specify
   one on every node. The candidate is to keep the keys all parents agree on, drop
   the ones they disagree on, and let the engine add its own provenance keys. It
   is recorded here as open rather than settled, because guessing it before there
   is material to judge it against is how a default becomes permanent by accident.

.. dec:: Content is reached through an API
   :id: DEC_CONTEXT_API
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall expose a context's content through an API rather than through a public field.

   Rendering, fragments and lineage are operations, not a string field.

   This is the seam that keeps the decision above reversible in one direction: a
   public text field would be depended on by every node ever written against it,
   and a later move to lazy rendering or a fuller fragment graph would break all
   of them at once. Behind an API it is an implementation change.

.. dec:: Identity belongs to the activation
   :id: DEC_IDENTITY_PER_ACTIVATION
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall give every context instance an identifier unique to the activation that produced it.

   A fresh identifier per activation is what distinguishes one pass's inputs from
   the same node's inputs on a later pass. Epoch tagging depends on that property:
   without it a join can silently pair the second iteration's context with the
   first's, which is a well-formed wrong answer rather than a crash.

   The corollary is not optional. A workflow definition binds a source node and
   its output to a parameter, never a literal context identifier - it could not do
   otherwise, since no identifier exists until a run does. That is also what makes
   a workflow a reusable definition rather than one graph per unit of work.

.. dec:: A context's type is declared and stays readable
   :id: DEC_CONTEXT_TYPING
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall give every context a declared type that node code can read at run time.

   Nominal rather than structural, and the type earns its place three times over:
   it keys a global context, it validates wiring before anything runs, and it
   documents what a node expects to be given.

   Staying readable at run time is the half that is easy to lose. A node assembles
   its own inputs, knowing its parameters' types and interleaving them
   accordingly, so the type cannot be only a static artifact that is checked and
   then erased before the node ever sees it.

.. dec:: Identity is the identifier and nothing else
   :id: DEC_NO_CONTENT_ADDRESSING
   :dec_status: accepted
   :decided_on: 2026-08-14
   :statement: Agconflo shall identify a context by a generated identifier alone.

   A content hash alongside the identifier is wanted eventually, for caching,
   replay and run-diffing. This records the boundary rather than the ambition:
   until it exists, nothing may assume that two contexts with equal content are
   the same context.

   Worth stating because it is orthogonal to the decision above rather than
   implied by it. Identifiers being fresh per activation does not rule out a
   content hash sitting beside them - the two could coexist, and are meant to. This
   is the decision that says the hash is not there yet.
