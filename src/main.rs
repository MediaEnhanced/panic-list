//! panic-list Binary
use std::{env, error, fs, path, process, str};

/// Get all files in a directory with a specific extension
fn get_ext_paths(dir: &str, ext_str: &str) -> Result<Vec<path::PathBuf>, Box<dyn error::Error>> {
    //println!("Directory: {}", dir);
    let paths = fs::read_dir(dir)?
        // Filter out all those directory entries which couldn't be read
        .filter_map(|res| res.ok())
        // Map the directory entries to paths
        .map(|dir_entry| dir_entry.path())
        // Filter out all paths with extensions other than matching the ext_str
        .filter_map(|path| {
            if path.extension().is_some_and(|ext| ext == ext_str) {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    //println!("Files: {:?}", paths);
    Ok(paths)
}

/// Generated Callgraph Constants:
#[cfg(target_os = "windows")]
const NODE_COMPARISON_LENGTH: usize = 11;
#[cfg(target_os = "macos")]
const NODE_COMPARISON_LENGTH: usize = 12;
#[cfg(target_os = "linux")]
const NODE_COMPARISON_LENGTH: usize = 12;

const NODE_PREFIX: usize = 6;
const NODE_LENGTH: usize = NODE_COMPARISON_LENGTH + NODE_PREFIX;
const SYMBOL_PREFIX: usize = 22;
const SYMBOL_TO_NODE: usize = SYMBOL_PREFIX + 1 + NODE_COMPARISON_LENGTH;
const POINT_LENGTH: usize = 4;
const CARROT_LENGTH_FORWARD: usize = 2;
const CARROT_LENGTH_BACKWARDS: usize = POINT_LENGTH - CARROT_LENGTH_FORWARD;
const PREV_CARROT_LENGTH: usize = NODE_PREFIX + CARROT_LENGTH_BACKWARDS;

/// Extract Node of rust_begin_unwind
fn extract_node_from_root(data: &[u8]) -> Option<[u8; NODE_COMPARISON_LENGTH]> {
    let search = b"rust_begin_unwind";
    for i in 0..=(data.len() - search.len()) {
        if data[i..].starts_with(search) {
            let mut node = [0; NODE_COMPARISON_LENGTH];
            node.copy_from_slice(
                &data[(i - SYMBOL_TO_NODE)..(i - SYMBOL_TO_NODE + NODE_COMPARISON_LENGTH)],
            );
            return Some(node);
        }
    }
    None
}

/// Extract Node of search data
fn extract_node_from_name(data: &[u8], name_search: &[u8]) -> Option<[u8; NODE_COMPARISON_LENGTH]> {
    for i in 0..=(data.len() - (name_search.len() + 1)) {
        if data[i] == b'{' && data[(i + 1)..].starts_with(name_search) {
            let mut node = [0; NODE_COMPARISON_LENGTH];
            node.copy_from_slice(
                &data[(i - SYMBOL_TO_NODE + 1)..(i - SYMBOL_TO_NODE + 1 + NODE_COMPARISON_LENGTH)],
            );
            return Some(node);
        }
    }
    None
}

/// Extract Node of rust_begin_unwind
fn remove_downstream(data: &mut [u8], node: [u8; NODE_COMPARISON_LENGTH]) {
    for i in 0..=(data.len() - node.len()) {
        if data[i..].starts_with(&node)
            && data[i + NODE_COMPARISON_LENGTH + CARROT_LENGTH_FORWARD] == b'>'
        {
            data[i + NODE_COMPARISON_LENGTH + CARROT_LENGTH_FORWARD] = b'-';
        }
    }
}

/// Store Node Name into vector data
fn store_node_name(data: &[u8], node: [u8; NODE_COMPARISON_LENGTH], out_data: &mut Vec<u8>) {
    for i in 0..=(data.len() - node.len()) {
        if data[i..].starts_with(&node)
            && data[i - PREV_CARROT_LENGTH] != b'>'
            && data[i + SYMBOL_TO_NODE - 1] == b'{'
        {
            let name_start = i + SYMBOL_TO_NODE;
            let mut name_end = name_start;
            for (j, b) in data.iter().enumerate().skip(name_start) {
                if *b == b'}' {
                    name_end = j;
                    break;
                }
            }
            if name_end == name_start {
                panic!("Shouldn't Happen!");
            }

            #[cfg(feature = "demangle")]
            {
                let d = rustc_demangle::demangle(unsafe {
                    str::from_utf8_unchecked(&data[name_start..name_end])
                })
                .to_string();
                let d_bytes = d.as_bytes();
                let mut d_len = d_bytes.len();
                for j in (0..(d_bytes.len() - 1)).rev() {
                    if d_bytes[j] == b':' && d_bytes[j + 1] == b':' {
                        d_len = j;
                        break;
                    }
                }
                out_data.extend_from_slice(&d_bytes[..d_len]);
            }
            #[cfg(not(feature = "demangle"))]
            {
                out_data.extend_from_slice(&data[name_start..name_end]);
            }
            out_data.push(b'\n');
            return;
        }
    }
}

/// Get Previous Nodes and Store the names into vector data
fn get_prev_nodes(
    data: &[u8],
    root_node: [u8; NODE_COMPARISON_LENGTH],
    top_level_nodes: &[[u8; NODE_COMPARISON_LENGTH]],
    out_data: &mut Vec<u8>,
    node: [u8; NODE_COMPARISON_LENGTH],
    depth: usize,
) -> usize {
    for tln in top_level_nodes {
        if node == *tln {
            store_node_name(data, node, out_data);
            return 1;
        }
    }
    if depth > 0 {
        let mut tab = 0;
        for i in 0..=(data.len() - node.len()) {
            if data[i..].starts_with(&node) && data[i - PREV_CARROT_LENGTH] == b'>' {
                let mut prev_node = [0; NODE_COMPARISON_LENGTH];
                prev_node.copy_from_slice(
                    &data[(i - NODE_LENGTH - POINT_LENGTH)
                        ..(i - NODE_LENGTH - POINT_LENGTH + NODE_COMPARISON_LENGTH)],
                );
                // Basic Recursive Check (But Needs something different)
                if prev_node != node || prev_node != root_node {
                    let new_tab = get_prev_nodes(
                        data,
                        root_node,
                        top_level_nodes,
                        out_data,
                        prev_node,
                        depth - 1,
                    );
                    if new_tab > tab {
                        tab = new_tab;
                    }
                }
            }
        }
        if tab > 0 {
            for _i in 0..tab {
                out_data.extend_from_slice(b"  ");
            }
            store_node_name(data, node, out_data);
            tab + 1
        } else {
            0
        }
    } else {
        0
    }
}

/// panic-list Program Arguments:
#[derive(Clone, Debug, bpaf::Bpaf)]
#[bpaf(options, version)]
struct ProgramArgs {
    #[bpaf(long, argument::<String>("DIR-PATH"), fallback(String::from("./")))]
    /// Specify Cargo Root Directory Path
    cargo_path: String,

    #[bpaf(short('d'), long)]
    /// Turn off library default features.
    /// Passes along --no-default-features
    no_default_feat: bool,

    #[bpaf(short('f'), long, argument::<String>("LIST"), fallback(String::new()))]
    /// Turn on features of the library.
    /// Use a space or comma separated list.
    features: String,

    #[bpaf(short('c'), long)]
    /// Generate ONLY the rust core (for #![no_std] libraries)
    only_core: bool,

    #[bpaf(short('p'), long, argument::<String>("NAME"), fallback(String::from("release")))]
    /// Specify what Cargo profile to use.
    /// Uses the release profile as default.
    profile: String,

    #[bpaf(short('o'), long, argument::<String>("FILE-PATH"), fallback(String::new()))]
    /// Write the panic-list output to: path/file
    output: String,

    #[bpaf(short('C'), long)]
    /// Clean-up temporary files before panic-list generation.
    should_clean: bool,

    #[bpaf(long, argument::<usize>("INT"), fallback(10))]
    /// Specify the max recursive depth of the panic callgraph analysis.
    recursive_depth: usize,

    #[bpaf(short('v'), long)]
    /// Print the extra command output.
    /// Errors should always be printed.
    verbose: bool,

    #[bpaf(short('w'), long)]
    /// Indicate if the package is a workspace member.
    in_workspace: bool,

    #[bpaf(positional("PACKAGE-NAME"))]
    /// Name of the package to generate a panic-list for
    package_name: String,
}

/// Standard Program Entry Point
fn main() -> Result<(), std::io::Error> {
    // Uncomment below to enable the rust backtrace printing when panic unwinding (debug-mode program execution)
    // #[allow(unsafe_code)]
    // unsafe {
    //     env::set_var("RUST_BACKTRACE", "1")
    // };

    // Collect the Program Arguments into a structure.
    let pargs = program_args().run();

    // Set the root directory of the tested Library Cargo
    env::set_current_dir(&pargs.cargo_path).unwrap();

    // Tell rustc to output llvm-bc files FOR ALL .rlib files generated in the process
    #[allow(unsafe_code)]
    unsafe {
        env::set_var("RUSTFLAGS", "--emit=llvm-bc"); //-Z print-llvm-passes
    }

    // Create the profile argument string for the rustc command
    let mut profile_str = String::from("--profile=");
    profile_str.push_str(pargs.profile.as_str());

    // Potentially clean the target directory before panic-list generation
    if pargs.should_clean {
        let mut cargo_command = process::Command::new("cargo");
        cargo_command.arg("clean").arg(profile_str.as_str());
        println!("Running Cargo clean Command!");
        let command_output = cargo_command
            .output()
            .expect("Cargo Clean Command Failure!");
        if !command_output.status.success() {
            println!(
                "Cargo clean Error: {}",
                str::from_utf8(&command_output.stderr).unwrap()
            );
            return Ok(());
        }
        if pargs.verbose {
            println!(
                "Cargo clean Print: {}",
                str::from_utf8(&command_output.stderr).unwrap()
            );
        }
    }

    // Use the cargo nightly rustc command to create the .rlib of the target library cargo
    let mut cargo_command = process::Command::new("cargo");
    cargo_command.args(["+nightly", "rustc", "--lib", "--crate-type=rlib", "-Z"]);
    // Build either std or just core
    if !pargs.only_core {
        cargo_command.arg("build-std=std");
    } else {
        cargo_command.arg("build-std=core");
    }
    // Tell rustc the specific package name. Mostly useful for a workspace Cargo
    if pargs.in_workspace {
        cargo_command
            .arg("--package")
            .arg(pargs.package_name.as_str());
    }
    // Specify the profile
    cargo_command.arg(profile_str.as_str());

    // Configure the features
    if pargs.no_default_feat {
        cargo_command.arg("--no-default-features");
    }
    if !pargs.features.is_empty() {
        cargo_command.arg("--features").arg(pargs.features);
    }

    // Execute the Cargo Command
    println!("Running Cargo rustc Command!");
    let mut command_output = cargo_command.output().expect("Cargo Command Failure!");
    if !command_output.status.success() {
        println!(
            "Cargo rustc Error: {}",
            str::from_utf8(&command_output.stderr).unwrap()
        );
        return Ok(());
    }
    if pargs.verbose {
        println!(
            "Cargo rustc Print: {}",
            str::from_utf8(&command_output.stderr).unwrap()
        );
    }

    // Currently uncertain if changing the working directory should affect this...?
    //let mut dir_str = pargs.cargo_path.clone();
    //dir_str.push_str("target/");
    let mut dir_str = String::from("target/");
    dir_str.push_str(pargs.profile.as_str());
    dir_str.push_str("/deps/");

    // Create a symbol list in byte form for all .bc files rustc generated that start with the target package name
    // This is not bullet proof and will probably need adjusting.
    let mut symbol_bytes = Vec::new();
    let bc_file_paths = get_ext_paths(&dir_str, "bc").unwrap();
    for p in &bc_file_paths {
        let file_name = p.file_name().unwrap().to_str().unwrap();
        if file_name.starts_with(pargs.package_name.as_str()) {
            command_output = process::Command::new("llvm-nm")
                .args(["--defined-only", "--extern-only", "--format=just-symbols"])
                .arg(p.as_path())
                .output()
                .expect("LLVM Symbol Naming Failure!");
            if !command_output.status.success() {
                println!(
                    "llvm-nm Error: {}",
                    str::from_utf8(&command_output.stderr).unwrap()
                );
                return Ok(());
            }
            symbol_bytes.extend_from_slice(command_output.stdout.as_slice());
        }
    }

    // Create the LTO output files with a basic optimization pass of 1 (might need changing)
    let mut output_file = dir_str.clone();
    output_file.push_str("lto.o");
    let mut llvm_lto = process::Command::new("llvm-lto");
    llvm_lto
        .args(["-O1", "--save-merged-module", "-o"]) //--save-linked-module
        .arg(output_file.as_str());

    // Add all the exported symbols using previous llvm-nm commands
    let mut symbol_name = Vec::from(b"--exported-symbol=");
    let symbol_name_start = symbol_name.len();
    for b in symbol_bytes.as_mut_slice() {
        if *b == b'\n' {
            llvm_lto.arg(str::from_utf8(&symbol_name).unwrap());
            symbol_name.truncate(symbol_name_start);
        } else {
            symbol_name.push(*b);
        }
    }

    // Add all valid .bc files as arguments for the lto command
    for p in &bc_file_paths {
        let file_name = p.file_name().unwrap().to_str().unwrap();
        if file_name.starts_with("lto.o") || file_name.starts_with("panic_unwind") {
            continue;
        }
        llvm_lto.arg(p.as_path());
    }

    // Execute the LTO command
    println!("Running LTO Command!");
    let res = llvm_lto.output().expect("LLVM LTO Command Failure");
    if !res.status.success() {
        println!(
            "llvm LTO Error: {}",
            str::from_utf8(&command_output.stderr).unwrap()
        );
        return Ok(());
    }
    if pargs.verbose {
        println!("LTO Print: {}", str::from_utf8(&res.stderr).unwrap());
    }

    // Create the Callgraph using the generated LTO merged file
    let mut output_file = dir_str.clone();
    output_file.push_str("lto.o.merged.bc");
    println!("Running Opt Callgraph Command!");
    let res = process::Command::new("opt")
        .args(["-disable-output", "-passes=dot-callgraph"])
        .arg(output_file.as_str())
        .output()
        .expect("LLVM opt Command Failure");
    if !res.status.success() {
        println!(
            "opt Error: {}",
            str::from_utf8(&command_output.stderr).unwrap()
        );
        return Ok(());
    }
    if pargs.verbose {
        println!("opt Print: {}", str::from_utf8(&res.stderr).unwrap());
    }

    // Load the callgraph file
    let mut callgraph_file = dir_str.clone();
    callgraph_file.push_str("lto.o.merged.bc.callgraph.dot");
    let mut callgraph_data = fs::read(callgraph_file.as_str()).unwrap();

    // Extract the root node, generate the top level nodes (exported-symbols),
    // remove root node downstream elements to prevent some forms of recursion,
    // and store the top level nodes and their paths that make it to the root node.
    let mut out_data = vec![];
    if let Some(root_node) = extract_node_from_root(&callgraph_data) {
        println!("Generating the panic-list! {:?}", root_node);
        let mut node_name = Vec::new();
        let mut top_level_nodes = Vec::new();
        for b in symbol_bytes.as_mut_slice() {
            if *b == b'\n' {
                if let Some(n) = extract_node_from_name(&callgraph_data, &node_name) {
                    if n != root_node {
                        top_level_nodes.push(n);
                    }
                }
                node_name.clear();
            } else {
                node_name.push(*b);
            }
        }
        remove_downstream(&mut callgraph_data, root_node);

        println!("Max Recursive Depth: {}", pargs.recursive_depth);
        get_prev_nodes(
            &callgraph_data,
            root_node,
            &top_level_nodes,
            &mut out_data,
            root_node,
            pargs.recursive_depth,
        );
    } else {
        println!("Cannot find rust_begin_unwind symbol!");
        return Ok(());
    }

    // If output data is empty there are no panics in the library!
    if out_data.is_empty() {
        println!(
            "No panics found! Create a staticlib library output to analyze for true panic-freeness."
        );
    }

    // Print output data to terminal and files based on program arguments
    if pargs.output.is_empty() {
        println!("\n{}", str::from_utf8(&out_data).unwrap());
        let mut out_file_path_str = dir_str.clone();
        out_file_path_str.push_str("panic-list.txt");
        fs::write(out_file_path_str.as_str(), &out_data).unwrap();
        println!("Also Written To: {}", out_file_path_str.as_str());
    } else {
        fs::write(pargs.output.as_str(), &out_data).unwrap();
        println!("Wrote panic-list output: {}", pargs.output.as_str());
    }

    Ok(())
}
