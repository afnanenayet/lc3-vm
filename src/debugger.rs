/// The functions and files pertaining to the TUI debugger for the VM.
///
/// This provides a way to step through instructions and inspect memory through the execution of a
/// program, allowing the user to either debug the VM or the program.
use crate::lc3::{consts::Register, DispatchTables, LC3};
use num_traits::FromPrimitive;
use std::io;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Row, SelectableList, Table, Widget};
use tui::{Frame, Terminal};

/// A struct representing the state of the debugging TUI
pub struct Debugger<'a> {
    /// A reference to the VM that is being monitored
    pub vm: &'a mut LC3,

    /// A list of opcodes that have been executed so far
    op_history: Vec<String>,
}

impl<'a> Debugger<'a> {
    pub fn new(vm: &'a mut LC3) -> Self {
        let next_op = format!("{:?}", vm.parse_next_op());
        Self {
            vm,
            op_history: vec![next_op],
        }
    }

    /// Perform an event tick on the debugger
    ///
    /// This performs an iteration on the VM. It will move forward the instruction by one step.
    pub fn tick(&mut self, tables: &DispatchTables) {
        self.vm.step(&tables);
        let next_op = format!("{:?}", self.vm.parse_next_op());
        self.op_history.push(next_op);
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
            .title("Execution")
            .borders(Borders::ALL)
            .render(&mut f, chunks[1]);
        draw_registers(&mut f, &app, chunks[0]);
        draw_instr_history(&mut f, &app, chunks[1]);
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
    // return value from the function, so we have to iterate through the loop
    for (idx, &c) in chunks.iter().enumerate() {
        draw_register(f, &app, c, idx);
    }
}

/// Draw an individual register debugging block
///
/// `register_idx` is the index of the register to print. This UI element prints the value of the
/// register in binary, hex, and as a regular integer in a vertical table.
fn draw_register<B: Backend>(f: &mut Frame<B>, app: &Debugger, area: Rect, register_idx: usize) {
    let register_value = app.vm.registers[register_idx];

    // A vector with the different representations of the register value to print in the register
    // debugger
    let register_strings = vec![
        format!("{:b}", register_value),
        format!("{:x}", register_value),
        format!("{}", register_value),
    ];
    let register_enum: Register = FromPrimitive::from_usize(register_idx).unwrap();
    let register_name = format!("{:?}", register_enum);

    SelectableList::default()
        .block(
            Block::default()
                .title(&register_name)
                .title_style(Style::default().modifier(Modifier::BOLD))
                .borders(Borders::ALL),
        )
        .items(&register_strings)
        .render(f, area);
}

/// Maintains a list of the instruction/opcode history and displays the next one to the user
fn draw_instr_history<B: Backend>(f: &mut Frame<B>, app: &Debugger, area: Rect) {
    let headers = ["Tick", "Instruction"];

    // We need to create a vector that owns the strings so that we can reference them with
    // iterators for the table
    let row_data: Vec<Vec<String>> = app
        .op_history
        .iter()
        .enumerate()
        .map(|(idx, item)| vec![format!("{}", idx), item.clone()])
        .collect();
    // Make the last instruction (the current one) bold
    let rows = row_data.iter().rev().enumerate().map(|(idx, item)| {
        let style = if idx == 0 {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };
        Row::StyledData(item.iter(), style)
    });
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(1)
        .split(area);
    Table::new(headers.iter(), rows)
        .header_style(Style::default().modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::NONE))
        .widths(&[10, 10])
        .column_spacing(2)
        .render(f, rects[0]);
}
