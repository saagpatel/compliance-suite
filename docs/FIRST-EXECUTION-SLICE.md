# Compliance Suite — First Execution Proof Slice

Defines the minimum end-to-end customer outcome that proves the
`questionnaire` lane is more than packaged software — it's a thing
someone would pay to use. If this slice runs against a real customer's
real questionnaire, the product premise is validated and the case for
hardening `binder` and `sop` opens up. If it does not, the suite is
still "shipping the build," not "delivering the value."

> **Audience:** anyone resuming compliance-suite work, scoping a
> first-customer pilot, or evaluating whether further investment is
> warranted.

---

## Why a slice doc, not another readiness doc

This repo already has thorough engineering-readiness documentation:

- `docs/source-of-truth/current-state.md` — what the codebase can do
- `docs/release/questionnaire-readiness.md` — release-candidate state
- `docs/release/manual-smoke-checklist.md` — packaged-app smoke gates
- `docs/release/signing-and-notarization.md` — distribution gates

What is missing is the **customer-outcome** measurement. The portfolio
operating system flagged a "first governed execution slice" packet
specifically because release-readiness ≠ business-readiness. This file
fills that gap.

A slice is a vertical cut through the workflow (import a real artifact
→ produce a real output the customer can hand to an auditor or
procurement counterparty) that delivers one observable business
outcome. Build health proves the bridge works; the slice proves
someone actually wants to cross it.

---

## The slice (v0)

**One pilot user, working from a real vendor security questionnaire
(SIG-Lite, CAIQ, or a custom RFP attachment), can import it,
answer-bank-match it against their saved evidence, complete the
review of all rows, and export a populated questionnaire pack the
user delivers to the counterparty.**

That's it. No `binder` lane involvement, no `sop` lane, no
multi-user, no cloud sync, no auditor seat.

### Slice ingredients

| Layer | What must exist |
|---|---|
| Import | Questionnaire app accepts at least one industry-standard format (SIG-Lite XLSX or CAIQ XLSX preferred) |
| Vault | At least one piece of evidence stored locally — a policy PDF or SOC 2 report fragment |
| Answer bank | At least 10 saved answers from prior questionnaires |
| Mapping | Auto-match new questions to bank answers with a confidence score the user can see |
| Review | Row-by-row pass through every question; user accepts, edits, or marks "needs evidence" |
| Export | Output an XLSX (or CSV / PDF — whichever the counterparty wants) with all answers populated and evidence references inline |
| Audit chain | Every action (import → map → edit → export) recorded immutably so the pilot user can show their work |

### Slice non-ingredients (defer)

- `binder` evidence collection app (out — that's its own slice)
- `sop` document lifecycle (out — that's its own slice)
- Signed / notarized distribution (out — unsigned `.app` is fine for
  a pilot under a directly-negotiated agreement)
- Multi-user vault (out — single operator only)
- Cross-platform builds (out — macOS-first per the repo's stated
  posture)
- Cloud anything (out — local-first is the point)

If any of these feel essential, the slice ambition is wrong.

---

## Definition of done — observable proof

The slice is "done" when one pilot user can reproduce the table below
in under 30 minutes from a working install, in front of an observer
who has not seen the workflow before.

| # | Action | Proof artifact |
|---|---|---|
| 1 | Launch the questionnaire app | App window opens; vault unlock works |
| 2 | Import a real SIG-Lite or CAIQ file | Question rows visible; row count matches source |
| 3 | Map answers from the bank | Auto-match suggestions visible with confidence scores |
| 4 | Walk the review pass | Each row shows accepted / edited / needs-evidence state |
| 5 | Mark at least 5 rows "needs evidence" | Vault prompts for evidence import on those rows |
| 6 | Export the populated questionnaire | XLSX output file produced; counterparty-ready |
| 7 | Show the audit chain | Every action visible with timestamp + actor (per the local-session actor work already shipped) |

Each row should produce a screenshot or file artifact checked into
`docs/slice-v0-proof/2026-XX-XX-pilot/` (date in path).

---

## Verification — local

Before recruiting a pilot, run from a clean checkout:

```bash
pnpm install
pnpm verify
pnpm smoke:questionnaire
pnpm package:questionnaire   # produces an unsigned .app
```

All four must succeed. After packaging, manually walk the `manual-smoke-checklist.md` against the packaged build, and additionally cover steps 1–7 above against an actual test questionnaire (e.g., a sanitized SIG-Lite from a prior engagement).

If any step fails on the packaged build, the slice is not done.

---

## Verification — pilot

The slice is **proven** when a real pilot user, NOT the author,
reproduces steps 1–7 end-to-end with their own vendor questionnaire
and their own evidence. Author-driven reproducibility doesn't count —
the question is "does someone else want to use this," not "does it
work when I run it."

The pilot can be informal: one operator, one questionnaire, a 30-min
Zoom, the audit chain emailed back as proof.

---

## What this slice does NOT prove

- It does NOT prove the product is differentiated (every compliance
  vendor has a questionnaire flow).
- It does NOT prove pricing (a pilot tells you "willing to use," not
  "willing to pay").
- It does NOT prove `binder` or `sop` are worth shipping — those are
  separate slices.
- It does NOT prove distribution-readiness (still unsigned).
- It does NOT prove auditor acceptance (the export is for vendor
  questionnaires, not external audit reports).

These are deliberate omissions, not gaps.

---

## Next slice candidates (informational, not committed)

Once the v0 slice is green and the pilot has used the export at least
once with a real counterparty, the natural next slices in priority
order:

1. **Pricing slice** — quote-to-pay loop with one pilot. Proves
   willingness to pay, not just willingness to use.
2. **Binder evidence slice** — pilot user imports SOC 2 report
   evidence and links it to controls. Proves the second lane's
   value beyond questionnaire-only.
3. **Signed-distribution slice** — first signed/notarized release
   delivered to a paying customer. Proves trust-tier delivery.
4. **Multi-user vault slice** — second seat per pilot user (auditor
   read-only). Proves the team workflow.

Each becomes its own packet.

---

## When to escalate / reframe

If the v0 slice can't be reproduced by a pilot user in 30 minutes,
the right move is to **reframe**, not push harder. Likely reframes:

- Drop "auto-match confidence score" from the slice — start with
  manual mapping.
- Drop XLSX export, allow CSV or even Markdown export — fewer
  rendering bugs.
- Drop the audit chain visibility requirement — keep the chain
  recording but don't surface it in the slice UI.
- Shrink the pilot to a single 10-row custom questionnaire instead of
  a full SIG-Lite (170+ rows) — proves the loop without endurance
  pressure.

The reframe is the point: ship a thinner slice that proves *someone
wants this*, not a thicker one that proves nothing.

---

## Status mapping for portfolio operating system

Independent of this slice's status, the suite's build/release posture
remains "macOS unsigned RC, awaiting Apple credentials" per
`docs/source-of-truth/current-state.md`. The slice is **orthogonal**
to that: the slice can be proven on an unsigned dev build delivered
directly to a pilot user; signing only matters at general distribution.

| Slice state | Portfolio posture |
|---|---|
| Not yet attempted | `Active` — first packet is "recruit pilot user" |
| Pilot scheduled | `Active` — next packet is "run the 7 steps with pilot" |
| Pilot reproduced steps 1–7 with their questionnaire | **Slice proven** — graduate to pricing slice |
| Pilot bounced off any step | `Active` — next packet is "reframe the failing step" |
