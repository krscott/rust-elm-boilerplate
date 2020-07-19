mod api_types;
mod gui;

use gui::Gui;
use std::{error::Error, fs::read_to_string, path::PathBuf, process::exit};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, help = "Silence all log messages")]
    quiet: bool,

    #[structopt(short, long, parse(from_occurrences), help = "Increase log output")]
    verbose: usize,

    #[structopt(parse(from_os_str), help = "Input file")]
    input: Option<PathBuf>,
}

fn app(opt: Opt) -> Result<(), Box<dyn Error>> {
    let msg = match opt.input {
        Some(pathbuf) => api_types::FromRustMsg::EditFile(api_types::FileData {
            filename: pathbuf.to_str().unwrap_or("").to_owned(),
            content: read_to_string(pathbuf)?,
        }),
        None => api_types::FromRustMsg::EditFile(api_types::FileData {
            filename: "New File.txt".into(),
            content: "".into(),
        }),
    };

    let gui = Gui::new();

    gui.send(msg)?;

    loop {
        use api_types::LogLevel;
        use api_types::ToRustMsg::*;
        match gui.recv()? {
            Log { level, msg } => match level {
                LogLevel::Error => log::error!("{}", msg),
                LogLevel::Warn => log::warn!("{}", msg),
                LogLevel::Info => log::info!("{}", msg),
                LogLevel::Debug => log::debug!("{}", msg),
            },
            UploadFile(api_types::FileData {
                filename: _,
                content,
            }) => {
                println!("{}", content);
                break;
            }
        }
    }

    Ok(())
}

fn main() {
    let opt = Opt::from_args();

    stderrlog::new()
        .module(module_path!())
        .quiet(opt.quiet)
        .verbosity(opt.verbose + 1)
        .init()
        .unwrap();

    match app(opt) {
        Ok(()) => {}
        Err(e) => {
            log::error!("Program exited: {}", e);
            exit(1);
        }
    }
}
