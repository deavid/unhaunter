# The problem is a lack of design

- Author: deavid
- Date: 2026-03-30
- Status: Draft / Guide to the problem.

Multiplayer has grown up retrofitted mainly by autonomous AI under my direction. The problem is that I did not want to
spend the time to define all the stuff to the detail - I just wanted to get multiplayer working, however that would look
like. There's an advantage to that: We got to see it the pros and cons. We got to understand problems that I couldn't
even phantom when I was about to start.

But now the problem is that I don't know how multiplayer works at all, and neither do the AI Agents working here. They
love hotfixing, and even though I refactored the shit out of it, still there's no clear direction.

And the lack of a plan, the lack of a direction, is making the codebase rot in pretty ways - pretty, because we have
lots of enforcement via design rules for ECS, so it's not super dirty, but it's rot because the different parts of the
code don't agree on the same direction. We got a ship that is in tension from different sides trying to go into a
different direction.

I want to create a proper Source of Truth in this folder.

Having bevy_replicon allows us to make offline code paths to be almost the same as online code paths, which would
significantly reduce testing surface.

Relevant docs:

- design/
  - 2026-02-14/multiplayer_architecture_tensions.md
  - 2026-02-21/hub_architecture_v2.md

- ddd-analysis/
  - 03_concept_design_v2.md

- replicon_refactor/
  - 07_target_architecture.md
  - 12_true_distributed_authority_refactor.md

- ECS_DOMAIN_MANIFESTO.md

---

## Mental model

We need a model that tells us how stuff should be done, so simple, so strict, so rigid, that a tired engineer can
look at a file and understand what ECS queries are legal and what is not.

We should stop talking about "stop doing X" or "Y is bad, Z is better", and try to get into just a schema to follow.
