extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Reverse;
use core::fmt;
use core::ops::Range;
use log::{debug, trace};

use crate::pci::{pci_foreach_device, PciAddrKind, PciAddrRange, PciDevAddr, PciDevice, PciHost};
use crate::size::*;

#[derive(Debug)]
enum PciResourceKind {
    Bridge { bus: u8 },
    Bar { nr: u8 },
}

impl fmt::Display for PciResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PciResourceKind::Bridge { bus } => write!(f, "bridge bus {bus}"),
            PciResourceKind::Bar { nr } => write!(f, "device bar {nr}"),
        }
    }
}

struct PciResource {
    device: PciDevAddr,
    kind: PciResourceKind,
    range: PciAddrRange,
}

pub struct PciSetup<'p> {
    pcihost: &'p dyn PciHost,
    lastbus: u8,
    resources: Vec<PciResource>,
    io: PciAddrRange,
    mem32: PciAddrRange,
    mem64: PciAddrRange,
    bridge_min_io: u64,
    bridge_min_mem32: u64,
    bridge_min_mem64: u64,
    window_mem32: Range<u64>,
    window_mem64: Range<u64>,
}

impl<'p> PciSetup<'p> {
    pub fn new(pcihost: &'p dyn PciHost) -> Self {
        Self {
            pcihost,
            lastbus: 0,
            resources: Vec::new(),
            io: PciAddrRange::new_empty(PciAddrKind::Io),
            mem32: PciAddrRange::new_empty(PciAddrKind::Mem32),
            mem64: PciAddrRange::new_empty(PciAddrKind::Mem64),
            bridge_min_io: 0x1000,
            bridge_min_mem32: 2 * MIB as u64,
            bridge_min_mem64: 256 * MIB as u64,
            window_mem32: 0..0,
            window_mem64: 0..0,
        }
    }

    pub fn configure_mem32(&mut self, range: Range<u64>) {
        self.window_mem32 = range;
    }

    pub fn configure_mem64(&mut self, range: Range<u64>) {
        self.bridge_min_mem64 = (range.end - range.start) / 256;
        self.window_mem64 = range;
    }

    fn bus_enumerate(&mut self, bus: u8) {
        pci_foreach_device(self.pcihost, bus, |dev| {
            if !self.pcihost.is_pci_bridge(dev) {
                return;
            }
            let mut sec = self.pcihost.bridge_secondary(dev);
            let sub = self.pcihost.bridge_subordinate(dev);
            if sec == 0 {
                self.lastbus += 1;
                sec = self.lastbus;
                trace!("  {}: [assign] secondary bus nr {sec:02x}", dev.addr);
                self.pcihost.set_bridge_primary(dev, bus);
                self.pcihost.set_bridge_secondary(dev, sec);
                self.pcihost.set_bridge_subordinate(dev, 0xff);
                self.bus_enumerate(sec);
                self.pcihost.set_bridge_subordinate(dev, self.lastbus);
                if sec != self.lastbus {
                    trace!(
                        "  {}: [assign] subordinate bus nr {:02x}",
                        dev.addr,
                        self.lastbus
                    );
                }
            } else {
                trace!("  {}: [exists] secondary bus nr {sec:02x}", dev.addr);
                if self.lastbus < sec {
                    self.lastbus = sec;
                }
                if sec != sub {
                    trace!("  {}: [exists] subordinate bus nr {sub:02x}", dev.addr);
                    if self.lastbus < sub {
                        self.lastbus = sub;
                    }
                }
                self.bus_enumerate(sec);
            }
        })
    }

    /// assign pci bus numbers
    pub fn enumerate_all(&mut self) {
        debug!("enumerate pci buses");
        self.bus_enumerate(0)
    }

    /// calculate the resource window size needed
    fn bus_resource_sum(&self, bus: u8, kind: PciAddrKind) -> Option<u64> {
        let mut sum: u64 = self
            .resources
            .iter()
            .filter(|r| r.device.bus == bus)
            .filter(|r| r.range.kind == kind)
            .map(|r| r.range.size)
            .sum();

        match kind {
            PciAddrKind::Io => {
                if sum == 0 {
                    return None;
                }
                if sum < self.bridge_min_io {
                    sum = self.bridge_min_io;
                }
            }
            PciAddrKind::Mem32 => {
                if sum < self.bridge_min_mem32 {
                    sum = self.bridge_min_mem32;
                }
            }
            PciAddrKind::Mem64 => {
                if sum == 0 {
                    return None;
                }
                if sum < self.bridge_min_mem64 {
                    sum = self.bridge_min_mem64;
                }
            }
        }
        sum = sum.next_power_of_two();

        let (v, u) = pretty_usize(sum as usize);
        trace!("  bus {bus}: {kind:?} -> {v}{u}");
        Some(sum)
    }

    /// create a PciResource instance for a bridge window
    fn bus_bridge_resource(
        &self,
        bus: u8,
        bridge: &PciDevice,
        kind: PciAddrKind,
    ) -> Option<PciResource> {
        let sum = self.bus_resource_sum(bus, kind)?;
        let range = PciAddrRange::new_sized(kind, sum.next_power_of_two());
        let res = PciResource {
            device: bridge.addr.clone(),
            kind: PciResourceKind::Bridge { bus },
            range,
        };
        Some(res)
    }

    fn bus_resources(&mut self, bus: u8) {
        pci_foreach_device(self.pcihost, bus, |dev| {
            if self.pcihost.is_pci_bridge(dev) {
                let sec = self.pcihost.bridge_secondary(dev);
                self.bus_resources(sec);
                if let Some(io) = self.bus_bridge_resource(sec, dev, PciAddrKind::Io) {
                    self.resources.push(io);
                }
                if let Some(m32) = self.bus_bridge_resource(sec, dev, PciAddrKind::Mem32) {
                    self.resources.push(m32);
                }
                if let Some(m64) = self.bus_bridge_resource(sec, dev, PciAddrKind::Mem64) {
                    self.resources.push(m64);
                }
            }

            if self.pcihost.is_device(dev) {
                for b in 0..6 {
                    if let Some(range) = self.pcihost.bar(dev, b) {
                        let res = PciResource {
                            device: dev.addr.clone(),
                            kind: PciResourceKind::Bar { nr: b },
                            range,
                        };
                        self.resources.push(res);
                    }
                }
            }
        });
    }

    /// collect all pci resources needed
    pub fn resources_all(&mut self) {
        if !self.resources.is_empty() {
            return;
        }
        debug!("discover resources for pci buses");
        self.bus_resources(0);
        self.resources.sort_by_key(|k| Reverse(k.range.size));
        self.io.size = self.bus_resource_sum(0, PciAddrKind::Io).unwrap_or(0);
        self.mem32.size = self.bus_resource_sum(0, PciAddrKind::Mem32).unwrap_or(0);
        self.mem64.size = self.bus_resource_sum(0, PciAddrKind::Mem64).unwrap_or(0);

        if self.io.base == 0 && self.io.size > 0 {
            self.io.base = 0x10000 - self.io.size;
        }
        if self.mem32.base == 0 && self.mem32.size > 0 {
            self.mem32.base = self.window_mem32.end - self.mem32.size;
        }
        if self.mem64.base == 0 && self.mem64.size > 0 {
            self.mem64.base = self.window_mem64.start;
        }

        debug!("  {}", &self.io);
        debug!("  {}", &self.mem32);
        debug!("  {}", &self.mem64);
    }

    fn assign_kind(&mut self, kind: PciAddrKind, base: u64) {
        let buscount = self.lastbus + 1;
        let mut bases = vec![0; buscount as usize];
        bases.insert(0, base);

        trace!("  assign {kind:?}");
        for bus in 0..buscount {
            let mut addr = bases[bus as usize];
            trace!("    base bus {bus}: {addr:x}");
            if addr == 0 {
                continue;
            };

            for r in self
                .resources
                .iter_mut()
                .filter(|r| r.device.bus == bus)
                .filter(|r| r.range.kind == kind)
            {
                if let PciResourceKind::Bridge { bus: sec } = r.kind {
                    bases[sec as usize] = addr;
                }
                r.range.base = addr;
                addr += r.range.size;
            }
        }
    }

    /// assign base addresses to all resources
    pub fn assign_all(&mut self) {
        debug!("assign resources for pci buses");
        self.assign_kind(PciAddrKind::Io, self.io.base);
        self.assign_kind(PciAddrKind::Mem32, self.mem32.base);
        self.assign_kind(PciAddrKind::Mem64, self.mem64.base);
        for r in &self.resources {
            debug!("  {}  {}  {}", r.device, r.kind, r.range);
        }
    }

    /// apply calculated resource configuration
    pub fn apply_all(&self) {
        debug!("apply resources config to pci buses");
        for r in &self.resources {
            if r.range.base == 0 {
                continue;
            }
            match (&r.range.kind, &r.kind) {
                (PciAddrKind::Io, PciResourceKind::Bridge { .. }) => {
                    self.pcihost.set_bridge_io(&r.device, &r.range);
                }
                (PciAddrKind::Mem32, PciResourceKind::Bridge { .. }) => {
                    self.pcihost.set_bridge_mem(&r.device, &r.range);
                }
                (PciAddrKind::Mem64, PciResourceKind::Bridge { .. }) => {
                    self.pcihost.set_bridge_prefmem(&r.device, &r.range);
                }
                (_, PciResourceKind::Bar { nr }) => {
                    self.pcihost.set_bar(&r.device, *nr, &r.range);
                }
            }
        }
    }
}
