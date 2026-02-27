use bitfield_struct::bitfield;
use spin::Mutex;

use crate::exception::StackFrame;
use crate::snp::cpuid::snp_cpuid;
use crate::snp::ghcb::msr_ghcb_terminate;
use crate::snp::ghcb::GHCB;

pub static GHCB: Mutex<GHCB> = Mutex::new(unsafe { core::mem::zeroed() });

// vc debug logging (using SERCON would trigger recursive #vc)

pub fn snp_outb(port: u16, value: u8) {
    let info = ExitInfoIoIo::outb(port);
    io_write(info, value as u64);
}

fn snp_putsln(buf: &[u8]) {
    for &c in buf {
        snp_outb(0x3f8, c);
    }
    snp_outb(0x3f8, b'\n');
}

macro_rules! vclog {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut linebuf = $crate::linebuf::LineBuf::new();
        let _ = core::write!(&mut linebuf, $($arg)*);
        snp_putsln(linebuf.as_bytes());
    }};
}

// ioio handling

#[bitfield(u64)]
struct ExitInfoIoIo {
    pub input: bool,
    pub _res1: bool,
    pub string: bool,
    pub repeat: bool,
    pub data8: bool,
    pub data16: bool,
    pub data32: bool,
    pub addr16: bool,
    pub addr32: bool,
    pub addr64: bool,
    #[bits(3)]
    pub segment: u8,
    #[bits(3)]
    pub _res2: u8,
    #[bits(48)]
    pub port: u64,
}

impl ExitInfoIoIo {
    fn inport(port: u16) -> Self {
        Self::new()
            .with_input(true)
            .with_addr16(true)
            .with_port(port as u64)
    }
    fn inb(port: u16) -> Self {
        Self::inport(port).with_data8(true)
    }
    fn inw(port: u16) -> Self {
        Self::inport(port).with_data16(true)
    }
    fn inl(port: u16) -> Self {
        Self::inport(port).with_data32(true)
    }

    fn out(port: u16) -> Self {
        Self::new()
            .with_input(false)
            .with_addr16(true)
            .with_port(port as u64)
    }
    fn outb(port: u16) -> Self {
        Self::out(port).with_data8(true)
    }
    fn outw(port: u16) -> Self {
        Self::out(port).with_data16(true)
    }
    fn outl(port: u16) -> Self {
        Self::out(port).with_data32(true)
    }
}

fn io_read(info: ExitInfoIoIo) -> u64 {
    let mut ghcb = GHCB.lock();
    ghcb.reset();
    ghcb.set_exit(0x7b, info.into(), 0);
    ghcb.set_rax(0);
    ghcb.vmmcall();
    ghcb.rax
}

fn io_write(info: ExitInfoIoIo, data: u64) {
    let mut ghcb = GHCB.lock();
    ghcb.reset();
    ghcb.set_exit(0x7b, info.into(), 0);
    ghcb.set_rax(data);
    ghcb.vmmcall();
}

fn handle_ioio(regs: &mut StackFrame) {
    let ins = unsafe {
        let rip = regs.rip as *const u8;
        core::slice::from_raw_parts(rip, 4)
    };

    match (ins[0], ins[1]) {
        (0xec, _) => {
            // inb
            let info = ExitInfoIoIo::inb(regs.rdx as u16);
            regs.rax = io_read(info);
            regs.rip += 1
        }
        (0x66, 0xed) => {
            // inw
            let info = ExitInfoIoIo::inw(regs.rdx as u16);
            regs.rax = io_read(info);
            regs.rip += 2
        }
        (0xed, _) => {
            // inl
            let info = ExitInfoIoIo::inl(regs.rdx as u16);
            regs.rax = io_read(info);
            regs.rip += 1
        }

        (0xee, _) => {
            // outb
            let info = ExitInfoIoIo::outb(regs.rdx as u16);
            io_write(info, regs.rax);
            regs.rip += 1
        }
        (0x66, 0xef) => {
            // outw
            let info = ExitInfoIoIo::outw(regs.rdx as u16);
            io_write(info, regs.rax);
            regs.rip += 2
        }
        (0xef, _) => {
            // outl
            let info = ExitInfoIoIo::outl(regs.rdx as u16);
            io_write(info, regs.rax);
            regs.rip += 1
        }

        _ => {
            vclog!(
                "#vc/ioio: opcodes {:02x} {:02x} {:02x} {:02x}",
                ins[0],
                ins[1],
                ins[2],
                ins[3],
            );
            msr_ghcb_terminate(0, 0);
        }
    }
}

// mmio handling

fn mmio_read(addr: u64, size: u64) -> u64 {
    let mut ghcb = GHCB.lock();
    ghcb.reset();
    ghcb.set_scratch(0);
    ghcb.set_exit(0x8000_0001, addr, size);
    ghcb.vmmcall();
    ghcb.get_scratch()
}

fn mmio_write(addr: u64, data: u64, size: u64) {
    let mut ghcb = GHCB.lock();
    ghcb.reset();
    ghcb.set_scratch(data);
    ghcb.set_exit(0x8000_0002, addr, size);
    ghcb.vmmcall();
}

fn handle_mmio(regs: &mut StackFrame) {
    let ins = unsafe {
        let rip = regs.rip as *const u8;
        core::slice::from_raw_parts(rip, 4)
    };

    match (ins[0], ins[1], ins[2]) {
        (0x66, 0x8b, 0x02) => {
            // movw %ax, (%rdx)
            regs.rax = mmio_read(regs.rdx, 2);
            regs.rip += 3;
        }
        (0x88, 0x02, _) => {
            // movb (%rdx), %al
            mmio_write(regs.rdx, regs.rax, 1);
            regs.rip += 2;
        }
        (0x89, 0x02, _) => {
            // movl (%rdx), %eax
            mmio_write(regs.rdx, regs.rax, 4);
            regs.rip += 2;
        }
        (_, _, _) => {
            vclog!(
                "#vc/npf: opcodes {:02x} {:02x} {:02x} {:02x}",
                ins[0],
                ins[1],
                ins[2],
                ins[3],
            );
            msr_ghcb_terminate(0, 0);
        }
    }
}

// cpuid handling

fn handle_cpuid(regs: &mut StackFrame) {
    let func = regs.rax as u32;
    let subfn = regs.rcx as u32;
    let (eax, ebx, ecx, edx) = snp_cpuid(func, subfn);
    regs.rax = eax as u64;
    regs.rbx = ebx as u64;
    regs.rcx = ecx as u64;
    regs.rdx = edx as u64;
    regs.rip += 2;
}

// main #vc handler

pub fn handle_vc(regs: &mut StackFrame) {
    match regs.error_code {
        0x72 => {
            handle_cpuid(regs);
        }
        0x7b => {
            handle_ioio(regs);
        }
        0x400 => {
            handle_mmio(regs);
        }
        _ => {
            vclog!("#vc: ec=0x{:x}", regs.error_code);
            msr_ghcb_terminate(0, 0);
        }
    }
}
