
/**
 * HIP-25: Pure HACD Staking
 *
 * Constants, status values, and on-chain state types.
 * See HIP-25 for the full protocol specification.
 */

/// 13% of eligible inscription protocol fees and transfer fees → reward pool
pub const STAKING_FEE_SHARE_PERCENT: u64 = 13;

/// ~3 days cooldown after unstake (1000 blocks ≈ 3.5 days per HIP-15)
pub const COOLDOWN_BLOCKS: u64 = 864;

/// ~90 days / 3 months minimum stake before unstake is allowed
pub const MIN_STAKE_BLOCKS: u64 = 25714;

/// HVM external action opcode (HIP-21)
pub const STAKE_HACD_VMKIND: u8 = 0x01;

/// HVM external action opcode (HIP-21)
pub const UNSTAKE_HACD_VMKIND: u8 = 0x02;

/// On-chain staking event kinds (HIP-25 Events section)
pub const STAKING_EVENT_STAKED: Uint1 = Uint1::from(1);
pub const STAKING_EVENT_UNSTAKE_REQUESTED: Uint1 = Uint1::from(2);
pub const STAKING_EVENT_UNSTAKED: Uint1 = Uint1::from(3);
pub const STAKING_EVENT_REWARD_DISTRIBUTED: Uint1 = Uint1::from(4);

/// Diamond is locked and earning rewards
pub const DIAMOND_STATUS_STAKED: Uint1 = Uint1::from(4);

/// Diamond is in post-unstake cooldown; HACD and rewards release at unlock_height
pub const DIAMOND_STATUS_STAKING_COOLDOWN: Uint1 = Uint1::from(5);

pub fn diamond_status_allows_transfer(status: &Uint1) -> bool {
    *status == DIAMOND_STATUS_NORMAL
}

pub fn diamond_status_allows_inscription(status: &Uint1) -> bool {
    *status == DIAMOND_STATUS_NORMAL
}

pub fn diamond_status_is_staking_locked(status: &Uint1) -> bool {
    *status == DIAMOND_STATUS_STAKED || *status == DIAMOND_STATUS_STAKING_COOLDOWN
}

pub fn staking_status_label(status: &Uint1) -> &'static str {
    if *status == DIAMOND_STATUS_STAKED {
        return "Staked";
    }
    if *status == DIAMOND_STATUS_STAKING_COOLDOWN {
        return "Cooldown";
    }
    if *status == DIAMOND_STATUS_NORMAL {
        return "Available";
    }
    "unknown"
}

pub fn staking_event_kind_label(kind: &Uint1) -> &'static str {
    if *kind == STAKING_EVENT_STAKED {
        return "Staked";
    }
    if *kind == STAKING_EVENT_UNSTAKE_REQUESTED {
        return "UnstakeRequested";
    }
    if *kind == STAKING_EVENT_UNSTAKED {
        return "Unstaked";
    }
    if *kind == STAKING_EVENT_REWARD_DISTRIBUTED {
        return "RewardDistributed";
    }
    "unknown"
}

/**
 * Global staking pool and reward index.
 * Singleton key: &[2, 3] in MintState (see state/def.rs).
 */
StructFieldStruct!(GlobalStakingState,
    total_staked_shares : Uint5
    global_reward_index : Uint8
    reward_pool_zhu     : Uint8
    paused              : Uint1
    unlock_queue_head   : Uint5
    unlock_queue_tail   : Uint5
    activation_height   : BlockHeight
    event_log_tail      : Uint5
);

impl GlobalStakingState {
    pub fn is_paused(&self) -> bool {
        self.paused.uint() != 0
    }

    pub fn is_active_at(&self, height: u64) -> bool {
        height >= self.activation_height.uint()
    }
}

/**
 * Per-HACD staking metadata.
 * Keyed by DiamondName while status is Staked or Cooldown.
 * Removed when diamond returns to Normal.
 */
StructFieldStruct!(StakingRecord,
    stake_height   : BlockHeight
    unlock_height  : BlockHeight
    reward_index   : Uint8
    pending_reward : Amount
);

impl StakingRecord {
    pub fn is_active_stake(&self) -> bool {
        self.unlock_height.uint() == 0
    }
}

/**
 * FIFO unlock queue entry.
 * Appended on unstake; consumed at block close when unlock_height <= block height.
 * Keyed by monotonic Uint5 entry id.
 */
StructFieldStruct!(StakingUnlockEntry,
    unlock_height : BlockHeight
    diamond       : DiamondName
    staker        : Address
    reward        : Amount
);

/**
 * Append-only HIP-25 event log entry.
 * Keyed by monotonic Uint5 id in MintState (see state/def.rs).
 */
StructFieldStruct!(StakingEvent,
    kind          : Uint1
    height        : BlockHeight
    diamond       : DiamondName
    staker        : Address
    unlock_height : BlockHeight
    reward        : Amount
    shares        : Uint5
);