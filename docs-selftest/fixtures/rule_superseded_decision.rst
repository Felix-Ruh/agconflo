=================================
Fixture: rule_superseded_decision
=================================

.. A decision marked superseded that nothing supersedes: a tombstone with no
   forwarding address, which is worse than deleting the decision outright,
   because it says "do not follow this" without saying what to follow instead.

   This is the project's only `network_back` rule and the only rule anywhere here
   that looks at a need's INCOMING links. The message says
   `supersedes (incoming)`, and that word is the whole difference from the
   forward link rules.

   It is safe to gate on where a coverage rule would not be, and the difference
   is worth understanding rather than memorising. A coverage rule fires on every
   need that has not yet been built out, so it would block the first step of
   authoring - which is why there are none in this project. This one fires only
   on a need whose author has deliberately typed `superseded`, and that is itself
   a claim that a successor exists.

   One limit of this fixture, stated because it cannot be tested here: every
   fixture is checked as a one-file project, so the successor would have to share
   this file. In the real requirements project it will not, and that was measured
   separately - `network_back` resolves across files, going silent as soon as a
   superseding decision exists anywhere in the project.

.. dec:: A decision marked superseded with nothing superseding it
   :id: DEC_ORPHANED
   :dec_status: superseded
   :decided_on: 2026-08-14
   :statement: Agconflo shall build its requirements project with Sphinx and Python.
