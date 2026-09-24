================================
Changes to existing requirements
================================

Every change to what an existing requirement obliges, one decision each
(``DEC_CHANGE_BY_RECORDED_REQUEST``). A requirement changes only when its parents
change or when it is found wrong against its own parents
(``DEC_REQUIREMENT_ANSWERS_TO_ITS_PARENTS``); the needs of other work are never
the reason.

Each record is a ``dec`` whose body names the requirement it changes, gives the
statement before and after, says what raised the change and which of the two
justifications applies, and carries the analysis against the requirement's own
parents with a verdict on every need the impact analysis found. It is
``supported_by`` an ``evd`` in ``evidence/changes`` holding that impact analysis
as run: the date, the command and the counts in each direction. The changed
requirement's body names its record.

``scripts/change-records.sh`` refuses a change to an existing requirement's
statement, pattern, verification method, stakeholder or placing links, or its
removal, unless the same range adds a line here naming it. The procedure is in
``AGENTS.md``, under "The V-Model".

No change has been recorded yet.
