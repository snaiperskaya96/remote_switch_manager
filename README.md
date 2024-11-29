# Remote Switch Manager 

Remote Switch Manager (RMS) is a badly named application used to interact with network-able devices that have the ability of being switched on and off.

The frontend has heavenly produced by our overlord AI, as I can't be bothered to spend days learning what's the current everchanging new way of making UI and what shiny new library to use.
Backend is 95% human made.

## Features
 - Remotely turn on and off networked switches.
 - Set timers on which a device will be switched on/off.

## Support
Right now it only supports Shelly Gen2 APIs. I'll most likely add Tasmota and SONOFF DIY support at some point soon as I have a few of those around the house.

## Limitations
Those might or might not change in the future, dependently on how fast I'll forget about this application.
  - No UI for adding a new user.
  - No per-user permissions.

## Cross-Compilation
Currently being cross-compiled for `armv7-unknown-linux-musleabi` and it works just fine using [cross](https://github.com/cross-rs/cross)

```bash
cross build --target armv7-unknown-linux-musleabi --release
```

## Notes
Any stored information is saved into a toml file. 

I know, I know, but at the time I wanted something easily configurable without the need to write frontend and backend logic to add/remove/modify them.

Honestly I'm not sure I would go for TOML again after this but hey.

Admin user is generated if no users.toml is found, password is printed to stdout for you to shiver about it.
