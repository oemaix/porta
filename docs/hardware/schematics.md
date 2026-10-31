# Schematics

Status: baseline · 2026-10-31

These are the circuit prints M5 publishes for the boards in [architecture](../architecture.md). They were saved from the product pages on 2026-10-31 so implementation can use a local copy. They remain M5Stack documents. The Apache license of this repository does not apply to them.

The pages link these prints. For the Stamp Pico the print is one image. The Altium sheet sources are not on those pages. This directory keeps the circuit prints.

| Part | SKU | File | Product page |
| --- | --- | --- | --- |
| Core2 v1.3, core section | K010-V13 | [core2-v1.3-core-section.pdf](core2-v1.3-core-section.pdf) | [Core2 v1.3](https://docs.m5stack.com/en/core/Core2_v1.3) |
| Core2 v1.3, expansion board | K010-V13 | [core2-v1.3-expansion.pdf](core2-v1.3-expansion.pdf) | [Core2 v1.3](https://docs.m5stack.com/en/core/Core2_v1.3) |
| Module Audio | M144 | [module-audio.pdf](module-audio.pdf) | [Module Audio](https://docs.m5stack.com/en/module/Module_Audio) |
| Module HMI | M129 | [module-hmi.pdf](module-hmi.pdf) | [Module HMI](https://docs.m5stack.com/en/module/Module_HMI) |
| M5GO Bottom2 v1.3 | A014-C-V13 | [m5go-bottom2-v1.3.pdf](m5go-bottom2-v1.3.pdf) | [M5GO Bottom2 v1.3](https://docs.m5stack.com/en/base/Base_M5GO_Bottom2_v1.3) |
| M5Stamp Pico | C050-B | [stamp-pico.webp](stamp-pico.webp) | [Stamp Pico](https://docs.m5stack.com/en/core/Stamp-Pico) |

The Core2 page splits the print in two. The core-section file is the download M5 names `CORE2_V1.0_SCH.pdf` and labels "Core2 v1.3 Core Section Schematics". The expansion file is the download dated 2025-11-14 in its URL, labeled "Core2 v1.3 Expansion Board Section Schematics". Read them as one board. [D-0003](../decisions.md#d-0003-host-board).

The Bottom2 print's sheet title is `M5GO Bottom2_SCH_Main_V1.31.SchDoc`. That is the file linked from the v1.3 product page. [D-0006](../decisions.md#d-0006-base).

The Stamp Pico page shows the schematic as one image, not a PDF. The WebP is that image. [D-0007](../decisions.md#d-0007-bluetooth-transmitter).

Module Proto (SKU M001) has a mechanical drawing on its [product page](https://docs.m5stack.com/en/module/Module_Proto), and an Altium board file, `Module_Proto_Board.PcbDoc`, under [M5_Hardware](https://github.com/m5stack/M5_Hardware/tree/master/Products/M001_Module_Proto/Footprints). The page has no circuit print, so this directory has none for that module. [D-0007](../decisions.md#d-0007-bluetooth-transmitter).
