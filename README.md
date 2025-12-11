# RsWC

A Command line word count tool written in rust.

## Features

* Prints character, newline, word, and byte counts for a file or multiple files.

* Reads from the file into a fixed buffer size.


## Usage 

The tool takes the file or multiple files as an argument like:

```shell
rswc [OPTIONS] [FILE]
```

Or you can use `cat` to pipe the content of the file into the tool via stdin:

```shell
cat [FILE] | rswc [OPTIONS]
```

The options below may be used to select which counts are printed:

* `-c` or `--bytes` : prints the byte counts

* `-m` or `--chars` : prints the character counts

* `-l` or `--lines` : prints the newline counts

* `-w` or `--chars` : prints the word counts

* `--help` : prints help

If no `OPTIONS` are provided, the tool will always print the count in the following order:

```shell
# newlines, words, bytes [FILE]
  7145  58164  342190  test.txt
```

## Getting started

### Clone the repo

```shell
git clone https://github.com/nobletk/rswc
# then build the binary
make build
```

### Example usage

```shell
cargo run --release -- testdata/test.txt testdata/test.txt 
# output for mulitple files
   7145   58164  342190 testdata/test.txt
   7145   58164  342190 testdata/test.txt
  14290  116328  684380 total
```
