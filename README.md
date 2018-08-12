# cuivre

[crates.io](https://crates.io/crates/cuivre)

A WIP game engine for Rust.

Goals:
 - Non-intrusive: you decide what you call and when you call it. No need to build you whole application around the engine.
 - Graphics layer over SDL2 and OpenGL with features such as automatic sprite instancing, while still allowing custom shaders and the like.
 - Centralized resource management: load text, images, shaders, serialized objects ([ron](https://github.com/ron-rs/ron)+[serde](https://github.com/serde-rs/serde))

Note: since this is based on SDL, you will need to place SDL2.dll next to your executable before launching it. A build.rs script can be useful for this (see example in my (WIP) example game repository: [rust-chess](https://github.com/alexandrejanin/rust-chess))
