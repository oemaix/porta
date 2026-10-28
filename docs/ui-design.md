# UI design

Status: baseline · 2026-10-28

The panel is the Core2 v1.3 screen, 320×240, capacitive touch, plus three virtual buttons along the bottom edge. The Module HMI adds a wheel, the wheel press, and two side buttons. Those inputs share one action set ([D-0010](decisions.md#d-0010-interaction-model)).

## Actions

| Action | Wheel | Touch | HMI button 2 | HMI button 3 | Screen buttons |
| --- | --- | --- | --- | --- | --- |
| Next / previous focus | Rotate | Tap an unfocused block (moves focus there) | — | — | Whatever the bottom label says ([D-0034](decisions.md#d-0034-screen-buttons)) |
| Activate | Press | Tap the focused block | — | — | Whatever the bottom label says |
| Back | — | — | Press ([D-0009](decisions.md#d-0009-hmi-mapping)) | — | Whatever the bottom label says |

Button index 3 has no command ([D-0026](decisions.md#d-0026-second-hmi-button)). The API still reports it, so a later binding does not require a new input path. On the root screen, Back does nothing visible.

The three screen buttons are not bound yet. The current idea, still open, is that each one does what the label at the bottom of the screen says ([D-0034](decisions.md#d-0034-screen-buttons)).

Touch targets sit inside the panel. The Core2's edges can report touch poorly ([P-0016](pitfalls.md#p-0016-touch-edges)).

## Visual language

Screens are stacks of flat blocks: a title, a focused block, and a short list. Type is large. One block is focused. The focused block is the thing Activate will run.

The look is not a Blackberry skin and not a desktop port ([P-0021](pitfalls.md#p-0021-blackberry-pixels)). The palette is not decided. Prototypes may use the interim values in [D-0035](decisions.md#d-0035-palette) and must be able to drop them.

## Phase 1 screens

The phase 1 set is small and must be finishable with the wheel alone ([FR-05](requirements.md)):

| Screen | Purpose |
| --- | --- |
| Now Playing | Title, play or pause, volume |
| Connect | Wi-Fi state, and the phase 1 login step once [D-0030](decisions.md#d-0030-login) exists |
| Settings | Entries that exist in phase 1 only. Battery is absent until phase 2 |

The screen module owns the stack. Adapters for the encoder, the buttons, and touch translate hardware into the four actions. Widgets do not read I2C themselves.

## Phase 2

Auxiliary mode, AirPlay, DLNA, and battery are new screens or new blocks on these screens. They call the same actions. Elaboration is density and feedback, not a new navigation model ([FR-12](requirements.md)).

## HMI LEDs

The module has two LEDs. No meaning is assigned. Leave them off until a decision gives them a job.
