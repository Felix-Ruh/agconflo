===============================
Evidence about deciding a route
===============================

Measurements for ``STKH_ROUTING``, taken before anything is decided for it:
what a model built to decide rather than to write answers, how it is reached,
how it fails, and what the engine already records of a call that offers a
model no tools.

Unlike the measurements in ``evidence/models``, these reached a provider:
OpenRouter's decisions endpoint, ``/api/alpha/decisions``, with the model
``~typesafe/jev-latest``, through a key the maintainer allowed for testing, from
a cloud development container through its proxy, on 2026-09-26. "Alpha" is
OpenRouter's word for the endpoint, and it may change shape; each finding
below is of the day it was taken. The last was taken after, through what was
specified and built on the others.

.. evd:: A decisions model answers only on its own endpoint, in a shape of named questions
   :id: EVD_DECISIONS_MODEL_SHAPE
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: Jev answered only on /api/alpha/decisions, taking a state and named questions of type choice, score or noul, and gave each choice with probabilities and a confidence, each score between 0 and 1, and each noul as one number.

   ``~typesafe/jev-latest`` on ``/api/v1/chat/completions`` was refused with
   400: "is a decisions model and cannot be used with the chat/completions
   endpoint". On the decisions endpoint a request is ``model``, ``state`` and
   ``questions``; the state was accepted as text and as a record. A choice
   question takes ``instructions`` and ``criteria``, a record from each option
   to what it means; a score question takes ``instructions`` and ``criteria``
   as a list, read as the two ends of the scale and given back as its
   ``legend``; a noul question takes ``instructions`` alone and was answered
   as a number between 0 and 1. The model answering was named
   ``typesafe/jev-1.13-20260917`` and the provider ``TypeSafe``.

   ``typesafe/jev-router`` is a different thing: a chat model that picks
   another to answer, and it answered a classification on the chat endpoint
   through ``deepseek/deepseek-v4.1-flash``. It is not the decisions model.

.. evd:: The same decision asked five times gave the same choice every time
   :id: EVD_DECISIONS_REPEAT
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: Asked five times each about a clean review and one with three findings, Jev chose accept and revise every time with confidence 1, in 337 to 433 ms at a fixed cost per state, while scores and noul values moved slightly between runs.

   The questions were one choice (accept or revise), one score (how likely
   the change breaks the build) and one noul (whether it is ready to merge).
   For the clean review every score was 0.01 with confidence 0.98, and the
   noul 0.72 to 0.74. For the review with findings the score ranged 0.53 to
   0.61 with confidence 0.06 to 0.22, and the noul was 0.05 every time. Each
   call cost 0.00001743 dollars for the first state and 0.000017724 for the
   second, the same on every repeat. Two states and five repeats show a
   choice that did not waver; they do not measure how often one would.

.. evd:: A decisions request is refused with a status and a list of what is wrong
   :id: EVD_DECISIONS_REFUSALS
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: A malformed decisions request came back 400 with a JSON list of issues inside the error message, a chat model's name 400, a bad key 401, and a provider the account's guardrails exclude 404 with the reason in metadata.

   An unknown question type gave ``invalid_union`` naming the options
   ``noul``, ``choice`` and ``score``; a choice with no criteria
   ``invalid_type`` at ``questions.q.criteria``; no question at all "At least
   one question is required". Each list is text inside ``error.message``,
   parseable as JSON. A chat model's name gave "Model ... does not exist".
   The guardrail refusal read "0 endpoints out of 1 requested are available
   matching your guardrail restrictions", with
   ``provider-not-allowed-by-guardrail`` in its metadata, and is the one of
   these a person fixes outside the workflow.

.. evd:: The engine already records and replays a model exchange that offered nothing and made no call
   :id: EVD_EXCHANGE_WITHOUT_OFFER_REPLAYS
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: A run's record held a model exchange with an empty offer and no calls, from a node declaring no calls, and the run was resumed from that record twice and went on past it both times.

   The run was a seven-instance workflow driven by ``agconflo run`` and
   ``agconflo answer``; its planning node calls a model with no declared
   calls, so ``agconflo-core``'s ``Exchange::new(window, answer)`` - documented
   as "offering nothing ... making no call" - is what it reported. Each
   answer to a later person's step replayed the record through that exchange
   before reaching the step. A decision is the same shape: something sent, an
   answer back, nothing offered and nothing called.

.. evd:: The HTTP client a decisions call needs is already built, only through genai
   :id: EVD_REQWEST_THROUGH_GENAI
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: reqwest 0.13.5 was in the workspace's build only as genai's dependency, with its json, rustls, http2 and system-proxy features on.

   ``cargo tree -i reqwest`` over the workspace at ``d072ee8``. ``genai``
   speaks each provider's chat format and nothing else, so the decisions
   endpoint is not reachable through it; a direct dependency on the same
   ``reqwest`` with those features adds no crate to the build. The calls
   above were made with Python's ``urllib`` through the container's proxy,
   not through ``reqwest``; whether ``system-proxy`` reaches that proxy the
   same way was not measured.

.. evd:: A router's script asked Jev through agconflo and branched on its choice
   :id: EVD_JEV_THROUGH_AGCONFLO
   :evd_kind: measurement
   :observed_on: 2026-09-26
   :observation: Through agconflo at 0e81bec, a router's host.decide chose accept for a clean review and revise for one with findings, confidence 1 each, in under 0.7 s a run; the record held an exchange offering nothing, and a bad key failed with 401.

   Taken with the ``agconflo`` binary built for debugging, from a scratch
   directory: a workflow of an entry instance giving a review and a router
   instance declaring no calls, whose script asked ``host.decide`` one
   question - revise or accept, with the instructions and criteria of
   ``EVD_DECISIONS_REPEAT`` - and output the choice, the confidence and the
   answer; a model mapping giving the role ``routing`` the decisions model
   ``~typesafe/jev-latest`` at ``https://openrouter.ai/api/alpha/``, its key
   in a variable given to that one process, as the maintainer allowed for
   testing. ``check`` found nothing first.

   Each run exited 0, its output the choice, its confidence and the answers
   as the endpoint sent them, probabilities included. The times are whole
   runs of the command. The record held the decision as ``exchange = {
   offer = [], window = "4", answer = "5", calls = [] }``. With a key the
   endpoint does not know, the run exited 5 with "the model for routing
   answered 401" and the endpoint's body, "User not found.". The request went
   through ``reqwest`` and the container's proxy, which
   ``EVD_REQWEST_THROUGH_GENAI`` had left unmeasured.
