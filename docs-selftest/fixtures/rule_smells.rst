=====================
Fixture: rule_smells
=====================

.. One requirement per smell class, each tripping exactly one. The three classes
   are the machine-checkable part of ISO 29148 by way of Femmer et al.: wording
   that makes the obligation optional, wording no test could settle, and a degree
   with nothing to compare against.

   Three of the classic smells are deliberately absent, and this is the place to
   look for why. Vague quantities - some, several, many - are left out because a
   regex eats good text there most easily, and one of them already appears
   legitimately elsewhere in these fixtures. "about" is left out because it has
   honest uses and the hyphen counts as a word boundary, so about-face would fire.
   Negative statements are left out because EARS unwanted-behaviour requirements
   need "shall not" by construction, so the check would fight the grammar rules.

   Passive voice and pronoun referents are not here either: they are the two
   smells that genuinely need parsing rather than matching, and they are the least
   reliable of the set.

.. stkh_req:: A loophole leaves the obligation optional
   :id: STKH_SMELL_LOOPHOLE
   :stakeholder: user
   :statement: Agconflo shall compress a context as appropriate.

.. stkh_req:: A word no test could settle
   :id: STKH_SMELL_SUBJECTIVE
   :stakeholder: user
   :statement: Agconflo shall expose a simple interface to the host.

.. stkh_req:: A degree with nothing to compare against
   :id: STKH_SMELL_DEGREE
   :stakeholder: user
   :statement: Agconflo shall render a run log quickly.
