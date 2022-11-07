# Pico badge, made with 🦀 Rust

A quick weekend project made for a Rust event. Here is a little [video demo](https://odysee.com/@werediver:d/Pico-badge-demo:8).

## Power source

The badge is designed (mechanically, mostly) to be powered off a CR2032 battery. This is a poor choice, because CR2032 capacity heavily regrades under heavier loads (heavier than a fraction of a milliampere, see [High pulse drain impact on CR2032 coin cell battery capacity](https://www.dmcinfo.com/Portals/0/Blog%20Files/High%20pulse%20drain%20impact%20on%20CR2032%20coin%20cell%20battery%20capacity.pdf), Figure 3: CR2032 coin cell continuous discharge patterns).

A better—yet still very limiting—choice would be the bigger CR2450.

After all the optimizations, the badge draws around 13 mA from a 3.3 V source, which is way too stressful for these batteries. The badge operates for around 35 minutes on a single battery, than the MCU (RP2040) ceases it's activity, but the OLED continues to display tha last frame for at least another half an hour.
