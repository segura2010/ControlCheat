# CONTROL Cheat

It is just a cheat for CONTROL game (https://store.steampowered.com/app/870780/Control_Ultimate_Edition/) written in Rust.

### Features

- Infinite Ammo
- Infinite Health
- Oneshot kills
- Infinite Energy

### Structure/Components

- main.rs -> It is the main source code for the .exe binary. It will allow you to inject the DLL with the cheat to the game process.
- lib.rs -> It is the main source code for the .dll binary. This DLL must be injected (you can use the compiled .exe) in the game to set up the hooks, etc.

### Why?

I want to start to use Rust for future projects, so I need to practise. At the same time I wanted to do some game hacking stuff, so:

Learn Rust + Game Hacking = Rust cheat for a game.

### Video

https://twitter.com/alberto__segura/status/1428036152322367492