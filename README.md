# WayPlover
*Steno for Wayland*

## Description
A Steno tool for wayland. supports some of the plover dictionary
#learing
[Open Steno Project](https://www.openstenoproject.org/plover/)

[Learn Plover]()
## build

Requires libsound2-dev libspeechd-dev
```

```

## Install
Requires Alsa/Pulse(needed beeps and boops)& speech-dispacther(need for text to speech)
is also needs a dictionary file which can be made simply or borrowed from the Plover Project

```
//install speech-dispacther
pacman -S speech-dispacther espeak-ng
systemctl enable --now speech-dispacther
```


## Usage
`wayplover --port /dev/ttyACM0 --dictionary plover.json`

## Features
- [x] Output History
- [x] Chord History
- [x] Last Chord Visual
- [ ] Dictionary Lookup

## Support
*Only tested with Qmk Keyboard(Planck rev6)*
- [x] Gemini-PR
- [ ] TX-Bolt

## TODO
- [ ] feature flag sound
- [ ] feature flag tts

- [ ] full dictionary support
    - [ ] fuzzy hashmap
    - [ ] prefix strokes
