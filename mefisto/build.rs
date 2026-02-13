use std::process::ExitCode;

fn main() -> ExitCode {
    let target = std::env::var("TARGET").unwrap();
    let srcdir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    if target != "x86_64-unknown-none" {
        println!(
            "cargo::error=target not supported (expected x86_64-unknown-none, found {target})"
        );
        return ExitCode::from(1);
    }

    cc::Build::new().file("src/entry64.S").compile("entry64");
    cc::Build::new().file("src/idt.S").compile("idt");
    cc::Build::new().file("src/snp.S").compile("snp");

    println!("cargo::rustc-link-arg-bins=-no-pie");
    println!("cargo::rustc-link-arg-bins=-T{srcdir}/src/bios64.lds");
    println!("cargo:rerun-if-changed={srcdir}/src/bios64.lds");
    println!("cargo:rerun-if-changed={srcdir}/src/entry64.S");
    println!("cargo:rerun-if-changed={srcdir}/src/idt.S");
    println!("cargo:rerun-if-changed={srcdir}/src/snp.S");

    ExitCode::from(0)
}
