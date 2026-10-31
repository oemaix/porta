# Use cases

Status: baseline · 2026-10-31

The actor is the listener. Phase 1 is the Core2 player alone. Postern stays a separate image until phase 2 ([D-0023](decisions.md#d-0023-separate-phase-1-prototypes)).

## UC-01 Play Wi-Fi audio on headphones

- **Phase:** 1
- **Preconditions:** The Core2 is on a Wi-Fi network. A Premium account has handed the device a Connect session ([D-0029](decisions.md#d-0029-spotify-transport)).
- **Main flow:** The listener opens Now Playing, starts playback, and hears audio on the Module Audio headphone jack.
- **Extensions:** Pause and resume use the same confirm action as any other focused control. Volume is one block. Its sides step the level. The middle shows the number.

## UC-02 Send audio to Bluetooth headphones

- **Phase:** 2
- **Preconditions:** The Core2 is linked to the Stamp Pico ([D-0031](decisions.md#d-0031-board-link)). A headphone is in range.
- **Main flow:** The listener connects that headphone from the UI. Audio from the product is transmitted with SBC. AAC is used when the link and the encoder both support it. The listener can remove the headphone.
- **Extensions:** If AAC setup fails, the link stays up on SBC ([D-0007](decisions.md#d-0007-bluetooth-transmitter)). The headphone is the A2DP sink. The Core2 does not receive Bluetooth audio.

## UC-03 Move and confirm with the wheel

- **Phase:** 1
- **Preconditions:** A screen with more than one block is showing.
- **Main flow:** Turning the wheel moves the focus. Pressing the wheel activates the focused block.
- **Extensions:** Touch does not invent a different action. A tap focuses a block; a tap on the focused block activates it ([D-0010](decisions.md#d-0010-interaction-model)). On the transport and volume blocks, the left and right thirds are the side actions drawn on that block. The wheel still moves focus.

## UC-04 Leave a screen

- **Phase:** 1
- **Preconditions:** A screen is above the root screen.
- **Main flow:** The listener presses HMI BtnA and returns to the previous screen.
- **Extensions:** On the root screen the same button does not change mode and does not bind a new command ([D-0009](decisions.md#d-0009-hmi-mapping)).

## UC-05 Bridge the line input

- **Phase:** 2, auxiliary mode
- **Preconditions:** A source is plugged into the TRS microphone jack. Bluetooth headphones are connected.
- **Main flow:** The listener selects the line bridge. Audio from the jack is heard on the headphones.
- **Extensions:** The TRRS jack is not the line input ([P-0008](pitfalls.md#p-0008-trs-jack)).

## UC-06 Receive AirPlay

- **Phase:** 3
- **Preconditions:** The sender and Porta are on the same network. The sender uses AirPlay 1, which carries ALAC ([D-0036](decisions.md#d-0036-airplay-and-dlna)).
- **Main flow:** The listener starts AirPlay on a sender. Porta plays it on the headphone jack and on Bluetooth.

## UC-07 Receive DLNA

- **Phase:** 3
- **Preconditions:** Same network. The item is 16-bit stereo LPCM, WAV PCM, or ALAC at 44.1 or 48 kHz ([D-0036](decisions.md#d-0036-airplay-and-dlna)).
- **Main flow:** A controller gives Porta the URI. Porta fetches it and plays it on the headphone jack and on Bluetooth.

## UC-08 Stop before the battery dies

- **Phase:** 3
- **Preconditions:** The power architecture in [D-0033](decisions.md#d-0033-power-architecture) names which cell is measured. The listener can already see a level ([UC-09](#uc-09-see-the-power-level)).
- **Main flow:** At the low threshold the device stops playback and powers off instead of browning out.

## UC-09 See the power level

- **Phase:** 2
- **Preconditions:** The power architecture in [D-0033](decisions.md#d-0033-power-architecture) names which cell is measured.
- **Main flow:** The listener sees a simple power level on the screen.
- **Extensions:** The level does not power the device off. That behaviour is [UC-08](#uc-08-stop-before-the-battery-dies).

## UC-10 Wake a dark panel

- **Phase:** 2
- **Preconditions:** Audio is playing. No one has operated the device for the idle period.
- **Main flow:** The LCD turns off. The audio stream keeps playing. A Module HMI control, the touchscreen, or a touch button turns the panel back on.
- **Extensions:** Waking the panel does not restart playback and does not change the source. The Module Audio LEDs keep showing the playing source while the panel is dark ([UC-11](#uc-11-see-which-source-is-playing)).

## UC-11 See which source is playing

- **Phase:** 2
- **Preconditions:** Module Audio is on the stack. A source may be playing. The panel may already be dark ([UC-10](#uc-10-wake-a-dark-panel)).
- **Main flow:** The listener looks at the three LEDs on the side of Module Audio. They are dark when nothing is playing and when playback is paused. They glow green while a Spotify Connect session is playing, and red while the TRS source is playing. The color stays while the panel is asleep.
- **Extensions:** AirPlay and DLNA glow blue. That pattern waits until those sources exist in phase 3 ([D-0047](decisions.md#d-0047-module-audio-leds)). The tone stand-in stays dark ([P-0001](pitfalls.md#p-0001-spotify-on-the-esp32)). The Module HMI LEDs stay off. The lamps do not say whether the output is the jack or Bluetooth.
