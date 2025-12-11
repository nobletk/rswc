use std::{convert::TryFrom, path::PathBuf};

#[derive(Debug)]
pub struct ArgSet {
    pub flags: Vec<String>,
    pub file_paths: Vec<PathBuf>,
}

impl ArgSet {
    pub fn has(&self, name: &str) -> bool {
        self.flags.iter().any(|f| f == name)
    }
}

impl<I, S> TryFrom<(I, &[&str])> for ArgSet
where
    I: IntoIterator<Item = S>,
    S: Into<String> + AsRef<str>,
{
    type Error = String;

    fn try_from((args, custom_flags): (I, &[&str])) -> Result<Self, Self::Error> {
        let mut flags = Vec::new();
        let mut file_paths = Vec::new();

        for arg in args {
            let arg = arg.as_ref();

            if arg.starts_with('-') && arg != "-" {
                if arg.len() > 2 && !arg.starts_with("--") {
                    for ch in arg.chars().skip(1) {
                        let flag = format!("-{}", ch);
                        if custom_flags.contains(&flag.as_str()) {
                            flags.push(flag);
                        } else {
                            return Err(format!("rswc: invalid option -{}", ch));
                        }
                    }
                } else if custom_flags.contains(&arg) {
                    flags.push(arg.to_string());
                } else {
                    return Err(format!("rswc: unrecognized option {}", arg));
                }
            } else {
                file_paths.push(PathBuf::from(arg));
            }
        }

        Ok(ArgSet { flags, file_paths })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUSTOM_FLAGS: &[&str] = &[
        "-l", "-c", "-w", "-m", "--lines", "--bytes", "--words", "--chars",
    ];

    #[test]
    fn test_single_valid_flag_and_file() {
        let args = vec!["-l", "file.txt"];
        let result = ArgSet::try_from((args, CUSTOM_FLAGS)).unwrap();
        assert_eq!(result.flags, vec!["-l"]);
        assert_eq!(result.file_paths, vec![PathBuf::from("file.txt")]);
    }

    #[test]
    fn test_combined_valid_flags() {
        let args = vec!["-lc", "file.txt"];
        let result = ArgSet::try_from((args, CUSTOM_FLAGS)).unwrap();
        assert_eq!(result.flags, vec!["-l", "-c"]);
        assert_eq!(result.file_paths, vec![PathBuf::from("file.txt")]);
    }

    #[test]
    fn test_invalid_combined_valid_flags() {
        let args = vec!["-lz"];
        let err = ArgSet::try_from((args, CUSTOM_FLAGS)).unwrap_err();
        assert_eq!(err, "rswc: invalid option -z");
    }

    #[test]
    fn test_unknown_flag() {
        let args = vec!["-z"];
        let err = ArgSet::try_from((args, CUSTOM_FLAGS)).unwrap_err();
        assert_eq!(err, "rswc: unrecognized option -z");
    }

    #[test]
    fn test_multiple_files() {
        let args = vec!["-l", "file1.txt", "file2.txt"];
        let result = ArgSet::try_from((args, CUSTOM_FLAGS)).unwrap();
        assert_eq!(result.flags, vec!["-l"]);
        assert_eq!(
            result.file_paths,
            vec![PathBuf::from("file1.txt"), PathBuf::from("file2.txt")]
        );
    }

    #[test]
    fn test_long_flag_equivalent() {
        let args = vec![
            "--lines",
            "--bytes",
            "--chars",
            "--words",
            "file1.txt",
            "file2.txt",
        ];
        let result = ArgSet::try_from((args, CUSTOM_FLAGS)).unwrap();
        assert_eq!(
            result.flags,
            vec!["--lines", "--bytes", "--chars", "--words"]
        );
        assert_eq!(
            result.file_paths,
            vec![PathBuf::from("file1.txt"), PathBuf::from("file2.txt")]
        );
    }

    #[test]
    fn test_invalid_long_flag() {
        let args = vec!["--byte"];
        let err = ArgSet::try_from((args, CUSTOM_FLAGS)).unwrap_err();
        assert_eq!(err, "rswc: unrecognized option --byte");
    }
}
