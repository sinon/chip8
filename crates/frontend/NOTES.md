Serve html from Yew with:
`trunk serve --open`

Next steps:
- Investigate:
    - `wasm_bindgen`
    - Look at creating the js file using [requestAnimationFrame](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame) to create the needed render loop for calling `emu.tick()`


Links:

- https://rustwasm.github.io/docs/book/game-of-life/hello-world.html
- https://github.com/rustwasm/create-wasm-app
- https://github.com/rustwasm/wasm-pack-template
- https://yew.rs/docs/getting-started/build-a-sample-app
- https://alexcrichton.github.io/wasm-bindgen/examples/2d-canvas.html