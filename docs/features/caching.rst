===========================
Caching a model's next turn
===========================

What a provider is given so that it can serve a model call from what it
cached for an earlier one. The one requirement here derives from
``STKH_PROMPT_CACHE_REUSED`` and is written against the decisions in
``decisions/caching``.

It covers the turns of one activation, each beginning with the whole of the
turn before. Calls of different activations that begin alike - a node
type's instructions on every pass - are the rest of the parent, left undone
for the reason ``DEC_CACHE_MARKED_BEFORE_A_CONTINUATION`` gives; a provider
that caches by itself serves them all the same.

.. feat_req:: A model's next turn can be served from the cache of the turn before
   :id: FEAT_CONTINUATION_CACHED
   :derived_from: STKH_PROMPT_CACHE_REUSED
   :ears_pattern: event
   :verification_method: test
   :statement: When a model continues after calls it made, Agconflo shall let the provider serve the window it was sent before from its cache.

   The parent for the calls one activation makes. Each turn after the first
   begins with the whole of the one before (``CREQ_HOST_NEXT_WINDOW``), so it
   is the call the parent most surely applies to, and the steps that call
   node types most are the ones it applies to most often.

   It claims no more than the parent: it is the parent narrowed to one kind
   of call that begins as an earlier one did, the kind ``genai`` lets
   Agconflo serve today. Narrowed, it can be tested: what reaches a stub
   shows whether a turn begins with the one before and carries a mark where
   that one ended, which is everything a provider of either kind needs from
   the request.

   It can be false while the parent's other calls are served. A host that
   marked where a prompt's instructions end, for the next activation of a
   node type, would serve those and leave every turn of a calling step to be
   paid in full on a provider that caches only what is marked.

.. feat_arch:: Marking a window for the provider's cache is the model roster's
   :id: ARCH_CONTINUATION_CACHED
   :realises: FEAT_CONTINUATION_CACHED
   :uses: COMP_MODEL_ROSTER
   :statement: Agconflo shall allocate marking a model call's window for the provider's cache to the model roster.

   The roster turns a window's parts into messages, and knows which of them
   is a model's answer, so it can tell where the turn before ended from the
   window alone. It is also where ``genai`` is reached, so a mark is set
   where the rest of the request is (``DEC_CACHE_MARKS_THROUGH_GENAI``).

   The script host is not among the components: it already sends each turn
   the window before with what followed it after (``CREQ_HOST_NEXT_WINDOW``),
   and nothing in that changes.
