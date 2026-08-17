=======================
Fixture: rule_id_prefix
=======================

.. One need of every type whose id prefix disagrees with its type, so this
   fixture covers all six id-prefix rules at once. Each need is otherwise fully
   conformant - every mandatory field present, every link pointing at a need of
   the right type - so the only thing wrong with any of them is the prefix.

   The needs link to each other rather than to a separate valid spine: a link
   rule cares about the TYPE of its target, and these are all of the right types
   despite their ids. That keeps the fixture to six needs.

.. stkh_req:: A stakeholder requirement not called STKH
   :id: WRONG_STKH
   :stakeholder: maintainer
   :statement: Agconflo shall reject an id whose prefix disagrees with its type.

.. feat_req:: A feature requirement not called FEAT
   :id: WRONG_FEAT
   :derived_from: WRONG_STKH
   :ears_pattern: ubiquitous
   :verification_method: review
   :statement: Agconflo shall reject an id whose prefix disagrees with its type.

.. feat_arch:: A feature architecture not called ARCH
   :id: WRONG_ARCH
   :realises: WRONG_FEAT
   :uses: WRONG_COMP
   :statement: Agconflo shall check id prefixes in the schema layer.

.. comp:: A component not called COMP
   :id: WRONG_COMP
   :crate: agconflo-core

.. comp_req:: A component requirement not called CREQ
   :id: WRONG_CREQ
   :derived_from: WRONG_FEAT
   :allocated_to: WRONG_COMP
   :ears_pattern: ubiquitous
   :statement: A component not called COMP shall reject a mismatched id prefix.

.. test_case:: A test case not called TEST
   :id: WRONG_TEST
   :verifies: WRONG_CREQ
   :test_kind: error_path
   :coverage: full
