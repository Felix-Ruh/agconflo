=============================
Fixture: rule_statement_shape
=============================

.. The two levels that carry a statement but no EARS pattern, and so are held
   only to the loose shape: a capitalised subject, the word shall, a terminal
   period.

   The stakeholder requirement here describes instead of obliging, which is the
   commonest way a goal stops being checkable. The architecture decision starts
   lowercase - trivial, but it is the anchor that stops the subject slot from
   matching mid-sentence, so it is worth having pinned.

.. stkh_req:: A goal that describes rather than obliges
   :id: STKH_DESCRIBES
   :stakeholder: user
   :statement: Agconflo records the provenance of every context.

.. feat_req:: A conformant parent for the architecture decision below
   :id: FEAT_SHAPE_OK
   :derived_from: STKH_DESCRIBES
   :ears_pattern: ubiquitous
   :verification_method: review
   :statement: Agconflo shall record the provenance of every context.

.. comp:: A component for the architecture decision to use
   :id: COMP_SHAPE_OK
   :crate: agconflo-core

.. feat_arch:: An architecture decision whose statement starts lowercase
   :id: ARCH_LOWERCASE
   :realises: FEAT_SHAPE_OK
   :uses: COMP_SHAPE_OK
   :statement: agconflo shall record provenance inside the context store.
