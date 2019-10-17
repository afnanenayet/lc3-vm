use log::debug;
use pretty_env_logger;
use std::{
    io::{self, Write},
    path::PathBuf,
};
use structopt::StructOpt;
use termion::raw::IntoRawMode;
use tui::{backend::TermionBackend, Terminal};

mod debugger;
mod lc3;

/// A VM for the LC3 architecture
#[derive(Debug, StructOpt)]
#[structopt(author)]
struct Opt {
    /// The path to an image file for the VM to execute
    #[structopt(parse(from_os_str))]
    pub image_file: PathBuf,

    /// Whether the VM should run with the debugger
    #[structopt(short, long)]
    pub debug: bool,
}

fn main() -> Result<(), io::Error> {
    pretty_env_logger::init();
    let opt = Opt::from_args();
    debug!("Initialized VM");
    let mut vm = lc3::LC3::new();

    if opt.debug {
        let mut stdout = io::stdout().into_raw_mode()?;
        write!(stdout, "{}", termion::clear::All)?;
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let debug_state = debugger::Debugger { vm: &vm };
        debugger::draw(&mut terminal, &debug_state)?;
        // TODO clear after running
    }
    vm.read_image_file(&opt.image_file)?;
    vm.run_loop();
    Ok(())
}
