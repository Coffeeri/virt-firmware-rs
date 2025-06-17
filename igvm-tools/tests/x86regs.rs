use igvm::snp_defs::SevFeatures;

use virtfw_igvm_tools::x86regs::{flat32_mode_regs, native_context, real_mode_regs, vmsa_context};

#[test]
fn test_real_mode() {
    let features = SevFeatures::new();
    let regs = real_mode_regs();
    let _ = native_context(&regs);
    let _ = vmsa_context(&regs, features);
}

#[test]
fn test_flat32_mode() {
    let features = SevFeatures::new();
    let regs = flat32_mode_regs();
    let _ = native_context(&regs);
    let _ = vmsa_context(&regs, features);
}
