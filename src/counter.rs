use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};

macro_rules! print_field {
    ($writer:expr, $value:expr, $enabled:expr, $width:expr) => {
        if $enabled {
            write!($writer, "{:>width$} ", $value, width = $width)?;
        }
    };
}

#[derive(Debug, PartialEq, Eq)]
pub struct Counts {
    pub lines: usize,
    pub words: usize,
    pub bytes: usize,
    pub chars: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Flags {
    pub lines: bool,
    pub words: bool,
    pub bytes: bool,
    pub chars: bool,
}

const MAX_WIDTH: usize = 7;

fn count_reader<R: Read>(mut reader: R, flags: &Flags) -> io::Result<Counts> {
    let mut buf = [0u8; 512 * 1024];
    let mut counts = Counts {
        lines: 0,
        words: 0,
        bytes: 0,
        chars: 0,
    };
    let mut in_word = false;

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }

        counts.bytes += n;

        let mut i = 0;
        while i < n {
            let b = buf[i];
            if b == b'\n' {
                counts.lines += 1;
            }

            if b.is_ascii_whitespace() {
                in_word = false;
            } else if !in_word {
                counts.words += 1;
                in_word = true;
            }

            i += 1;
        }

        if flags.chars {
            counts.chars += std::str::from_utf8(&buf[..n])
                .unwrap_or_default()
                .chars()
                .count();
        }
    }

    Ok(counts)
}

fn count_file(path: &Path, flags: &Flags) -> io::Result<Counts> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(512 * 1024, file);
    count_reader(reader, flags)
}

pub fn process_stdin(flags: &Flags) -> io::Result<Counts> {
    let stdin = io::stdin();
    let handle = stdin.lock();
    count_reader(handle, flags)
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileResult {
    Ok(PathBuf, Counts),
    Err(PathBuf, String),
}

pub fn process_files(files: &[PathBuf], flags: &Flags) -> Vec<FileResult> {
    files
        .par_iter()
        .map(|path| match count_file(path, flags) {
            Ok(counts) => FileResult::Ok(path.clone(), counts),
            Err(e) => FileResult::Err(path.clone(), e.to_string()),
        })
        .collect()
}

pub fn print_files_results<W: Write>(
    writer: &mut W,
    results: &[FileResult],
    flags: &Flags,
) -> io::Result<()> {
    let mut max_lines = 0;
    let mut max_words = 0;
    let mut max_bytes = 0;
    let mut max_chars = 0;

    let mut total = Counts {
        lines: 0,
        words: 0,
        bytes: 0,
        chars: 0,
    };

    for r in results {
        if let FileResult::Ok(_, c) = r {
            if flags.lines {
                max_lines = max_lines.max(c.lines);
            }
            if flags.words {
                max_words = max_words.max(c.words);
            }
            if flags.bytes {
                max_bytes = max_bytes.max(c.bytes);
            }
            if flags.chars {
                max_chars = max_chars.max(c.chars);
            }

            total.lines += c.lines;
            total.words += c.words;
            total.bytes += c.bytes;
            total.chars += c.chars;
        }
    }

    let width_lines = max_lines.to_string().len().max(MAX_WIDTH);
    let width_words = max_words.to_string().len().max(MAX_WIDTH);
    let width_bytes = max_bytes.to_string().len().max(MAX_WIDTH);
    let width_chars = max_chars.to_string().len().max(MAX_WIDTH);

    for r in results {
        match r {
            FileResult::Err(path, msg) => {
                writeln!(writer, "rswc: {}: {} ", path.display(), msg)?;
            }
            FileResult::Ok(path, c) => {
                print_field!(writer, c.lines, flags.lines, width_lines);
                print_field!(writer, c.words, flags.words, width_words);
                print_field!(writer, c.bytes, flags.bytes, width_bytes);
                print_field!(writer, c.chars, flags.chars, width_chars);
                writeln!(writer, "{}", path.display())?;
            }
        }
    }

    if results.len() > 1 {
        print_field!(writer, total.lines, flags.lines, width_lines);
        print_field!(writer, total.words, flags.words, width_words);
        print_field!(writer, total.bytes, flags.bytes, width_bytes);
        print_field!(writer, total.chars, flags.chars, width_chars);
        writeln!(writer, "total")?;
    }

    Ok(())
}

pub fn print_stdin_results<W: Write>(
    writer: &mut W,
    counts: &Counts,
    flags: &Flags,
) -> io::Result<()> {
    print_field!(
        writer,
        counts.lines,
        flags.lines,
        counts.lines.to_string().len().max(MAX_WIDTH)
    );
    print_field!(
        writer,
        counts.words,
        flags.words,
        counts.words.to_string().len().max(MAX_WIDTH)
    );
    print_field!(
        writer,
        counts.bytes,
        flags.bytes,
        counts.bytes.to_string().len().max(MAX_WIDTH)
    );
    print_field!(
        writer,
        counts.chars,
        flags.chars,
        counts.chars.to_string().len().max(MAX_WIDTH)
    );
    writeln!(writer, "-")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_print_results_from_mutli_files_with_large_numbers() {
        let flags = Flags {
            lines: true,
            words: true,
            bytes: true,
            chars: false,
        };

        let results = vec![
            FileResult::Ok(
                PathBuf::from("file1.txt"),
                Counts {
                    lines: 2143500000,
                    words: 17449200000,
                    bytes: 102657000000,
                    chars: 0,
                },
            ),
            FileResult::Ok(
                PathBuf::from("file2.txt"),
                Counts {
                    lines: 2143500000,
                    words: 17449200000,
                    bytes: 102657000000,
                    chars: 0,
                },
            ),
        ];

        let mut output = Cursor::new(Vec::new());
        print_files_results(&mut output, &results, &flags).unwrap();
        let actual = String::from_utf8(output.into_inner()).unwrap();

        let expected = "\
            2143500000 17449200000 102657000000 file1.txt
2143500000 17449200000 102657000000 file2.txt
4287000000 34898400000 205314000000 total
";

        assert_eq!(actual, expected, "Output does not match");
    }

    #[test]
    fn test_print_stdin_results_with_large_numbers() {
        let flags = Flags {
            lines: true,
            words: true,
            bytes: true,
            chars: false,
        };

        let counts = Counts {
            lines: 2143500000,
            words: 17449200000,
            bytes: 102657000000,
            chars: 0,
        };

        let mut output = Cursor::new(Vec::new());
        print_stdin_results(&mut output, &counts, &flags).unwrap();
        let actual = String::from_utf8(output.into_inner()).unwrap();

        let expected = "\
            2143500000 17449200000 102657000000 -
";

        assert_eq!(actual, expected, "Output does not match");
    }

    #[test]
    fn test_count_file() {
        let flags = Flags {
            lines: true,
            words: true,
            bytes: true,
            chars: true,
        };
        let path = Path::new("testdata/test.txt");
        assert!(path.exists(), "Test file does not exist: {:?}", path);

        let actual = count_file(path, &flags).unwrap();
        let expected = Counts {
            lines: 7145,
            words: 58164,
            bytes: 342190,
            chars: 339292,
        };
        assert_eq!(actual, expected);
    }

    //test results for seq 300000
    //2143500000 17449200000 102657000000

    #[test]
    fn test_process_files_mixed_ok_and_err() {
        let flags = Flags {
            lines: true,
            words: true,
            bytes: true,
            chars: false,
        };
        let valid_path = PathBuf::from("testdata/test.txt");
        let invalid_path = PathBuf::from("testdata/test.t");

        let actual = process_files(&[valid_path.clone(), invalid_path.clone()], &flags);

        assert_eq!(actual.len(), 2);

        let mut ok_found = false;
        let mut err_found = false;
        let expected_counts = Counts {
            lines: 7145,
            words: 58164,
            bytes: 342190,
            chars: 0,
        };

        for a in actual {
            match a {
                FileResult::Ok(path, actual_counts) => {
                    ok_found = true;
                    assert_eq!(path, valid_path);
                    assert_eq!(actual_counts, expected_counts);
                }
                FileResult::Err(path, msg) => {
                    err_found = true;
                    assert_eq!(path, invalid_path);
                    assert_eq!(msg, "No such file or directory (os error 2)");
                }
            }
        }

        assert!(ok_found, "Expected one successful FileResult::Ok");
        assert!(err_found, "Expected one unsuccessful FileResult::Err");
    }
}
