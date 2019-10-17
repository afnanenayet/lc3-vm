use crate::lc3::{consts::Register, LC3};
/// The functions and files pertaining to the TUI debugger for the VM.
///
/// This provides a way to step through instructions and inspect memory through the execution of a
/// program, allowing the user to either debug the VM or the program.
///
use num_traits::FromPrimitive;
use std::io;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle};
use tui::widgets::{
    Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, List, Marker, Paragraph, Row,
    SelectableList, Sparkline, Table, Tabs, Text, Widget,
};
use tui::{Frame, Terminal};

/// A struct representing the state of the debugging TUI
pub struct Debugger<'a> {
    /// A reference to the VM that is being monitored
    pub vm: &'a LC3,
}

impl<'a> Debugger<'a> {
    pub fn new(vm: &'a LC3) -> Self {
        Debugger { vm }
    }
}

/// The main drawing routine for the UI
pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &Debugger) -> Result<(), io::Error> {
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
            .split(f.size());
        Block::default()
            .title("Registers")
            .borders(Borders::ALL)
            .render(&mut f, chunks[0]);
        Block::default()
            .title("Source")
            .borders(Borders::ALL)
            .render(&mut f, chunks[1]);
        draw_registers(&mut f, &app, chunks[0]);
    })
}

/// Tag with registers, allowing user to monitor what is in each register
fn draw_registers<B: Backend>(f: &mut Frame<B>, app: &Debugger, area: Rect) {
    let num_registers = app.vm.registers.len();
    // Percentage is 100 / # of registers
    let percentage = 100 / (num_registers as u16);
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(percentage); num_registers])
        .direction(Direction::Horizontal)
        .margin(1)
        .split(area);

    // can't use map here because iterators are lazy and it won't execute without collecting a
    // return value from the function
    for (idx, &c) in chunks.iter().enumerate() {
        draw_register(f, &app, c, idx);
    }
}

/// Draw an individual register debugging block
///
/// `register_idx` is the index of the register to print
fn draw_register<B: Backend>(f: &mut Frame<B>, app: &Debugger, area: Rect, register_idx: usize) {
    let register_enum: Register = FromPrimitive::from_usize(register_idx).unwrap();
    let register_name = format!("{:?}", register_enum);
    let register_value = vec![Text::raw(format!("{}", app.vm.registers[register_idx]))];
    Paragraph::new(register_value.iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(&register_name)
                .title_style(Style::default().modifier(Modifier::BOLD)),
        )
        .render(f, area);
}
