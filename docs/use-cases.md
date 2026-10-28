# Use cases

Status: baseline · 2026-10-28

The actor is the listener. Phase 1 cases are proved on separate prototypes ([D-0023](decisions.md#d-0023-separate-phase-1-prototypes)). Phase 2 cases are one product.

## UC-01 Play Wi-Fi audio on headphones

- **Phase:** 1
- **Preconditions:** The Core2 is on a Wi-Fi network. A Premium account has handed the device a Connect session ([D-0029](decisions.md#d-0029-spotify-transport)).
- **Main flow:** The listener opens Now Playing, starts playback, and hears audio on the Module Audio headphone jack.
- **Extensions:** Pause and resume use the same confirm action as any other focused control. Volume is a focused control, not a hidden gesture.

## UC-02 Send I2S audio to Bluetooth headphones

- **Phase:** 1, as its own prototype
- **Preconditions:** Headphones are paired to the Stamp Pico. An I2S source is connected to that board.
- **Main flow:** Audio present on the I2S input is transmitted with SBC. AAC is used when the link and the encoder both support it.
- **Extensions:** If AAC setup fails, the link stays up on SBC ([D-0007](decisions.md#d-0007-bluetooth-transmitter)).

## UC-03 Move and confirm with the wheel

- **Phase:** 1
- **Preconditions:** A screen with more than one block is showing.
- **Main flow:** Turning the wheel moves the focus. Pressing the wheel activates the focused block.
- **Extensions:** Touch does not invent a different action. A tap focuses a block; a tap on the focused block activates it ([D-0010](decisions.md#d-0010-interaction-model)).

## UC-04 Leave a screen

- **Phase:** 1
- **Preconditions:** A screen is above the root screen.
- **Main flow:** The listener presses HMI button index 2 and returns to the previous screen.
- **Extensions:** On the root screen the same button does not change mode and does not bind a new command ([D-0009](decisions.md#d-0009-hmi-mapping)).

## UC-05 Bridge the line input

- **Phase:** 2, auxiliary mode
- **Preconditions:** A source is plugged into the TRS microphone jack. Bluetooth headphones are connected.
- **Main flow:** The listener selects the line bridge. Audio from the jack is heard on the headphones.
- **Extensions:** The TRRS jack is not the line input ([P-0008](pitfalls.md#p-0008-trs-jack)).

## UC-06 Receive AirPlay

- **Phase:** 2
- **Preconditions:** The sender and Porta are on the same network. The sender uses AirPlay 1, which carries ALAC ([D-0036](decisions.md#d-0036-airplay-and-dlna)).
- **Main flow:** The listener starts AirPlay on a sender. Porta plays it on the headphone jack and on Bluetooth.

## UC-07 Receive DLNA

- **Phase:** 2
- **Preconditions:** Same network. The item is 16-bit stereo LPCM, WAV PCM, or ALAC at 44.1 or 48 kHz ([D-0036](decisions.md#d-0036-airplay-and-dlna)).
- **Main flow:** A controller gives Porta the URI. Porta fetches it and plays it on the headphone jack and on Bluetooth.

## UC-08 Stop before the battery dies

- **Phase:** 2
- **Preconditions:** The power architecture in [D-0033](decisions.md#d-0033-power-architecture) names which cell is measured.
- **Main flow:** The listener can see the level. At the low threshold the device stops playback and powers off instead of browning out.
