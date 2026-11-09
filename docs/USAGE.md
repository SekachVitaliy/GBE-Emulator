# Time and token usage

Derived from the local Claude Code transcripts of this project (17 transcript files: the main sessions plus all subagent sessions; 1,367 assistant messages after de-duplication). Figures are approximate: they only cover what the transcripts recorded.

## Time
- Calendar span: 2026-11-05 18:48 to 2026-11-09 16:38 (EET), 3 days 21 h 50 min, including idle periods and a usage-limit pause.
- Active time: about **6 h 12 min**, computed as the sum of gaps of at most 15 minutes between consecutive events across all transcripts. Agents ran in parallel, so this is elapsed active time, not agent-hours.
- Human time is not tracked; the human's role was approving direction (what to run, disk cleanup, publishing) between agent runs.

## Tokens (all models, all agents)
| Kind | Tokens |
|---|---|
| Input (uncached) | 2,900 |
| Output | 1,282,182 |
| Cache creation | 10,852,728 |
| Cache read | 323,964,180 |

## By model
| Model | Output | Input | Cache creation | Cache read |
|---|---|---|---|---|
| claude-sonnet-5-5 (lead + all coding agents) | 1,224,554 | 2,774 | 10,397,891 | 316,227,144 |
| claude-opus-5-5 (independent code review) | 43,592 | 78 | 221,018 | 6,266,345 |
| claude-haiku-5-5 (background/auxiliary) | 14,036 | 48 | 233,819 | 1,470,691 |

Monetary cost is not computed here (it depends on the pricing in effect for the account). Not counted: any usage not recorded in the transcripts, and the final publishing steps after the figures above were taken.

## Process summary
Phase 1: ruflo init, analysis and task checklist. Phase 2: module skeleton and contract, five agents in parallel. Phase 3: grading loop, three rounds of fitting against the reference (score 0.0085 to 0.934). Phase 4: independent Opus review and fixes (determinism, audio buffer cap, EEPROM region, code cleanup, ABI tests). The score history is in `.claude-flow/data/scores.md`.
