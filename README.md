# WayPlover

*Steno for Wayland*

<div align="center">
  
  ![Demo](https://via.placeholder.com/400x150)
  
</div>

## Important notes 
- Very workin progress and progress is slow i only mess with this project every blue moon

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
