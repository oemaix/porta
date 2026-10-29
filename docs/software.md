# Software

Status: baseline · 2026-10-29

The Core2 application is one Rust image on ESP-IDF v5.5.6 ([D-0046](decisions.md#d-0046-core2-language), [D-0042](decisions.md#d-0042-core2-components)). The tree is `firmware/` ([D-0043](decisions.md#d-0043-firmware-tree)). The modules live in `firmware/src`. This page names them and what crosses between them. Pins stay in [architecture](architecture.md). Listener actions stay in [ui design](ui-design.md).

The panel and the player share that image. Their seam is a set of player commands and one status snapshot. PCM does not cross the seam. cspot is the part that needs the Core2's 8 MB PSRAM ([D-0029](decisions.md#d-0029-spotify-transport)). The panel draws its own color blocks ([D-0011](decisions.md#d-0011-visual-language)). A stored 320×240 frame at 16 bits per pixel is 150 KB and belongs in PSRAM when the renderer keeps one.

```mermaid
flowchart LR
  input[input]
  ui[ui]
  player[player]
  sink[sink]
  net[net]
  snap[status snapshot]

  input -->|four actions| ui
  ui -->|player commands| player
  player -->|playback fields| snap
  net -->|Wi-Fi field| snap
  snap --> ui
  player -->|PCM| sink
```

## Components

| Component | Phase | Scope |
| --- | --- | --- |
| `seam` | 1 | Command, snapshot, and queue declarations. No driver and no task. The queue holds eight commands. |
| `app` | 1 | Starts the others, owns the snapshot and the command queue, and runs the two steps. Holds no screen rules and no codec registers. `firmware_main` calls `App::init`, then the panel supply and one paint. |
| `input` | 1 | Module HMI at I2C `0x41`, the FT6336U, and the three virtual screen buttons. Emits next, previous, activate, and back. Also reports button index 3 and screen-button presses. Leaves the HMI LEDs off. Does not draw. |
| `ui` | 1 | Screen stack, focus index, and the phase 1 screens: Now Playing, Connect, and Settings. Draws on the ILI9342C. Consumes actions. Emits a player command only for a focused playback block. Reads the snapshot. Drops button index 3 ([D-0026](decisions.md#d-0026-second-hmi-button)). Drops screen buttons until a label exists ([D-0034](decisions.md#d-0034-screen-buttons)). |
| `player` | 1 | Accepts play, pause, next, previous, and set volume. Fills title, transport, volume, and session-ready. The phase 1 source is the labelled tone stand-in ([P-0001](pitfalls.md#p-0001-spotify-on-the-esp32)). cspot is not linked. |
| `sink` | 1 | Accepts 16-bit stereo PCM at 44100 or 48000 Hz and resamples between those two rates ([D-0032](decisions.md#d-0032-i2s-format)). `Board::limit_measured` stays false until the AXP192 limit is written down. That does not block hardware I2S ([P-0040](pitfalls.md#p-0040-unmeasured-rail), [P-0006](pitfalls.md#p-0006-internal-i2s)). |
| `net` | 1 | Writes the Wi-Fi field of the snapshot. The station is not started in this tree yet. Starting it before the current limit is measured is allowed ([P-0040](pitfalls.md#p-0040-unmeasured-rail)). |
| `board` | 1 | Internal I2C, the pin map in [architecture](architecture.md), and the AXP192 current limit ([NFR-04](requirements.md)). Enables the panel logic supply and backlight. Does not mount the microSD slot ([D-0025](decisions.md#d-0025-microsd-in-phase-1), [P-0046](pitfalls.md#p-0046-lcd-rail-and-the-microsd-slot)). Does not measure a cell until [D-0033](decisions.md#d-0033-power-architecture) closes. |

`app` may call the others. The others do not include `app`. The command and snapshot types live in `seam` and contain no drivers. `ui` does not include cspot, the ES8388, I2S, or the Wi-Fi driver ([P-0038](pitfalls.md#p-0038-panel-calls-the-player)). `player`'s public surface is the commands, the playback fields, and the calls that send and read them. `app` passes the current block rectangles into `input`.

## Interactions

**Actions.** `input` delivers them to `ui` only. Focus, back, and the stack stay in `ui`. Button index 3 and unbound screen buttons stay there too. In one poll, button and touch actions come before wheel steps, so a press confirms the block that already has focus. Wheel steps that do not fit stay pending. A rising edge stays armed when the action list is full. A volume step is sent only when the value changes, and only when the queue accepts it.

**Player commands.** `ui` sends play, pause, next, previous, and set volume ([D-0029](decisions.md#d-0029-spotify-transport)). Next and previous move focus. Activate on Louder or Quieter sends set volume, ten steps at a time, clamped to 0..100. A tap on an unfocused block is a focus action so the finger can name that block; it is still the same focus index. Zeroconf and stored credentials stay in `player` ([D-0030](decisions.md#d-0030-login)). The Connect screen reads the result from the snapshot. Until cspot is linked, the title is "Tone stand-in" and the session is not ready ([P-0001](pitfalls.md#p-0001-spotify-on-the-esp32)).

**Status snapshot.** One fixed-size record, replaced as a whole. Phase 1 fields are a short title, playing or paused, volume, session ready, and Wi-Fi associated. `app` holds the record. `player` and `net` write their fields. `ui` copies it before drawing.

**PCM.** `player` hands 16-bit stereo frames to `sink`. `sink` clocks them into Module Audio. This path does not enter the snapshot.

**Startup.** `App::init` calls `board` before the steps run. `firmware_main` then enables the panel rails and paints the current screen once. This tree has not started hardware I2S or Wi-Fi yet. Starting them before the current limit is measured is allowed ([P-0040](pitfalls.md#p-0040-unmeasured-rail)). A wheel tick does not pass through `board`.

## Tasks

`App::panel_step` runs `input` and `ui`. `App::audio_step` runs `player` and the software `sink`. Station events will update the Wi-Fi field once a station exists. This tree has not started Wi-Fi yet ([P-0040](pitfalls.md#p-0040-unmeasured-rail)). The steps share the command queue and the snapshot. FreeRTOS priorities and core pinning wait until a build is measured.

## Phase 2 seams

These pieces stay out of the phase 1 image ([D-0023](decisions.md#d-0023-separate-phase-1-prototypes), [P-0012](pitfalls.md#p-0012-phase-creep)).

| Later piece | How it meets the phase 1 seam |
| --- | --- |
| AirPlay 1 source, DLNA source | Each is its own component. PCM goes to `sink`. Snapshot fields are filled where the source has them. Transport commands are accepted where the source has them. |
| Line input | PCM from the ES8388 ADC into `sink`. Auxiliary mode is a source choice on the existing actions. |
| Cable output | Added inside `sink`: the second I2S and Postern's register file, rate, start, stop, and link status ([D-0031](decisions.md#d-0031-board-link)). `ui` does not address the Stamp Pico. |
| Battery field | `board` fills it after [D-0033](decisions.md#d-0033-power-architecture) names the cell. `ui` displays the snapshot. |

New screens use the same four actions ([FR-12](requirements.md)).

## Left open

Task priorities, stack sizes, and the ES8388 register sequence. `Board::limit_measured` stays false until the AXP192 limit is written down, and that does not block I2S or Wi-Fi ([P-0040](pitfalls.md#p-0040-unmeasured-rail)). The Rust module names are the component names above. Panel power and the ILI9342C paint are `board_panel` and `ui_lcd`, compiled only for the ESP-IDF target. The host test does not link them.
