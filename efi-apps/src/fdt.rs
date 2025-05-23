extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use uefi::println;

pub fn print_fdt_node(node: fdt::node::FdtNode, level: usize) {
    let compatible = if let Some(c) = node.compatible() {
        c.first()
    } else {
        "-"
    };

    let regs: Vec<String> = if let Some(reg) = node.reg() {
        reg.map(|m| {
            format!(
                "0x{:x} +0x{:x}",
                m.starting_address as u64,
                m.size.unwrap_or(0),
            )
        })
        .collect()
    } else {
        Vec::new()
    };

    println!(
        "{:ilen$}{:nlen$}{:20}{}",
        "",
        node.name,
        compatible,
        if regs.len() == 1 {
            regs.first().unwrap()
        } else {
            "-"
        },
        ilen = level,
        nlen = 24 - level
    );

    if regs.len() != 1 {
        for r in regs {
            println!("{:level$}reg: {}", "", r, level = level + 2);
        }
    }

    for c in node.children() {
        print_fdt_node(c, level + 4);
    }
}
