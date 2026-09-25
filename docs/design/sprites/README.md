# Component sprites

Bredboard draws breadboard components as 8-bit pixel art generated in code
(`crates/app/src/sprites/`). There are no image files: each body sprite is a
small pixel grid built at scene spawn and uploaded as a nearest-filtered Bevy
image. The `.txt` files in this directory are golden references of the body
sprites, one character per pixel, using the legend in
`crates/app/src/sprites/palette.rs` (`.` is transparent).

The design was approved on 2026-09-25 from a canvas mockup, then revised the
same day to compact resistor and LED bodies (owner-only link:
<https://claude.ai/artifact/HM3Apo7m4PoRJLLGChX7eu>). The Rust code and these
references are authoritative; the mockup is not needed to implement or review.

## Style rules

- Hole pitch is 8 art pixels. One art pixel is 2 world units, so the board uses
  a square 16-unit pitch in both directions.
- Every body has a 1 px `OUTLINE` (#1b1b24) edge. Light comes from the top
  left: a highlight row or pixels on the upper/left side, a darker shade on the
  lower/right side.
- Colours come only from `palette.rs`. Add a new colour there with a unique
  legend symbol before using it.
- Bodies are drawn unrotated with the first axis pin on the left. Placement
  rotates them in quarter turns, so sprites stay pixel-exact.
- Leads are not part of a body. `sprites::place` draws a 2 px silver lead from
  every pin's hole to the body and darkens the pixel where the lead enters the
  hole, so any valid hole layout works.
- Keep bodies smaller than their pin span wherever possible so the legs and the
  holes they enter stay visible.
- All states of one part have the same size so the app can swap images in place.
- Runtime appearance changes only through `PartArt::state`, which reads
  presentation inputs derived from core state (LED current, control state).
  Sprites never influence electrical results.

## Current parts

| Kind | Body | States | Notes |
| --- | --- | --- | --- |
| `resistor` | 14 x 8 dog-bone | 1 | Compact so leads and holes stay visible on a 3-hole span. Four 1 px bands: two significant digits, multiplier, gold tolerance, computed from `resistance`. |
| `led` | 20 x 20 (12 px dome plus halo room) | off, dim, lit | Compact dome; rim flat marks the cathode. State from calculated current: dim at 0.5 mA, lit at 5 mA. Lit adds a dithered halo. |
| `momentary_button` | 10 x 10 | released, pressed | Small plate between the pins so both legs and holes stay visible. Pressed shrinks the cap and removes highlight and shadow. |

`dc_voltage_source`, `capacitor`, `npn_transistor`, and `changeover_switch`
still use the plain fallback drawing in `crates/app/src/main.rs`.

## Adding a component sprite

1. Create `crates/app/src/sprites/<part>.rs` with a unit struct implementing
   `PartArt`: `axis_pins` (two pin names defining left-to-right), optional
   `state_count` and `state`, and `body` returning the unrotated sprite.
2. Register it in `sprites::art_for` and remove the kind from the fallback arm.
3. Add golden cases to `body_sprites_match_design_references`, generate them
   with `BREDBOARD_BLESS_SPRITES=1 cargo test -p bredboard-app --locked`, and
   review the new `.txt` files here.
4. Extend the placement property test and the same-size test with the new part.
5. Check the part in the Linux executable in every visual state.
