# WayPlover

*Steno for Wayland*

<div align="center">
  
  ![Demo](https://via.placeholder.com/400x150)
  
</div>


##  Why

Plover did not support wayland so I built this to avoid changing my system to learn about it. plus tui's are badass 

## Description

A Stenography tool for wayland. supports some of the plover dictionary format. 

# Learning

if you interested in learning or a try to understand what this is for please checkout

[The Open Steno Project]([https://www.openstenoproject.org/plover](https://www.openstenoproject.org/plover/))

## Build

`cargo build`

## Install
*download dictionary database from GitHub as well or be ready to provide you custom dictionary*

`cargo install wayplover`

## Caveats 
- by some I mean none but I'm working on it
- the database file is a dump of plover but it only can resolve single strokes. 
- works best with custom dictionary's and not full theory's

## Use Examples 
you don't use steno for all day typing but what simple access to your "abbreviations"  

## Usage
`wayplover --port /dev/ttyACM0 --dictionary plover.db`
## Features
- [x] Output History
- [x] Chord History
- [x] Last Chord Visual
- [x] Dictionary Lookup

## Support
*Only tested with Qmk Keyboard(Planck rev6)*
- [x] Gemini-PR
- [ ] TX-Bolt

## TODO
- [x] feature flag sound
- [ ] output toggle
- [ ] settings ui
- [ ] translations ui
- [ ] Dictonary manager 
- [ ] full dictionary support
