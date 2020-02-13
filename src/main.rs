use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Result, Write};
use std::process::exit;

const PATH: &str = "/tmp/buildstatus";

fn main() {
    let mode = env::args().nth(1);

    let mode = match mode {
        Some(mode) => mode,
        None => {
            eprintln!("Mode missing");
            exit(1);
        }
    };

    let res = match &mode as &str {
        "success" => success(),
        "fail" => fail(),
        "status" => status(),
        "tmux" => tmux(),
        "reset" => reset(),
        _ => {
            eprintln!("Invalid command");
            exit(1);
        }
    };

    if res.is_err() {
        eprintln!("{:?}", res.unwrap_err());
    }
}

fn get_file() -> Result<File> {
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(PATH)
}

fn read_number() -> Result<isize> {
    let mut file = get_file()?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let num = s.parse::<isize>().unwrap_or(0);
    Ok(num)
}

fn write_number(num: isize) -> Result<()> {
    let mut file = File::create(PATH)?;
    let buf = format!("{}", num);
    file.write(buf.as_bytes())?;
    Ok(())
}

fn success() -> Result<()> {
    let mut num = read_number()?;
    num += 1;
    write_number(num)?;
    Ok(())
}

fn fail() -> Result<()> {
    let mut num = read_number()?;
    num -= 1;
    write_number(num)?;
    Ok(())
}

fn status() -> Result<()> {
    let num = read_number()?;
    println!("{:?}", num);
    Ok(())
}

fn reset() -> Result<()> {
    write_number(0)?;
    Ok(())
}

fn tmux() -> Result<()> {
    let num = read_number()?;
    if num == 0 {
        print!("#[fg=colour66,bg=colour234]#[bg=colour66,fg=colour234] {} #[fg=colour234,bg=colour66]", num);
    }

    let success_colour = 237;
    let success_fg = 255;
    if num > 0 {
        print!(
            "#[fg=colour{success},bg=colour234]#[bg=colour{success},fg=colour{success_fg}] {} #[fg=colour234,bg=colour{success}]", 
            num, 
            success=success_colour, 
            success_fg=success_fg
        );
    }

    let fail_colour = 196;
    if num < 0 {
        print!("#[fg=colour{fail},bg=colour234]#[bg=colour{fail},fg=colour234] {} #[fg=colour234,bg=colour{fail}]", num, fail=fail_colour);
    }
    Ok(())
}
