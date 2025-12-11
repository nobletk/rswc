mod args;
mod counter;

use args::ArgSet;
use counter::{Flags, print_files_results, print_stdin_results, process_files, process_stdin};
use std::convert::TryInto;
use std::io::stdout;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut my_flags = Flags {
        bytes: false,
        lines: false,
        words: false,
        chars: false,
    };
    let custom_flags = [
        "-c", "--bytes", "-l", "--lines", "-w", "--words", "-m", "--chars", "--help",
    ];

    let help_msg = [
        "Usage: rswc [OPTION]... [FILE]...",
        "  -c, --bytes    print the byte counts",
        "  -l, --lines    print the line counts",
        "  -w, --words    print the word counts",
        "  -m, --chars    print the character counts",
        "      --help     display help and exit",
    ];

    let args_set: ArgSet = (std::env::args().skip(1), &custom_flags[..])
        .try_into()
        .map_err(|e: String| {
            eprintln!("{}", e);
            std::process::exit(1);
        })?;

    my_flags.bytes = args_set.has("--bytes") || args_set.has("-c");
    my_flags.lines = args_set.has("--lines") || args_set.has("-l");
    my_flags.words = args_set.has("--words") || args_set.has("-w");
    my_flags.chars = args_set.has("--chars") || args_set.has("-m");

    if args_set.has("-h") || args_set.has("--help") {
        print_help(&help_msg);
        std::process::exit(1);
    }

    if !my_flags.bytes && !my_flags.lines && !my_flags.words && !my_flags.chars {
        my_flags.bytes = true;
        my_flags.lines = true;
        my_flags.words = true;
    }

    let files = &args_set.file_paths;

    if files.is_empty() {
        let counts = process_stdin(&my_flags)?;
        print_stdin_results(&mut stdout(), &counts, &my_flags)?;
    } else {
        let results = process_files(&files, &my_flags);
        print_files_results(&mut stdout(), &results, &my_flags)?;
    }

    Ok(())
}

fn print_help(messages: &[&str]) {
    for msg in messages {
        println!("{}", msg);
    }
}
