use std::{
    fs::File,
    io::{self, Read},
    time::Duration,
};

use chip8::{Chip8Emulator, SCREEN_WIDTH};
use clap::Parser;
use clap::Subcommand;
use itertools::Itertools;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Position, Rect},
    style::Color,
    symbols::Marker,
    widgets::{
        Block, Widget,
        canvas::{Canvas, Points},
    },
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    commands: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    Pong,
    Guess,
    Maze,
}

#[derive(Debug, Default)]
pub struct App {
    emulator: Chip8Emulator,
    points: Vec<Position>,
    exit: bool,
}

fn main() -> io::Result<()> {
    let file_name = match Args::parse().commands {
        Commands::Pong => "PONG",
        Commands::Guess => "GUESS",
        Commands::Maze => "MAZE",
    };
    let mut terminal = ratatui::init();
    let app_result = App::new(file_name).run(&mut terminal);
    ratatui::restore();
    app_result
}

impl App {
    pub fn new(file_name: &str) -> Self {
        let mut emulator = Chip8Emulator::new();
        let mut file = File::open(format!("./src/roms/{file_name}")).expect("Failed to open file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("Failed to read GUESS file");
        emulator.load_data(&buffer);
        App {
            emulator,
            exit: false,
            points: vec![],
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            self.emulator.tick();
            self.emulator.tick_timers();
            self.calculate_points();
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::horizontal([Constraint::Percentage(75), Constraint::Percentage(25)]);
        let [emulator, _] = vertical.areas(frame.area());
        frame.render_widget(self.draw_emu_display(emulator), emulator);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_micros(10))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        }
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char(c) => match c {
                '0'..='9' => {
                    let idx = c as usize - '0' as usize;
                    self.emulator.keypress(idx, true);
                }
                'a'..='f' => {
                    let idx = c as usize - 'a' as usize;
                    self.emulator.keypress(idx, true);
                }
                'A'..='F' => {
                    let idx = c as usize - 'A' as usize;
                    self.emulator.keypress(idx, true);
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn calculate_points(&mut self) {
        let display = self.emulator.get_display();
        let mut points: Vec<Position> = Vec::new();
        for (i, pixel) in display.iter().enumerate() {
            if *pixel {
                let x = (i % SCREEN_WIDTH) as u16;
                let y = (i / SCREEN_WIDTH) as u16;
                points.push(Position::new(x, y));
            }
        }
        self.points = points;
    }

    fn draw_emu_display(&self, area: Rect) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().title("Chip8 Emulator"))
            .marker(Marker::Block)
            .x_bounds([0.0, f64::from(area.width)])
            .y_bounds([0.0, f64::from(area.height)])
            .paint(move |ctx| {
                let points = self
                    .points
                    .iter()
                    .map(|p| {
                        (
                            f64::from(p.x) - f64::from(area.left()),
                            f64::from(area.bottom()) - f64::from(p.y),
                        )
                    })
                    .collect_vec();
                ctx.draw(&Points {
                    coords: &points,
                    color: Color::White,
                });
            })
    }
}
