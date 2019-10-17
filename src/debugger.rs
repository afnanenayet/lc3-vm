/// The functions and files pertaining to the TUI debugger for the VM.
///
/// This provides a way to step through instructions and inspect memory through the execution of a
/// program, allowing the user to either debug the VM or the program.
use crate::lc3::consts::Op;
use crate::lc3::{consts::Register, DispatchTables, LC3};
use num_traits::FromPrimitive;
use std::io;
use std::rc::Rc;
use termion::event::Event;
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
            .title("Instructions")
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
    let register_value = vec![Text::raw(format!("{:b}", app.vm.registers[register_idx]))];
    Paragraph::new(register_value.iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(&register_name)
                .title_style(Style::default().modifier(Modifier::BOLD)),
        )
        .render(f, area);
}

/// Maintains a list of the instruction/opcode history and displays the next one to the user
fn draw_instr_history<B: Backend>(f: &mut Frame<B>, app: &Debugger, area: Rect) {
    // Convert all of the items in the history to the proper `Text` type. Every item is stylized
    // normally, except for the last item, which is the item that is ABOUT to be executed (the
    // current op), which is in bold.
    //let text: Vec<Vec<String>> = app
    //.op_history
    //.iter()
    //.enumerate()
    //.map(|(idx, s)| vec![format!("{}", idx), s.clone()])
    //.collect();
    let headers = ["Tick", "Opcode"];
    let text = vec![vec!["test", "test"], vec!["test", "test"]];
    let rows = text.iter().enumerate().map(|(idx, item)| {
        if idx == app.op_history.len() - 1 {
            Row::StyledData(item.iter(), Style::default().modifier(Modifier::BOLD))
        } else {
            Row::StyledData(item.iter(), Style::default())
        }
    });
    // There's no point trying to render a widget if there's nothing to render
    let rects = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(2)
        .split(area);
    Table::new(headers.into_iter(), rows)
        .block(Block::default())
        .render(f, rects[0]);
}
