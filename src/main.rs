use log::debug;
use pretty_env_logger;
use std::{
    io::{self, Read, Write},
    path::PathBuf,
};
use structopt::StructOpt;
use termion::raw::IntoRawMode;
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
    vm.read_image_file(&opt.image_file)?;

    if opt.debug {
        let mut stdout = io::stdout().into_raw_mode()?;
        write!(stdout, "{}", termion::clear::All)?;
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut debug_state = Debugger::new(&mut vm);
        let mut reader = termion::async_stdin();
        let mut buf = String::new();

        // Draw the initial state
        debugger::draw(&mut terminal, &debug_state)?;

        // We don't call the draw method on every iteration of the loop because we don't need to
        // repaint the screen constantly, only when the display output changes (otherwise this will
        // waste a lot of resources just constantly repainting).
        loop {
            // get next key and perform appropriate action
            reader.read_to_string(&mut buf)?;
            match buf.as_ref() {
                "q" => {
                    let mut stdout = io::stdout().into_raw_mode().unwrap();
                    write!(stdout, "{}", termion::clear::All)?;
                    return Ok(());
                }
                "n" => {
                    debug_state.tick(&tables);
                    debugger::draw(&mut terminal, &debug_state)?;
                }
                _ => (),
            }
            buf.clear();
        }
    } else {
        vm.run_loop(&tables);
    }
    Ok(())
}
