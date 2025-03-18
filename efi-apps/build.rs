use std::process::ExitCode;

fn main() -> ExitCode {
    let target = std::env::var("TARGET").unwrap();

    if !target.ends_with("-uefi") {
        println!("cargo::error=target not supported (expected *-uefi)");
        return ExitCode::from(1);
    }

    ExitCode::from(0)
}
