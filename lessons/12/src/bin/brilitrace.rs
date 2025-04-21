use bril_rs::{Code, Position, Program, output_program};
use brilirs::basic_block::BBProgram;
use brilirs::error::PositionalInterpError;
use clap::Parser;
use lesson_12::interp::execute_main;
use std::fs::File;
use std::io::Read;
use utils::{CanonicalizeLiterals, Pass};

#[derive(Parser)]
#[command(about, version, author)] // keeps the cli synced with Cargo.toml
#[command(allow_hyphen_values(true))]
pub struct Cli {
    /// Flag to output the total number of dynamic instructions
    #[arg(short, long, action)]
    pub profile: bool,

    /// The bril file to run. stdin is assumed if file is not provided
    #[arg(short, long, action)]
    pub file: Option<String>,

    /// Flag for when the bril program is in text form
    #[arg(short, long, action)]
    pub text: bool,

    /// Arguments for the main function
    #[arg(action)]
    pub args: Vec<String>,

    /// The length of the trace
    #[arg(short, long, default_value_t = 100)]
    pub length: usize,
}

fn report_error<T: Into<PositionalInterpError>>(e: T) -> ! {
    let e = e.into();
    eprintln!("error: {e}");
    if let PositionalInterpError {
        pos:
            Some(Position {
                pos,
                pos_end,
                src: Some(src),
            }),
        ..
    } = e
    {
        let mut f = String::new();
        File::open(src).unwrap().read_to_string(&mut f).unwrap();

        let mut lines = f.split('\n');

        // print the first line
        eprintln!("{}", lines.nth((pos.row - 1) as usize).unwrap());
        eprintln!("{:>width$}", "^", width = pos.col as usize);

        // Then check if there is more
        if let Some(end) = pos_end {
            if pos.row != end.row {
                let mut row = pos.row + 1;
                while row < end.row {
                    eprintln!("{}", lines.nth((row - 1) as usize).unwrap());
                    eprintln!("^");
                    row += 1;
                }
                eprintln!("{}", lines.nth((end.row - 1) as usize).unwrap());
                eprintln!("{:>width$}", "^", width = end.col as usize);
            }
        }
    }
    std::process::exit(2)
}

fn main() {
    let args = Cli::parse();

    let input: Box<dyn std::io::Read> = match args.file.clone() {
        None => Box::new(std::io::stdin()),

        Some(input_file) => Box::new(File::open(input_file).unwrap()),
    };

    /*
    todo should you be able to supply output locations from the command line interface?
    Instead of builtin std::io::stdout()/std::io::stderr()
    */

    // It's a little confusing because of the naming conventions.
    //      - bril_rs takes file.json as input
    //      - bril2json takes file.bril as input
    let prog: Result<Program, _> = if args.text {
        bril2json::parse_abstract_program_from_read(input, true, true, args.file)
    } else {
        bril_rs::load_abstract_program_from_read(input)
    }
    .try_into();

    let prog = prog.unwrap_or_else(|e| {
        report_error(e);
    });

    // First run canonicalize literals
    let prog = CanonicalizeLiterals.run(prog);

    let bbprog: BBProgram = prog.clone().try_into().unwrap_or_else(|e| {
        report_error(e);
    });
    brilirs::check::type_check(&bbprog).unwrap_or_else(|e| {
        report_error(e);
    });

    let (trace, mut bbidx, mut iidx) = execute_main(&bbprog, args.length, &args.args)
        .unwrap_or_else(|e| {
            report_error(e);
        });

    let trace = trace.take();

    let mut prog = prog;

    // Find the main function and insert the __trace_succeeded label into the main function at that location
    let main_func = prog
        .functions
        .iter_mut()
        .find(|f| f.name == "main")
        .expect("No main function found in the program");

    eprintln!("Done at {}, {}", bbidx, iidx);

    for idx in 0..main_func.instrs.len() {
        if bbidx == 0 && iidx == 0 {
            // Insert the __trace_succeeded label before this instruction, then break
            main_func.instrs.insert(
                idx,
                Code::Label {
                    label: "__trace_succeeded".to_string(),
                    pos: None,
                },
            );

            break;
        }
        // If we are in the block, start decrementing iidx
        if bbidx == 0 {
            iidx -= 1;
        }

        // At a label, decrement bbidx
        if matches!(&main_func.instrs[idx], Code::Label { .. }) {
            bbidx -= 1;
        }
    }

    // Insert the trace into the main function
    main_func.instrs = trace.into_iter().chain(main_func.instrs.clone()).collect();

    // println!(
    //     "{}",
    //     main_func
    //         .instrs
    //         .iter()
    //         .map(|i| i.to_string())
    //         .collect::<Vec<_>>()
    //         .join("\n")
    // );

    output_program(&prog);
}
