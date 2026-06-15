
/// Local dev / HIP-25 testnet only. Mainnet must use a different chain_id.
pub const HIP25_DEV_CHAIN_ID: u64 = 1;

#[derive(Clone)]
pub struct MintConf {
    pub chain_id: u64, // sub chain id
    pub difficulty_adjust_blocks: u64, // height
    pub each_block_target_time: u64, // secs
    pub _test_mul: u64,
    /// HIP-25 soft-fork height; staking rules apply from this block onward.
    pub staking_activation_height: u64,
    /// Dev/testnet: seed one HACD + HAC to a password-derived account at genesis.
    pub hip25_testnet_seed: bool,
    pub hip25_testnet_seed_password: String,
    /// Dev only: min_stake=5 blocks, cooldown=3 blocks (requires hip25_testnet_seed).
    pub hip25_testnet_demo_periods: bool,
}

impl MintConf {

    pub fn new(ini: &IniObj) -> MintConf {

        let sec = ini_section(ini, "mint");

        let cnf = MintConf {
            chain_id: ini_must_u64(&sec, "chain_id", 0),
            difficulty_adjust_blocks: ini_must_u64(&sec, "difficulty_adjust_blocks", 288), // 1 day
            each_block_target_time: ini_must_u64(&sec, "each_block_target_time", 300), // 5 mins
            _test_mul: ini_must_u64(&sec, "_test_mul", 1), // test
            staking_activation_height: ini_must_u64(&sec, "staking_activation_height", 1),
            hip25_testnet_seed: ini_must_bool(&sec, "hip25_testnet_seed", false),
            hip25_testnet_seed_password: ini_must(&sec, "hip25_testnet_seed_password", "hip25test"),
            hip25_testnet_demo_periods: ini_must_bool(&sec, "hip25_testnet_demo_periods", false),
        };

        if let Err(e) = cnf.validate_hip25_dev_flags() {
            panic!("[Config Error] {}", e);
        }

        cnf
    }

    /// Reject dev-only HIP-25 flags on non-dev chain_id (mainnet safety).
    pub fn validate_hip25_dev_flags(&self) -> Result<(), String> {
        if self.hip25_testnet_demo_periods && !self.hip25_testnet_seed {
            return Err(
                "hip25_testnet_demo_periods requires hip25_testnet_seed = true".to_string(),
            );
        }
        if self.hip25_testnet_seed && self.chain_id != HIP25_DEV_CHAIN_ID {
            return Err(format!(
                "hip25_testnet_seed is only allowed on dev chain_id {} (configured chain_id={})",
                HIP25_DEV_CHAIN_ID, self.chain_id
            ));
        }
        if self.hip25_testnet_demo_periods && self.chain_id != HIP25_DEV_CHAIN_ID {
            return Err(format!(
                "hip25_testnet_demo_periods is only allowed on dev chain_id {} (configured chain_id={})",
                HIP25_DEV_CHAIN_ID, self.chain_id
            ));
        }
        Ok(())
    }
}