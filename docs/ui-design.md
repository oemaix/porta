# UI design

Status: baseline · 2026-11-01

The panel is the Core2 v1.3 screen, 320×240, capacitive touch, plus three virtual buttons along the bottom edge. The Module HMI adds a wheel, the wheel press, and two side buttons. Those inputs share one action set ([D-0010](decisions.md#d-0010-interaction-model)).

## Actions

| Action | Wheel | Touch | HMI BtnB | HMI BtnA | Screen buttons |
| --- | --- | --- | --- | --- | --- |
| Next / previous focus | Rotate | Tap an unfocused block (moves focus there) | — | — | Whatever the bottom label says ([D-0034](decisions.md#d-0034-screen-buttons)) |
| Activate | Press | Tap the focused block | — | — | Whatever the bottom label says |
| Back | — | — | — | Press ([D-0009](decisions.md#d-0009-hmi-mapping)) | Whatever the bottom label says |

BtnB has no command ([D-0026](decisions.md#d-0026-second-hmi-button)). The API still reports it, so a later binding does not require a new input path. On the root screen, Back does nothing visible.

A tap on the focused transport or volume block uses the third under the finger. The middle is Activate for that block. The left and right thirds are the actions drawn as `<` and `>`. The wheel still moves the highlight.

The three screen buttons are not bound yet. The current idea, still open, is that each one does what the label at the bottom of the screen says ([D-0034](decisions.md#d-0034-screen-buttons)).

Touch targets sit inside the panel. The Core2's edges can report touch poorly ([P-0016](pitfalls.md#p-0016-touch-edges)).

## Visual language

Screens are stacks of flat blocks: a title, a focused block, and a short list. Type is large. Block labels are centered in the block. One block is focused. The focused block is the thing Activate will run.

The look is not a Blackberry skin and not a desktop port ([D-0011](decisions.md#d-0011-visual-language)). The palette is not decided. Prototypes may use the interim values in [D-0035](decisions.md#d-0035-palette) and must be able to drop them.

## Phase 1 screens

The phase 1 set is small and must be finishable with the wheel alone ([FR-05](requirements.md)):

| Screen | Purpose |
| --- | --- |
| Now Playing | Title, one transport block, one volume block, and the way into Connect and Settings. The transport word is Play or Pause, with `<` and `>` for previous and next. The volume word is `vol` and the level, with `<` and `>` for quieter and louder. There is no volume line under the title. |
| Connect | Wi-Fi up or down. With an address and no session, the phone picks the device named in `.env` (`porta` when that key is empty). After Zeroconf the second line is the track ([D-0030](decisions.md#d-0030-login)) |
| Settings | Entries that exist in phase 1 only. The power level is absent until phase 2 |

`ui` owns the stack. `input` translates the encoder, the buttons, and touch into the four actions. Widgets do not read I2C. A player command leaves `ui` only when the focused block is a playback control ([software](software.md)).

## Phase 2

Connecting and removing a Bluetooth headphone, the TRS source, and the power level are new blocks on these screens. They call the same actions. After the idle period the panel is dark and the same actions wake it ([UC-10](use-cases.md#uc-10-wake-a-dark-panel)). While the panel is dark, the playing source stays visible on the Module Audio LEDs ([UC-11](use-cases.md#uc-11-see-which-source-is-playing)).

## Phase 3

AirPlay, DLNA, and the elaborated layout are new screens or new blocks on these screens. Elaboration is density and feedback, not a new navigation model ([FR-12](requirements.md)).

## HMI LEDs

The module has two LEDs. No meaning is assigned. Leave them off until a decision gives them a job. Source status is the Module Audio LEDs, not these ([D-0047](decisions.md#d-0047-module-audio-leds)).

## Module Audio LEDs

Three lamps on the side of Module Audio show the playing source from phase 2. Phase 1 leaves them off. Idle and pause are dark, Spotify Connect is green, and the TRS source is red. AirPlay and DLNA are blue once those sources exist. The colors, the brightness, and the registers are [D-0047](decisions.md#d-0047-module-audio-leds).
