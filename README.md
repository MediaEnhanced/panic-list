## Rust Panic-List Generation Tool
This tool is executed on Rust library crates in order to generate a list of library functions that can [panic](https://doc.rust-lang.org/book/ch09-01-unrecoverable-errors-with-panic.html). It is useful for determining what functions should be modifed in order to make them panic-free! Creating no-panic Rust code is useful for a variety of applications such as embedded targets. Functions that have paths leading to the panic handler function indicate that a program design has unrecoverable errors and does NOT account for all possible factors.

Rust libraries usually panic due to either program bugs or missuse by the user application. A more stable and reliable Rust library should never panic and using this tool can help achieve that goal. Analyzing panic function paths can lead to bug discovery (and then elimination) or errors that are recoverable and should be propagated up to the library user.

### How to Use
The [LLVM Software Binaries](https://github.com/llvm/llvm-project/releases) must be installed on the path. This project currently requires the `llvm-nm`, `llvm-lto`, and `opt` programs that should be included in the release packages.

Nightly Rust is also required for generating the complete panic analysis but it might not be a hard requirement in the future (probably with a different cost). It can be added by running:

```
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

This program is not currently published to [crates.io](https://crates.io/) but can be installed by running:

`cargo install --git https://github.com/MediaEnhanced/panic-list.git panic-list`

To generate the panic-list for a Rust library crate package, change directory into the Cargo root for the library and then execute the command:

`panic-list PACKAGE-NAME`

This will print out the panic list text into the terminal as well as write it to a text file with a shown path. A specific text file output path can be written to by applying the `--output=FILE-PATH` argument.

---
Features of the library can be turned on with `--features=LIST`. Default features of the library can be turned off by the `-d` flag. When generating a panic-list for a `#![no-std]` library the `-c` flag is recommended so that only the Rust core is linked and not std in its entirety. The Cargo release profile is used by default but can be changed by applying `--profile=NAME`.

If `panic-list` is executed with different features than a previous execution while using the same profile the `-C` flag is HEAVILY recommended for the first execution call with the new features due to current program design limitations. This argument will clean the target/profile temporary rustc files to ensure that rustc regenerates only relevant files for the panic-list program to operate with. This limitation should be eliminated in a future version.

All temporary files that are generated from executing panic-list will be constrained to the cargo target directory to make it easy for file exclusion in typical Rust git setups and can be removed from a typical `cargo clean` operation. The default directory where the temporary files are written to is: `target/release/deps/`. The callgraph `*.dot` temporary file that is generated can be pasted into [Online Graphviz](https://dreampuf.github.io/GraphvizOnline/) to create a usually messy callgraph diagram.

Unfortunately non-default Cargo targets are not currently supported however workspace libraries ARE supported by adding the `-w` flag. All valid program arguments can be seen by running: `panic-list --help`

---
### Some Notes
No changes have to be made to the analyzed library Cargo.toml and it will use the release profile by default. This tool was originally written with a focus on `#![no-std]` Rust library crates so there *maybe* issues with the panic-list output on libraries that require std. The code creates a convenient way of automating using certain llvm tools together with rustc output to create a useful list that can help the developer. Ideally, all functionality (and more) of the panic-list program will get implemented into Rust in the future to render this program obsolete. Ironically this application has a lot of ways it could panic and future versions should slowly be made more panic-free.

I wrote this after wanting to remove as many panics as possible from a no-std library I am developing that always leaves overflow-checks on (even with the release profile). These checks add even more ways that code can panic wherever a mathematical overflow can happen. I was unsatisfied with the lack of options/tools that exist to make finding all possible panics a reasonable task and after stumbling upon [No-Panic Rust](https://blog.reverberate.org/2025/02/03/no-panic-rust.html) during research, I was finally inspired to just make my own tool. I think I ended up making a program that is easy-to-use tool and can be applied to a variety of Rust libraries so I am now publishing it for the use by any Rust developer.

##### I want to hear from you! Please fill out a repository issue with any questions, problems, or suggestions!
---
### Example Library Output
A panic-list example library (plel) is included in this source [here](examples/lib/src/lib.rs) and it is used to help demonstrate the capabilities of this tool and show what CAN be done for some typical operations in order to create a panic-free version. [Cargo command aliases](.cargo/config.toml) were executed to generate the following example panic-list outputs for different configurations of the plel library.

No Rust standard library `#![no_std]` and overflows WILL panic:
<details>
<summary><code>cargo print-panic-list-for-example-lib-no-std</code></summary>

```
plel::possible::slice_byte
plel::possible::add_entries
  plel::possible::first_entry_internal
    core::panicking::panic_bounds_check
plel::possible::add
plel::possible::add_entries
  core::panicking::panic_const::panic_const_add_overflow
plel::possible::mult
  core::panicking::panic_const::panic_const_mul_overflow
plel::possible::sub
  core::panicking::panic_const::panic_const_sub_overflow
plel::possible::div
  core::panicking::panic_const::panic_const_div_by_zero
      core::panicking::panic_fmt
        rust_begin_unwind
```
</details>

Include Rust Standard Library (std) and overflows DO NOT panic:
<details>
<summary><code>cargo write-panic-list-for-example-lib</code></summary>

```
plel::possible::slice_byte
plel::possible::add_entries
  plel::possible::first_entry_internal
    core::panicking::panic_bounds_check
plel::print_hello_world
  std::io::stdio::_print
    std::io::stdio::print_to_buffer_if_capture_used
      core::ops::function::FnOnce::call_once
plel::print_hello_world
  std::io::stdio::_print
    <&std::io::stdio::Stdout as std::io::Write>::write_fmt
      std::thread::current::id::get_or_init::{{closure}}
plel::print_hello_world
  std::io::stdio::_print
    <&std::io::stdio::Stdout as std::io::Write>::write_fmt
        std::sys::thread_local::key::windows::LazyKey::init
          core::panicking::assert_failed
plel::print_hello_world
  std::io::stdio::_print
    std::io::stdio::print_to_buffer_if_capture_used
      core::ops::function::FnOnce::call_once
plel::print_hello_world
  std::io::stdio::_print
    <&std::io::stdio::Stdout as std::io::Write>::write_fmt
      std::thread::current::id::get_or_init::{{closure}}
plel::print_hello_world
  std::io::stdio::_print
    <&std::io::stdio::Stdout as std::io::Write>::write_fmt
        std::sys::thread_local::key::windows::LazyKey::init
          core::panicking::assert_failed
            core::panicking::assert_failed_inner
plel::possible::div
  core::panicking::panic_const::panic_const_div_by_zero
plel::print_hello_world
  std::io::stdio::_print
    <&std::io::stdio::Stdout as std::io::Write>::write_fmt
      core::option::expect_failed
plel::print_hello_world
  std::io::stdio::_print
    std::io::stdio::print_to_buffer_if_capture_used
      core::ops::function::FnOnce::call_once
plel::print_hello_world
  std::io::stdio::_print
    <&std::io::stdio::Stdout as std::io::Write>::write_fmt
      std::thread::current::id::get_or_init::{{closure}}
plel::print_hello_world
  std::io::stdio::_print
    <&std::io::stdio::Stdout as std::io::Write>::write_fmt
        std::sys::thread_local::key::windows::LazyKey::init
plel::print_hello_world
  std::io::stdio::_print
    std::io::stdio::print_to_buffer_if_capture_used
      std::io::Write::write_fmt
plel::print_hello_world
  std::io::stdio::_print
    std::sync::once_lock::OnceLock<T>::initialize
      std::sys::sync::once::futex::Once::call
plel::print_hello_world
  std::io::stdio::_print
    <&std::io::stdio::Stdout as std::io::Write>::write_fmt
      std::thread::current::id::get_or_init::{{closure}}
        std::thread::ThreadId::new::exhausted
plel::print_hello_world
  std::io::stdio::_print
    <&std::io::stdio::Stdout as std::io::Write>::write_fmt
plel::print_hello_world
  std::io::stdio::_print
              core::panicking::panic_fmt
                rust_begin_unwind
```
</details>
Notice all of the possible panics that occur by calling Rust's std::println! macro.

Note that because of the feature limitation explained above the `-C` flag should be added when executing either of these commands if the other command was run previously.
```
cargo print-panic-list-for-example-lib-no-std
cargo print-panic-list-for-example-lib-no-default
```

### Rust Library Test Outputs
Current demangle panic-list feature [dependency](https://github.com/rust-lang/rustc-demangle):
<details>
<summary><code>panic-list rustc-demangle</code></summary>

```
No panics found! Create a staticlib library output to analyze for true panic-freeness.
```
</details>
Supposedly no-panics

Popular Data Structures Library: [Hasbrown](https://github.com/rust-lang/hashbrown) (commit b5b0655 | tested on 2025-3-11)
<details>
<summary><code>panic-list hasbrown</code></summary>

```
hashbrown::raw::Fallibility::capacity_overflow
  core::panicking::panic_fmt
    rust_begin_unwind
```
</details>
This seems to indicate that for a release version of default-featured hashbrown there is ONE panic call that keeps the whole panic handler system in the code...

Popular Data Encoding Library: [base64 v0.22.1](https://github.com/marshallpierce/rust-base64/tree/v0.22.1)
<details>
<summary><code>panic-list base64</code></summary>

```
<base64::chunked_encoder::StringSink as base64::chunked_encoder::Sink>::write_encoded_bytes
<alloc::string::String as base64::write::encoder_string_writer::StrConsumer>::consume
  alloc::raw_vec::RawVecInner<A>::reserve::do_reserve_and_handle
    alloc::raw_vec::handle_error
      alloc::raw_vec::capacity_overflow
<base64::engine::general_purpose::GeneralPurpose as base64::engine::Engine>::internal_encode
<base64::engine::general_purpose::GeneralPurpose as base64::engine::Engine>::internal_decode
base64::alphabet::Alphabet::from_str_unchecked
base64::encode::add_padding
  core::panicking::panic_bounds_check
<base64::engine::general_purpose::GeneralPurpose as base64::engine::Engine>::internal_encode
<base64::engine::general_purpose::GeneralPurpose as base64::engine::Engine>::internal_decode
  core::slice::index::slice_end_index_len_fail
    core::slice::index::slice_end_index_len_fail::do_panic::runtime
<base64::engine::general_purpose::GeneralPurpose as base64::engine::Engine>::internal_encode
<base64::engine::general_purpose::GeneralPurpose as base64::engine::Engine>::internal_decode
  core::slice::index::slice_index_order_fail
    core::slice::index::slice_index_order_fail::do_panic::runtime
<base64::chunked_encoder::StringSink as base64::chunked_encoder::Sink>::write_encoded_bytes
<base64::display::FormatterSink as base64::chunked_encoder::Sink>::write_encoded_bytes
base64::alphabet::Alphabet::as_str
  core::result::unwrap_failed
        core::panicking::panic_fmt
          rust_begin_unwind
```
</details>
Could manually ensure that slice indices are not out of bounds and propagate an error up to remove some of the panics.

Note that most release profiles are compiled to NOT panic on overflow cutting out list-able panics and creates undesigned behavior /potential bugs if overflow ends up occurring.

---
#### TODO Other README Additions
