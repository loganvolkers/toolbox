# Design audit rubric

Score every dimension 0–4 from rendered evidence only. Cite the screenshot or metric for each
score. A score without evidence is a 0.

| Score | Meaning |
|---|---|
| 0 | Absent or actively misleading |
| 1 | Present but a reader fails the related task without help |
| 2 | A reader succeeds with effort, rereading or guessing |
| 3 | A reader succeeds; a few rough edges remain |
| 4 | Effortless; would hold up as a reference example |

## Usability (U): can the reader get the answer?

Adapted from Nielsen's ten heuristics, Few's dashboard rules, and WCAG 2.2 AA.

| Id | Dimension | A 4 looks like | Common failures |
|---|---|---|---|
| U1 | Answer first | The first viewport states the verdict for the primary reader in one glance (a big number, a status word, a sentence) | Inventory or a table leads; the verdict is a run-on sentence; the key number is in small grey text |
| U2 | Reader's language | Human titles and plain words; machine ids are secondary; jargon is explained where it appears | Every label is a code identifier; levels and terms are never defined |
| U3 | Recognition over recall | Legends, units and "what this means" sit beside the data; nothing must be remembered between views | A level name without its meaning; counts without a denominator |
| U4 | Hierarchy and scent | Size, weight, colour and spacing rank content; headings say what the section answers; the eye has a path | Every section is the same white card at the same size; equal weight for primary and tertiary content |
| U5 | Data integrity | Encodings fit the data (position/length for quantities, ordered colour for ordered levels); proportions are comparable | Unordered hues for an ordered scale; tiny bars; legends repeated per row |
| U6 | Consistency | One thing, one look: a level, an outcome or an entity renders the same everywhere | The same level is a badge in one view and plain text in another |
| U7 | Wayfinding and control | Where am I, how did I get here, how do I go back or share this; filters show their state; empty states offer a way out | Lost after a drill-down; no breadcrumb; filters that reset silently |
| U8 | Honest states | Zero, empty, unknown and skipped are explained, never contradictory | "0%" next to "every entity meets the goal"; unknown shown as failure |
| U9 | Accessibility | No serious axe violations; 4.5:1 text contrast; not colour-only; keyboard reachable; targets ≥ 24 px | Nested interactive controls; low-contrast muted text; colour-only status |
| U10 | Density and responsiveness | Long lists are summarised or paged; mobile keeps the verdict and primary action | Thousands of rows rendered at once; tables overflow on a phone |

## Joy (J): does the reader want to come back?

Norman's three levels of emotional design, Walter's hierarchy of user needs (functional →
reliable → usable → pleasurable), and Kano's delighters. Joy never costs a usability score: a
delighter that lowers any U score is a regression.

| Id | Dimension | A 4 looks like | Common failures |
|---|---|---|---|
| J1 | Visceral: first impression | Distinct visual identity, rhythm and contrast between sections; a glance reads as crafted | Homogeneous cards and tables; one font size; nothing to look at first |
| J2 | Behavioral: feel of use | Hover, focus and transitions confirm actions; drill-downs feel fast and reversible | Static clicks with no feedback; jumpy re-renders |
| J3 | Reflective: progress and pride | What already passes is celebrated; progress toward the next level is visible; momentum is named | Only debt is shown; achievements are invisible |
| J4 | Voice | Microcopy is human, specific and encouraging without being cute; empty states have character | Robotic or internal phrasing; instructions about keyboard keys in body text |

## Tasks (T): can named readers finish real jobs?

For each reader task, record: outcome (success, partial, fail), steps (clicks or scrolls from the
landing view), and the single biggest obstacle. A task is a success only when the auditor can
state the answer and point to where it came from.

## Findings

One row per finding: id, dimension, severity, evidence, recommendation.

| Severity | Meaning (frequency × impact × persistence) |
|---|---|
| 4 | Catastrophe: a reader draws the wrong conclusion or cannot finish a primary task |
| 3 | Major: a primary task needs effort or guessing; fix before release |
| 2 | Minor: slows a reader or erodes trust |
| 1 | Cosmetic: fix when touching the area |

## Prioritisation

1. Usability fixes: severity first, then reach (how many readers and views it touches), then
   effort. Every severity 3–4 is a must.
2. Joy opportunities: Kano delighters that reinforce a task (progress, achievement, identity),
   ranked by how often a reader sees them. Drop any that add noise to a primary view.

## Sources

Nielsen, *10 Usability Heuristics* (1994, 2020) and *Severity Ratings for Usability Problems*;
Few, *Information Dashboard Design* (2013); Tufte, *The Visual Display of Quantitative
Information* (2001); W3C, *WCAG 2.2* (2023); Norman, *Emotional Design* (2004); Walter,
*Designing for Emotion* (2011); Kano et al., *Attractive quality and must-be quality* (1984);
Wharton et al., *The cognitive walkthrough method* (1994); Bach et al., *Dashboard Design
Patterns* (2022).
