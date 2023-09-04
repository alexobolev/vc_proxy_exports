use std::{env, fs, process::ExitCode};
use std::io::{Cursor, Write};
use goblin::Object;

fn main() -> ExitCode{
    let args = env::args().collect::<Vec<_>>();

    let is_help = |a: &String| a == "-h" || a == "--help";
    let show_help = args.len() != 3 || args.iter().any(|a| is_help(&a.to_lowercase()));

    if show_help {
        eprintln!("Usage: ./vc_proxy_exports SOURCE.DLL RENAMED");
        eprintln!("  SOURCE.DLL = input dll path; RENAMED = new name stem of orig. file.");
        eprintln!("  No other options or arguments are supported; output is sent to STDOUT.");
        return ExitCode::FAILURE;
    }

    let (input_dll, renamed_dll) = (&args[1], &args[2]);

    let input_buf = fs::read(input_dll).expect("failed to read input file");
    let goblin_obj = Object::parse(&input_buf).expect("failed to parse input file");

    if let Object::PE(pe) = goblin_obj {
        let mut output_buf = Vec::<u8>::new();
        let mut output = Cursor::new(&mut output_buf);

        for (index, export) in pe.exports.iter().enumerate() {
            if let Some(export_name) = export.name {
                writeln!(output, "#pragma comment(linker, \"/export:{0}={1}.{0}\")", export_name, renamed_dll)
                    .expect("failed to append to the internal string buffer");
            } else {
                eprintln!("unnamed export # {}, which is unsupported:", index);
                eprintln!("  {:?}", export);
                return ExitCode::FAILURE;
            }
        }

        println!("{}", String::from_utf8_lossy(&output_buf));
        ExitCode::SUCCESS
    } else {
        eprintln!("Input '{}' was not a valid PE file.", input_dll);
        ExitCode::FAILURE
    }
}
