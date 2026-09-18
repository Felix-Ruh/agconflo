===================================
Evidence about writing requirements
===================================

Measurements of the requirements themselves, as opposed to the tool that checks
them. Each is dated, and each says how to re-run it.

.. evd:: The first feature's requirements are all ubiquitous
   :id: EVD_EARS_FIRST_DISTRIBUTION
   :evd_kind: measurement
   :observed_on: 2026-09-18
   :observation: All twelve requirements written for the Context feature use the ubiquitous EARS pattern, and none uses the other five.

   Five feature requirements and seven component requirements. At the time they
   were the only needs carrying ``ears_pattern``, so the whole measurement was one
   query, grouping requirements that carry the field by pattern and level: it
   returned two rows, both ubiquitous.

   Re-running it later counts every feature written since, so the Context
   feature has to be selected explicitly: its feature requirements are the ones
   whose id begins ``FEAT_CONTEXT_``, and its component requirements are those
   derived from them.
