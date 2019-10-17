use log::debug;
use pretty_env_logger;
use std::{
    io::{self, Read, Write},
    path::PathBuf,
};
use structopt::StructOpt;
use termion::{raw::IntoRawMode, AsyncReader};
use tui::{backend::TermionBackend, Terminal};

mod debugger;
mod lc3;

use debugger::Debugger;

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
    let tables = lc3::DispatchTables::new();

    if opt.debug {
        let mut stdout = io::stdout().into_raw_mode()?;
        write!(stdout, "{}", termion::clear::All)?;
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut debug_state = Debugger::new(&mut vm);
        let mut reader = termion::async_stdin();
        let mut buf = String::new();

        loop {
            debugger::draw(&mut terminal, &debug_state)?;

            // get next key and perform appropriate action
            buf.clear();
            reader.read_to_string(&mut buf)?;
            match buf.as_ref() {
                "q" => {
                    let mut stdout = io::stdout().into_raw_mode().unwrap();
                    write!(stdout, "{}", termion::clear::All)?;
                    return Ok(());
                }
                "n" => debug_state.tick(&tables),
                _ => (),
            }
        }
    }
    vm.read_image_file(&opt.image_file)?;
    vm.run_loop(&tables);
    Ok(())
}
