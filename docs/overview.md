# Overview

Status: baseline · 2026-10-28

Porta is a handheld audio appliance. A listener uses a wheel, two hardware buttons, a touchscreen, and three on-screen buttons to control it. Sound leaves through a 3.5 mm headphone output and through Bluetooth headphones. Sound can enter from Wi-Fi or from a 3.5 mm input.

Three jobs define the product ([D-0002](decisions.md#d-0002-product-functions)):

1. **Spotify player.** Audio that originates as Spotify arrives over Wi-Fi and is played on the headphone jack and on Bluetooth.
2. **Line bridge.** Audio from the 3.5 mm input is sent to Bluetooth headphones.
3. **Network bridge.** AirPlay 1 (ALAC) and DLNA (LPCM, WAV PCM, ALAC) arrive over Wi-Fi and play on the headphone jack and on Bluetooth ([D-0036](decisions.md#d-0036-airplay-and-dlna)).

The host is an M5Stack Core2 v1.3. Analog jacks are the M5Stack Module Audio. The wheel is the M5Stack Module HMI. Power and the rear base, once modules are stacked, are the M5GO Bottom2 v1.3. Bluetooth transmission is a second board, an M5Stamp Pico on an M5Stack Module Proto, so the Core2 radio can stay on Wi-Fi ([D-0008](decisions.md#d-0008-radio-split)).

Phase 1 builds two separate prototypes: Wi-Fi to I2S playback on the Core2, and I2S to Bluetooth on the Stamp Pico, plus a simple UI that is complete enough to use. Phase 2 joins the bridge functions into the product, adds AirPlay and DLNA, improves login, and adds battery behaviour. See [roadmap](roadmap.md).

Spotify playback is Connect on the Core2 ([D-0029](decisions.md#d-0029-spotify-transport)). The phone chooses the music. The Core2 downloads and decodes the track. A desktop Spotify client still does not run on this chip ([P-0001](pitfalls.md#p-0001-spotify-on-the-esp32)).

Read [architecture](architecture.md) for the boards and [ui design](ui-design.md) for the controls.
