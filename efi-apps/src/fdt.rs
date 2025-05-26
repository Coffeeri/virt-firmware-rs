extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use uefi::println;

use crate::hobs::{Hob, HobList};

pub fn print_node(node: fdt::node::FdtNode, level: usize) {
    let compatible = if let Some(c) = node.compatible() {
        c.first()
    } else {
        "-"
    };

    let regs: Vec<String> = if let Some(reg) = node.reg() {
        reg.map(|m| {
            format!(
                "0x{:08x} +0x{:x}",
                m.starting_address as u64,
                m.size.unwrap_or(0),
            )
        })
        .collect()
    } else {
        Vec::new()
    };

    println!(
        "{:ilen$}{:nlen$}{:24}{}",
        "",
        node.name,
        compatible,
        if regs.len() == 1 {
            regs.first().unwrap()
        } else {
            ""
        },
        ilen = level,
        nlen = 32 - level
    );

    if regs.len() != 1 {
        for r in regs {
            println!("{:level$}reg: {}", "", r, level = level + 2);
        }
    }

    for c in node.children() {
        print_node(c, level + 4);
    }
}

pub fn find_addr(compatible: &str) -> Result<u64, &'static str> {
    let Some(hl) = HobList::new() else {
        return Err("failed to get hob list");
    };

    let mut iter = hl.filter_map(|h| match h {
        Hob::Fdt(data) => Some(data),
        _ => None,
    });
    let Some(fdt_addr) = iter.next() else {
        return Err("no fdt hob found");
    };

    let res = unsafe { fdt::Fdt::from_ptr(fdt_addr as *const u8) };
    let fdt = match res {
        Err(_) => {
            return Err("fdt: parse error");
        }
        Ok(f) => f,
    };

    let Some(device) = fdt.find_compatible(&[compatible]) else {
        return Err("fdt: device not found");
    };
    let Some(mut reg) = device.reg() else {
        return Err("fdt: reg property not found");
    };
    let Some(range) = reg.next() else {
        return Err("fdt: reg property is empty");
    };

    Ok(range.starting_address as u64)
}
