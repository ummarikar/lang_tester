// Copyright (c) 2019 King's College London created by the Software Development Team
// <http://soft-dev.org/>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, or the UPL-1.0 license <http://opensource.org/licenses/UPL>
// at your option. This file may not be copied, modified, or distributed except according to those
// terms.

//! This crate provide a simple language testing framework designed to help when you are testing
//! things like compilers and virtual machines. It allows users to embed simple tests for process
//! success/failure and for stderr/stdout inside a source file. It is loosely based on the
//! [`compiletest_rs`](https://crates.io/crates/compiletest_rs) crate, but is much simpler (and
//! hence sometimes less powerful), and designed to be used for testing non-Rust languages too.
//!
//! For example, a Rust language tester, loosely in the spirit of
//! [`compiletest_rs`](https://crates.io/crates/compiletest_rs), looks as follows:
//!
//! ```rust
//! use std::{path::PathBuf, process::Command};
//!
//! use lang_tester::LangTester;
//! use tempdir::TempDir;
//!
//! fn main() {
//!     // We use rustc to compile files into a binary: we store those binary files into `tempdir`.
//!     // This may not be necessary for other languages.
//!     let tempdir = TempDir::new("rust_lang_tester").unwrap();
//!     LangTester::new()
//!         .test_dir("examples/rust_lang_tester/lang_tests")
//!         // Only use files named `*.rs` as tests.
//!         .test_file_filter(|p| p.extension().unwrap().to_str().unwrap() == "rs")
//!         // Extract the first sequence of commented line(s) as the test.
//!         .test_extract(|s| {
//!             Some(
//!                 s.lines()
//!                     // Skip non-commented lines at the start of the file.
//!                     .skip_while(|l| !l.starts_with("//"))
//!                     // Extract consecutive commented lines.
//!                     .take_while(|l| l.starts_with("//"))
//!                     .map(|l| &l[2..])
//!                     .collect::<Vec<_>>()
//!                     .join("\n"),
//!             )
//!         })
//!         // We have two test commands:
//!         //   * `Compiler`: runs rustc.
//!         //   * `Run-time`: if rustc does not error, and the `Compiler` tests succeed, then the
//!         //     output binary is run.
//!         .test_cmds(move |p| {
//!             // Test command 1: Compile `x.rs` into `tempdir/x`.
//!             let mut exe = PathBuf::new();
//!             exe.push(&tempdir);
//!             exe.push(p.file_stem().unwrap());
//!             let mut compiler = Command::new("rustc");
//!             compiler.args(&["-o", exe.to_str().unwrap(), p.to_str().unwrap()]);
//!             // Test command 2: run `tempdir/x`.
//!             let runtime = Command::new(exe);
//!             vec![("Compiler", compiler), ("Run-time", runtime)]
//!         })
//!         .run();
//! }
//! ```
//!
//! This defines a lang tester that uses all `*.rs` files in a given directory as tests, running
//! two commands against them: `Compiler` (i.e. `rustc`); and `Run-time` (the compiled binary).
//!
//! Users can then write files with tests and their inputs such as the following:
//!
//! ```rust,ignore
//! // Compiler:
//! //   status: success
//! //   stderr:
//! //     warning: unused variable: `x`
//! //       ...unused_var.rs:12:9
//! //       ...
//! //
//! // Run-time:
//! //   status: success
//! //   stdout: Hello world
//! fn main() {
//!     let x = 0;
//!     println!("Hello world");
//! }
//! ```
//!
//! Tests use a two-level indentation syntax: the outer most level of indentation defines a command
//! name (multiple command names can be specified, as in the above); each command name can then
//! define tests for one or more of `status: <success|failure>`, `stderr: [<string>]`, `stdout:
//! [<string>]`.
//!
//! In essence, each keyword under a command name is a test for that command. The above file
//! contains 4 tests: the `Compiler` should succeed (e.g. return a `0` exit code when run on Unix),
//! and its `stderr` output should warn about an unused variable on line 12; and the resulting
//! binary should succeed and produce `Hello world` on `stdout`.
//!
//! Lines not mentioned are not tested: for example, the above file does not state whether the
//! `Compiler`s `stdout` should have content or not (but note that the line `stdout:` on its own
//! would state that the `Compiler` should have no content at all). `stderr`/`stdout` tests can use
//! `...` as a simple wildcard: if a line consists solely of `...`, it means "match zero or
//! more lines"; if a line begins with `...`, it means "match the remainder of the line only";
//! if a line ends with `...`, it means "match the start of the line only". A line may start and
//! end with `...`. `stderr`/`stdout` matches ignore leading/trailing whitespace and newlines, but
//! are case sensitive.

mod fuzzy;
mod parser;
mod tester;

pub use tester::LangTester;