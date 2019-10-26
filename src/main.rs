use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "prints this help message");
    opts.optflag("l", "lazy", "disables various data integrity checks");
    opts.optflag("v", "verbose", "enables verbose output");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            match f {
                getopts::Fail::ArgumentMissing(arg) => println!("Missing argument: {}", arg),
                getopts::Fail::UnrecognizedOption(opt) => println!("Unrecognized option: {}", opt),
                getopts::Fail::OptionMissing(opt) => println!("Missing option: {}", opt),
                getopts::Fail::OptionDuplicated(opt) => println!("Duplicated option: {}", opt),
                getopts::Fail::UnexpectedArgument(arg) => println!("Unexpected argument: {}", arg),
            }
            return;
        },
    };

    if matches.free.len() == 0 || matches.opt_present("h") {
        println!("{}", opts.usage(&format!("Command usage: {} FILENAME [options]", &args[0])));
        println!("Tip: to decompile a game, click and drag it on top of {}.", args[0]);
        return;
    }

    if matches.free.len() > 1 {
        println!("Unexpected input: {}", matches.free[1]);
        return;
    }

    let lazy = matches.opt_present("l");
    let verbose = matches.opt_present("v");
    let input = &matches.free[0];

    println!("Input file: '{}'", input);
    if lazy {
        println!("Lazy mode (--lazy, -l): data integrity checking disabled");
    }
    if verbose {
        println!("Verbose mode (--verbose, -v): verbose console output enabled");
    }

    let mut file = match fs::read(&input) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to open '{}': {}", input, e);
            return;
        },
    };

    let _assets = match gm8x::reader::from_exe(
        &mut file,
        !lazy,
        if verbose {
            Some(|s: &str| println!("[gm8x] {}", s))
        } else {
            None
        },
        None, // dump_dll
        true, // multithread
    ) {
        Ok(a) => a,
        Err(e) => {
            println!("Error parsing exe: {}", e);
            return;
        }
    };

    println!("Successfully parsed assets from '{}'", input);
}
