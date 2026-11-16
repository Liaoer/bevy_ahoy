# Ahoy upstream synchronization

- Official repository: https://github.com/janhohenheim/bevy_ahoy
- Official release: `v0.3.0`, commit `10484a4dbb19959b213d7605a51adedec3cd6595`.
- Fork baseline: https://github.com/Liaoer/bevy_ahoy / `33e14510bca6fcea0d540eea4160cd781b4a9ef3`.
- Template extensions originated in https://github.com/olekspickle/bevy_ahoy / `08d1f95b9bcf834389168671eae85eaaaf3f8e48`.
- Synchronization date: 2026-11-16. This remains a fork, not the unmodified crates.io package.

The existing Bevy 0.20 / Avian 0.8 migration already supplies the new AABB margin,
`bevy_shape` import, `SpeculativeCcd`, typed lifecycle observers and matched-query
iteration. Retain the explicit `bevy_shape/bevy_reflect` and serialization feature
forwarding needed by the fork's reflected configuration.

Apply the upstream generic camera constructor and equivalent tic-tac early return;
update optional pickup to Avian Pickup 0.7 and its Holding/Push API. Keep pickup
disabled in the host and generated template. Align example dependencies with the
official Bevy 0.20 release; retain the template camera dependency at its existing
reviewed 0.20 commit. Do not adopt upstream's nightly CI/toolchain or replace host
plugin ownership, schedules, input bindings, tuning, identities or persistence.
Native development uses static linking, matching the host; the upstream dynamic
development feature produced an unnecessarily expensive MSVC dylib link here.

Retain Dashing, AirJumping, WallRunning, JumpGravity and the fork's related state,
input and camera-yank behavior. `tests/movement.rs` is copied from the official
release (six headless 60 Hz movement tests); `tests/extensions.rs` preserves the
existing template's extension, water and serialization regressions. The minimal
template validation package runs these same files rather than a second copy.

The MIT and Apache-2.0 licenses are retained. Validation and remaining platform
limits are recorded by the integrating host; a passing headless test is not a
rendering, audio or browser certification.
