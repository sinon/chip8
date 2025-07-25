#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use std::{io, time::Duration};

use chip8_interpreter::{Chip8Emulator, SCREEN_WIDTH};
use clap::Parser;
use clap::Subcommand;
use itertools::Itertools;
use ratatui::crossterm::event::KeyboardEnhancementFlags;
use ratatui::crossterm::event::PushKeyboardEnhancementFlags;
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
pub enum Commands {
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
    let command = Args::parse().commands;
    let mut terminal = ratatui::init();
    ratatui::crossterm::execute!(
        io::stderr(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;
    let app_result = App::new(&command).run(&mut terminal);
    ratatui::restore();
    app_result
}

impl App {
    #[must_use]
    pub fn new(command: &Commands) -> Self {
        let pong = include_bytes!("../../roms/PONG");
        let guess = include_bytes!("../../roms/GUESS");
        let maze = include_bytes!("../../roms/MAZE");
        let mut emulator = Chip8Emulator::new();
        match command {
            Commands::Pong => emulator.load_data(pong),
            Commands::Guess => emulator.load_data(guess),
            Commands::Maze => emulator.load_data(maze),
        }
        Self {
            emulator,
            exit: false,
            points: vec![],
        }
    }
    /// # Errors
    /// - reading events
    /// - fails to draw state to terminal
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            for _ in 0..10 {
                self.emulator.tick();
                self.calculate_points();
            }
            self.emulator.tick_timers();
            self.handle_events()?;
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::horizontal([Constraint::Percentage(75), Constraint::Percentage(25)]);
        let [emulator, _] = vertical.areas(frame.area());
        frame.render_widget(self.draw_emu_display(emulator), emulator);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key_event) = event::read()? {
                let pressed = key_event.kind == KeyEventKind::Press;
                self.handle_key_event(key_event, pressed);
            }
        }
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent, pressed: bool) {
        if key_event.code == KeyCode::Esc {
            self.exit();
        }
        let x = match key_event.code {
            KeyCode::Char('1') => Some(0x1),
            KeyCode::Char('2') => Some(0x2),
            KeyCode::Char('3') => Some(0x3),
            KeyCode::Char('4') => Some(0xC),
            KeyCode::Char('q' | 'Q') => Some(0x4),
            KeyCode::Char('w' | 'W') => Some(0x5),
            KeyCode::Char('e' | 'E') => Some(0x6),
            KeyCode::Char('r' | 'R') => Some(0xD),
            KeyCode::Char('a' | 'A') => Some(0x7),
            KeyCode::Char('s' | 'S') => Some(0x8),
            KeyCode::Char('d' | 'D') => Some(0x9),
            KeyCode::Char('f' | 'F') => Some(0xE),
            KeyCode::Char('z' | 'Z') => Some(0xA),
            KeyCode::Char('x' | 'X') => Some(0x0),
            KeyCode::Char('c' | 'C') => Some(0xB),
            KeyCode::Char('v' | 'V') => Some(0xF),
            _ => None,
        };

        if let Some(idx) = x {
            self.emulator.keypress(idx, pressed);
        }
    }

    const fn exit(&mut self) {
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
