---
mode: agent
---

- Copy the ids of cards to implement (including full art versions) in `database.json`.
- In `attack_ids.rs` add the attack to the `AttackId` enum and the `ATTACK_ID_MAP` map (with the correct index). Only implement attacks with effects.
- Review similar attacks in `apply_attack_action.rs` to ensure consistency in implementation. Try to re-use code if possible,
  or refactor existing code to make it reusable.
- Implement the attack logic in `forecast_effect_attack` in `apply_attack_action.rs`. Try to keep
  the logic in the long match clause at a minimum (a 1 or 2 liner that calls a function).
