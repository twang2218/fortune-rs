

# `fortune` in Rust

## Introduction

This is yet another implementation of the `fortune` program in Rust. This implementation is intended to be a modern, feature-rich version of the `fortune` program that is compatible with the traditional `fortune` program, which can be dropped in as a replacement on Unix-like systems. The program is intended to support various versions of the fortune database format from different systems.

The `fortune` program is a simple program that displays a random message from a database of quotations. The `fortune` program is widely available on Unix-like systems. The program is commonly used to generate random messages for users logging into a system.

## Roadmap

- [x] Basic functionality;
  - [x] Read the fortune a cookies file and display a random message;
  - [x] Implement the `strfile` program to create the index file;
  - [x] Read the fortune the cookie file along with the `strfile` index file;
  - [x] Read the cookie files from a directory recursively;
- [ ] Implement traditional fortune options;
  - [x] `-a` - Choose from all lists of maxims, both offensive and not.  (See the -o option for more information on offensive fortunes.)
  - [x] `-c` - Show the cookie file from which the fortune came.
  - [x] `-D` - Enable additional debugging output.  Specify this option multiple times for more verbose output.  Only available if compiled with `-DDEBUG`.
  - [x] `-e` - Consider all fortune files to be of equal size (see discussion below on multiple files).
  - [x] `-f` - Print out the list of files which would be searched, but don't print a fortune.
  - [x] `-i` - Ignore case for `-m` patterns.
  - [x] `-l` - Long dictums only.  See -n on how **long** is defined in this sense.
  - [x] `-m pattern` - Print out all fortunes which match the basic regular expression pattern.  The syntax of these expressions depends  on  how your system defines re_comp(3) or regcomp(3), but it should nevertheless be similar to the syntax used in grep(1).
  - [x] `-n length` - Set the longest fortune length (in characters) considered to be **short** (the default is 160).  All fortunes longer  than this  are  considered  **long**.  Be careful!  If you set the length too short and ask for short fortunes, or too long and ask for long ones, fortune goes into a never-ending thrash loop.
  - [x] `-s` - Short apothegms only.  See -n on which fortunes are considered **short**.
  - [x] `-o` - Choose only from potentially offensive aphorisms.  The -o option is ignored if a fortune directory is specified.
  - [ ] `-u` - Don't translate UTF-8 fortunes to the locale when searching or translating.
  - [x] `-w` - Wait  before termination for an amount of time calculated from the number of characters in the message.  This is useful if it is executed as part of the logout procedure to guarantee that the message can be read before the screen is cleared.
  - [x] `[[n%] file/directory/all]`
  - [x] `-h` - Print a help message.
  - [x] `-v` - Print the version number.
- [ ] Implement modern enhancements;
  - [ ] Support TOML configuration file;
  - [ ] Embed the fortune database in the binary;
- [ ] Project management
  - [ ] Documentation;
  - [x] Testing;
  - [ ] CI/CD;

## References

### Existing Implementations

- [cmatsuoka/fortune-rs](https://github.com/cmatsuoka/fortune-rs), by Claudio Matsuoka, Brazil, at 2017;
- [c-OO-b/rust-fortune](https://github.com/c-OO-b/rust-fortune), by c-OO-b, Norway, at 2019;
- [wapm-packages/fortune](https://wapm.io/package/fortune), forked from [c-OO-b/rust-fortune](https://github.com/c-OO-b/rust-fortune), at 2019;
- [runebaas/fortune-mod.rs](https://github.com/runebaas/fortune-mod.rs), by [Daan Boerlage](https://boerlage.me), Switzerland, at 2019;
- [kvrohit/fortune](https://github.com/kvrohit/fortune), by [Rohit K Viswanath](https://kvrohit.dev/), at 2020;
- [davidkna/lolcow-fortune-rs](https://github.com/davidkna/lolcow-fortune-rs.git), by David Knaack, Berlin, Germany, at 2021;
- [blackbird1128/fortune_cookie](https://github.com/blackbird1128/fortune_cookie.rs.git), by Alexj, at 2023;
- [zuisong/rs-fortune](https://github.com/zuisong/rs-fortune), by ZuiSong, Changsha, China, at 2023;
- [cafkafk/fortune-kind](https://github.com/cafkafk/fortune-kind), by [Christina Sørensen](https://www.linkedin.com/in/cafkafk/), Denmark, at 2023;
- [rilysh/fortune-day](https://github.com/rilysh/fortune-day.git), at 2024;
- [FaceFTW/shell-toy](https://github.com/FaceFTW/shell-toy.git), by [Alex Westerman](http://faceftw.dev/), at 2024;
