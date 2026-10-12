# Player experience improvements

Native keyboard/mouse play exposed unclear starting goals, repeated confirmation
work, an ambiguous forest masquerading as an inn courtyard, and inadequate
feedback. This pass changes the parts supported by direct evidence:

- Automatically track main objectives and archive them at the exit. Entering a
  map shows three scene lines, rather than a contract-signing sequence.
- Show gold objective categories and nearby interaction hints. Walking past is
  harmless; Space/Enter activates an adjacent objective. The first story is
  within three walkable steps and visible on entry; later goals have a bearing.
- Six mechanisms require authored clue decisions. Wrong answers display hints,
  retain the choices and spend nothing; retreat permits another attempt. Only
  successful resolution settles rewards and local commissions, once.
- Title and dialogue use actual mouse buttons. Portrait dialogue has readable
  physical text sizes and larger options; landscape layouts retain their scale.
- First combat teaches charge, heavy defense and the next counter opportunity.
  Combat logs preserve the main action and key consequence; escape risk is clear.
- Cap all boss healing at a shared25% maximum-HP budget and strengthen learned
  skills from chapter four. Identical imperfect-strategy experiments dropped
  from39 timeouts to0, preserving losses and differences between policies.

Evidence: `screenshots/result/27/`. Latest check/build and246 library tests pass.
Full route:41/41 stages,7/7 seals,fox3/3,musician2/2,VictoryBoth. Native play
covered opening, branching dialogue, first fight, rewards and continued movement;
remaining chapters used internal regression. No recruited-human test was run.

See `docs/playtests/player-review-2026-10-07.md` for actions, results and limits.
Repeated terrain, weak visual location identity, unclear collision art and later
novice difficulty remain actionable design work.
