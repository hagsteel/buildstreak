use std::fs::{File, OpenOptions, create_dir, read_to_string};
use std::path::PathBuf;
use std::io::{Read, Result, Write};
use std::time::Instant;
use structopt::StructOpt;
use chrono::Local;

const CONF_FILE_NAME: &str = ".buildstreak";

fn dirname() -> PathBuf {
    PathBuf::from("buildstreak")
}

#[derive(StructOpt, Debug)]
#[structopt(version = "0.1", author = "Hagsteel", about = "Build status tracker")]
enum Opts {
    #[structopt(about = "Log a successful build")]
    Success,

    #[structopt(about = "Log a failed build")]
    Fail,

    #[structopt(about = "Print a number of successful / unsuccessful builds (- for failed builds)")]
    Status,

    #[structopt(about = "Setup a build status dir to keep all the log files in")]
    Init { path: Option<PathBuf> },

    #[structopt(about = "Reset todays stats")]
    Reset,

    #[structopt(about = "Output as a tmux status field")]
    Tmux,
}

fn main() {
    let opt = Opts::from_args();

    let res = match opt {
        Opts::Init { path } => init(path),
        Opts::Success => success(),
        Opts::Fail => fail(),
        Opts::Status => status(),
        Opts::Tmux => tmux(),
        Opts::Reset => reset(),
    };

    if res.is_err() {
        eprintln!("{:?}", res.unwrap_err());
    }
}

/// Read the config file in the current directory.
fn read_config() -> Result<PathBuf> {
    let file = read_to_string(CONF_FILE_NAME)?;
    Ok(file.into())
}

fn todays_file() -> Result<PathBuf> {
    let root = read_config()?;
    let today = {
        let now = Local::now();
        format!("{}.streak", now.format("%d-%m-%y"))
    };
    let file = root.join(today);
    Ok(file)
}

fn init(path: Option<PathBuf>) -> Result<()> {
    let dir = match path {
        None => dirname(),
        Some(path) => path.join(dirname()),
    };

    create_dir(&dir)?;
    let conf_file = CONF_FILE_NAME;
    let mut file = File::create(conf_file)?;
    let buf = format!("{}", dir.as_os_str().to_str().unwrap());
    file.write(buf.as_bytes())?;
    eprintln!("{:?}", "created file");
    Ok(())
}

fn get_file(path: PathBuf) -> Result<File> {
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)
}

/// Read the build numbers, first number is successes and second if failures.
/// If the file doesn't exist, create it with 0, 0.
/// In case the file has been modified outside of the program, reset the file
/// to 0, 0
fn read_numbers() -> Result<(usize, usize)> {
    let file_path = todays_file()?;
    if !file_path.exists() {
        write_numbers(0, 0)?;
    }
    let mut file = read_to_string(file_path)?;
    let split = file.split('\n').collect::<Vec<_>>();
    if split.len() != 2 {
        write_numbers(0, 0);
        return Ok((0, 0));
    }
    let success = split[0].parse::<usize>().unwrap_or(0);
    let fail = split[1].parse::<usize>().unwrap_or(0);
    Ok((success, fail))
}

fn write_numbers(success: usize, fail: usize) -> Result<()> {
    let mut file = File::create(todays_file()?)?;
    let buf = format!("{}\n{}", success, fail);
    file.write(buf.as_bytes())?;
    Ok(())
}

fn success() -> Result<()> {
    let (mut success, fail) = read_numbers()?;
    success += 1;
    write_numbers(success, fail)?;
    Ok(())
}

fn fail() -> Result<()> {
    let (success, mut fail) = read_numbers()?;
    fail += 1;
    write_numbers(success, fail)?;
    Ok(())
}

fn status() -> Result<()> {
    let (success, fail) = read_numbers()?;
    println!("{} | {}", success, fail);
    Ok(())
}

fn reset() -> Result<()> {
    write_numbers(0, 0)?;
    Ok(())
}

fn tmux() -> Result<()> {
    if read_config().is_err() {
        print!("");
        return Ok(());
    }

    let (success, fail) = read_numbers()?;
    if success  == fail {
        print!("#[fg=colour66,bg=colour234]#[bg=colour66,fg=colour234] {success} | {fail} #[fg=colour234,bg=colour66]", 
            success=success, 
            fail=fail
            );
    }

    let success_colour = 237;
    let success_fg = 255;
    if success > fail {
        print!(
            "#[fg=colour{success_bg},bg=colour234]#[bg=colour{success_bg},fg=colour{success_fg}] {success} | {fail} #[fg=colour234,bg=colour{success_bg}]", 
            success=success, 
            fail=fail,
            success_bg=success_colour, 
            success_fg=success_fg
        );
    }

    let fail_colour = 196;
    if success < fail {
        print!("#[fg=colour{fail_bg},bg=colour234]#[bg=colour{fail_bg},fg=colour234] {success} | {fail} #[fg=colour234,bg=colour{fail_bg}]", 
            success=success, 
            fail=fail,
            fail_bg=fail_colour);
    }
    Ok(())
}
