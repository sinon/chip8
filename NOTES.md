# TODO Notes

Want to implement a UI using gleam but have the core chip8 interpreter still be driven by the rust lib.

To achieve this:
- [x] broken rust code up into seperate crates (lib and original clap CLI)
- [x] Added `rustler` feature to core lib behind feature flag
- [x] Build the dynamic lib for us in `gleam`
- [ ] Figure out how to expose the relevant parts of the Chip8Emulator to erlang, blog did the nix plumbing manually but `rustler` on erl side might handle this instead?
- [ ] Call the needed methods in gleam to verify:
    - `tick`
    - `tick_timers`
    - `keypress`
    - `get_display`
- [ ] Build a UI in gleam ???
    - Potential big gap here, I had hoped to use `lustre` but that of course leverages the javascript side of `gleam` not the erlang side where I have the rust FFI available. I can't seem to find anything about WASM on gleam/lustre which would be the obvious replacement to `rustler`


## Resources

- https://www.jonashietala.se/blog/2024/01/11/exploring_the_gleam_ffi/
- https://docs.rs/rustler/latest/rustler/attr.resource_impl.html
- https://hexdocs.pm/rustler/readme.html