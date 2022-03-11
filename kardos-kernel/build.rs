use std::{
    env,
    fs::File,
    io::{prelude::*, BufRead, BufReader, BufWriter},
    path::Path,
};

fn main() {
    let input_path = Path::new("./src/shinonome-0.9.11/18/latin1/font_src.bit");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let output_path = Path::new(&out_dir).join("latin1_font.rs");

    let input_file = File::open(input_path).unwrap();
    let output_file = File::create(output_path).unwrap();

    let input_buffer_reader = BufReader::new(input_file);
    let mut output_buffer_writer = BufWriter::new(output_file);

    let lines = input_buffer_reader.lines();

    let mut in_char = false;

    writeln!(
        &mut output_buffer_writer,
        "pub const LATIN1_FONT: [[u16; 18]; 221] = ["
    )
    .unwrap();

    for result in lines {
        let line = result.unwrap();

        if line.starts_with("STARTCHAR") {
            in_char = true;
            writeln!(&mut output_buffer_writer, "[").unwrap();
        } else if line.starts_with("ENDCHAR") {
            in_char = false;
            writeln!(&mut output_buffer_writer, "],").unwrap();
        }
        if in_char && (line.starts_with('.') || line.starts_with('@')) {
            let mut output_num: u16 = 0;
            for c in line.chars() {
                let bit = match c {
                    '.' => 0,
                    '@' => 1,
                    _ => panic!("Invalid character: {:?}", c),
                };
                output_num = (output_num << 1) | bit;
            }
            writeln!(&mut output_buffer_writer, "0b{:08b},", output_num).unwrap();
        }
    }
    writeln!(&mut output_buffer_writer, "];").unwrap();

    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=build.rs");
}
