use chip8_interpreter::Chip8Emulator;
use yew::{prelude::*, virtual_dom::VNode};

#[function_component(App)]
fn app() -> Html {
    let mut emu = Chip8Emulator::new();
    let pong = include_bytes!("../../roms/PONG");
    emu.load_data(pong);
    let display: Vec<VNode> = emu
        .to_string()
        .split("\n")
        .into_iter()
        .map(|row| {
            html! {
                <p>{format!("{row}")}</p>
            }
        })
        .collect();
    html! {
        <>
            <h1>{ "Chip8 Emulator state" }</h1>
            <div>
                {display}
            </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
