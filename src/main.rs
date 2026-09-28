use post_calc::evaluate_expression;
use std::env;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Mode 1: Evaluate expression passed directly as command-line arguments
        let expr = args[1..].join(" ");
        run_evaluation(&expr);
    } else {
        // Mode 2: Interactive REPL mode for Windows 11 Terminal / Command Prompt
        run_repl();
    }
}

fn run_evaluation(expr: &str) {
    match evaluate_expression(expr) {
        Ok(result) => println!("{}", result),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

fn run_repl() {
    println!("=================================================");
    println!("  Rust Scientific Calculator (Windows 11 CLI)");
    println!("  Type a math expression or type 'exit' to quit.");
    println!("=================================================\n");

    let stdin = io::stdin();
    loop {
        print!("calc> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            println!("Error: Failed to read input.");
            break;
        }

        let expr = input.trim();
        if expr.eq_ignore_ascii_case("exit") || expr.eq_ignore_ascii_case("quit") {
            println!("Goodbye!");
            break;
        }

        if expr.is_empty() {
            continue;
        }

        match evaluate_expression(expr) {
            Ok(result) => println!("  = {}", result),
            Err(err) => eprintln!("  [Error] {}", err),
        }
    }
}
