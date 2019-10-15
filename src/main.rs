use log::debug;
use pretty_env_logger;
use std::{io, path::PathBuf};
use structopt::StructOpt;
mod lc3;

/// A VM for the LC3 architecture
#[derive(Debug, StructOpt)]
#[structopt(author)]
struct Opt {
    /// The path to an image file for the VM to execute
    #[structopt(parse(from_os_str))]
    pub image_file: PathBuf,
}

fn main() -> Result<(), io::Error> {
    pretty_env_logger::init();
    let opt = Opt::from_args();

    debug!("Initialized VM");
    let mut vm = lc3::LC3::new();
    vm.read_image_file(&opt.image_file)?;
    vm.run_loop();
    Ok(())
}
