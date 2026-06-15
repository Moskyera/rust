
use crate::interface::chain::State;
use crate::core::field::Address;
use crate::mint::component::{STAKE_HACD_VMKIND, UNSTAKE_HACD_VMKIND};
use crate::mint::operate::staking_exec_hvm_external;
use crate::sys::RetErr;

/// Rust-side entry for HIP-25 HVM opcodes. Full HVM runtime (Go) calls the same Mint hooks.
pub fn exec_staking_hvm_opcode(
    opcode: u8,
    payload: &[u8],
    staker: &Address,
    height: u64,
    state: &mut dyn State,
) -> RetErr {
    if opcode != STAKE_HACD_VMKIND && opcode != UNSTAKE_HACD_VMKIND {
        return errf!("unsupported staking HVM opcode {}", opcode);
    }
    staking_exec_hvm_external(opcode, payload, staker, height, state)?;
    Ok(())
}