============================
Evidence about the toolchain
============================

Measurements the decisions beside these rest on. Each says how it was obtained,
so it can be re-run rather than believed, and each is dated because evidence goes
stale: nearly all of it was taken against one pinned version of one tool.

.. evd:: ubc covers the documentation stack natively
   :id: EVD_UBC_REPLACES_STACK
   :evd_kind: vendor_doc
   :observed_on: 2026-08-16
   :observation: ubc provides the needs model, schema validation, code links and diagram rendering with no Python packages.

   Read from the tool's own documentation and changelog rather than run, which is
   why this is the weakest of the first four and labelled as such. It replaces the
   build engine, the needs model, schema validation, the code-link extractor and
   diagram rendering, the last of those without a Java runtime.

   One capability has no equivalent and the decision beside this one accepts it:
   there is nothing corresponding to the test-report importer, so bringing test
   results into the graph will need a small tool written here.

.. evd:: The formatter skips Markdown
   :id: EVD_FORMATTER_RST_ONLY
   :evd_kind: measurement
   :observed_on: 2026-08-18
   :observation: ubc format processed the six reStructuredText files of this project and skipped a Markdown file in the same source set.

   To re-run it: widen the source include pattern to cover Markdown as well, put a
   Markdown file beside the requirements, and run the formatter in check mode.

   Dated later than the decision it supports, deliberately. The markup choice was
   made on exactly this basis, but the method was never written down, so this is
   the first time it has actually been run here. An observation is dated when it
   was made rather than when it would be convenient for the arrow to point the
   other way.

.. evd:: A need's body is invisible to schema validation
   :id: EVD_CONTENT_INVISIBLE
   :evd_kind: measurement
   :observed_on: 2026-08-17
   :observation: An impossible pattern required of every decision's body matched nothing and reported nothing.

   Method: add a rule demanding a pattern that cannot occur in the body of every
   need of one type, then check the project. Six decisions, all of which have
   bodies, produced no violation whatsoever. The only sign that anything was wrong
   was a configuration note saying the property was not found in the field
   resolver.

.. evd:: A wrongly shaped rule is ignored rather than rejected
   :id: EVD_SILENT_RULE_DROP
   :evd_kind: measurement
   :observed_on: 2026-08-17
   :observation: A not keyword directly under validate.local reported nothing, where the same keyword inside allOf reported six violations.

   The sharpest of the first four, because it is an exact A and B. The rule forbade
   every decision from having an identifier beginning with its own prefix, which
   all six of them violate. Placed one way it found nothing; wrapped in a
   composition keyword it found all six.

   One wrapper is the entire difference between a rule that enforces and a rule
   that is decoration, and nothing reports the difference. This is the failure the
   fixture harness exists to catch.

.. evd:: Two misshapen rule forms still pass silently on 0.35.0
   :id: EVD_UBC035_RULE_SHAPES
   :evd_kind: measurement
   :observed_on: 2026-09-19
   :observation: On ubc 0.35.0 a misspelled keyword and a keyword of the wrong kind still pass silently, while a composite keyword under validate.local now fails the check.

   Taken when the pin moved from 0.31.2b1 to 0.35.0, as an A and B: the same
   probe rules against both binaries, each appended alone to the real rules and
   checked over the real project, with a control rule that fired on all twelve
   decisions under both versions.

   A misspelled keyword and a keyword of the wrong kind for the field - a
   minimum on a string - pass silently on both, exit 0 and nothing printed. A
   composite keyword placed directly under ``validate.local``, silent on
   0.31.2b1, is now reported as an unknown key that fails the check, and with it
   ubc stops applying every rule in the file: the control placed beside it fired
   on nothing. A rule about a need's body matches nothing on either version and
   is reported on both as a configuration warning that fails the check.

   That last result corrects the observation of ``EVD_CONTENT_INVISIBLE``, which
   says such a rule reported nothing. It reported no violation, and the body is
   still invisible to validation, so the decision resting on it stands - but the
   gate did not stay green.

   The fixture harness is still needed for the two shapes that pass silently,
   and for any rule of the right shape that matches the wrong thing.
