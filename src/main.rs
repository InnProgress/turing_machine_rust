use crossterm::{cursor, QueueableCommand};
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use std::{
    env, fs,
    io::{stdin, stdout, Write},
    path::Path,
    process, thread,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Machine {
    initial_tape_position: isize,
    tape: String,
    rules: Vec<Rule>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Rule {
    state: String,
    read: char,
    write: char,
    r#move: Move,
    next_state: String,
}

#[derive(Deserialize)]
enum Move {
    #[serde(rename = "L")]
    Left,
    #[serde(rename = "R")]
    Right,
}

fn main() {
    print!("{}[2J", 27 as char);

    let args = env::args().skip(1).collect::<Vec<_>>();
    let args_length = args.len() as u16;

    thread::spawn(move || {
        if stdin().read_line(&mut String::new()).is_ok() {
            let stdout = stdout();
            let mut handle = stdout.lock();
            handle.queue(cursor::MoveTo(0, args_length)).unwrap();
            println!("App closed");
            process::exit(0);
        }
    });

    args.into_par_iter().enumerate().for_each(|(i, filename)| {
        let machine = read_file(filename.to_string());
        match machine {
            Ok(value) => run_turing_machine(value, i),
            Err(err) => println!("Error: {}", err),
        }
    });

    stdout().queue(cursor::MoveTo(0, args_length)).unwrap();
}

fn run_turing_machine(machine: Machine, line_index: usize) {
    let mut index: isize = machine.initial_tape_position;
    let mut state = String::from("0");
    let mut tape: String = machine.tape.clone();
    let mut read = tape.chars().nth(index as usize);

    while read.is_some() {
        let read_symbol = read.unwrap();
        let rule = machine
            .rules
            .iter()
            .find(|rule| rule.state == state && rule.read == read_symbol);

        let rule = match rule {
            Some(rule_value) => rule_value,
            None => break,
        };

        let safe_index = index as usize;
        tape.replace_range(safe_index..safe_index + 1, &rule.write.to_string());
        state = rule.next_state.clone();
        index += match rule.r#move {
            Move::Right => 1,
            Move::Left => -1,
        };

        read = tape.chars().nth(index as usize);

        let stdout = stdout();
        let mut handle = stdout.lock();
        handle.queue(cursor::MoveTo(0, line_index as u16)).unwrap();
        handle.write(format!("{}\n", tape).as_bytes()).unwrap();
    }
}

fn read_file(filename: String) -> Result<Machine, &'static str> {
    let contents =
        fs::read_to_string(&filename).map_err(|_| "Something went wrong with reading file")?;

    match Path::new(&filename).extension().unwrap().to_str().unwrap() {
        "txt" => parse_txt(contents),
        "json" => parse_json(contents),
        _ => Err("Unsupported file extension"),
    }
}

fn parse_txt(contents: String) -> Result<Machine, &'static str> {
    let lines: Vec<&str> = contents.lines().filter(|s| s.len() > 0).collect();

    let rules = lines[2..]
        .iter()
        .filter_map(|l| {
            let e: Vec<&str> = l.split(" ").collect();

            let r#move = match e.get(3).unwrap().chars().next().unwrap() {
                'L' => Move::Left,
                'R' => Move::Right,
                _ => return None,
            };

            Some(Rule {
                state: e.get(0).unwrap().to_string(),
                read: e.get(1).unwrap().chars().next().unwrap(),
                write: e.get(2).unwrap().to_string().chars().next().unwrap(),
                r#move,
                next_state: e.get(4).unwrap().to_string(),
            })
        })
        .collect();

    Ok(Machine {
        initial_tape_position: lines.get(1).unwrap().parse().unwrap(),
        tape: lines.get(0).unwrap().to_string(),
        rules,
    })
}

fn parse_json(contents: String) -> Result<Machine, &'static str> {
    let data: Value = serde_json::from_str(&contents)
        .map_err(|_| "Something went wrong with parsing json data")?;

    let rules: Vec<Rule> = data["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|rule| serde_json::from_value(rule.to_owned()).ok())
        .collect();

    Ok(Machine {
        initial_tape_position: data["initialTapePosition"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap(),
        tape: data["tape"].as_str().unwrap_or("").to_string(),
        rules,
    })
}
