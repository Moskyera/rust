
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

        let mut cnf = MintConf {
            chain_id: ini_must_u64(&sec, "chain_id", 0),
            difficulty_adjust_blocks: ini_must_u64(&sec, "difficulty_adjust_blocks", 288), // 1 day
            each_block_target_time: ini_must_u64(&sec, "each_block_target_time", 300), // 5 mins
            _test_mul: ini_must_u64(&sec, "_test_mul", 1), // test
            staking_activation_height: ini_must_u64(&sec, "staking_activation_height", 1),
            hip25_testnet_seed: ini_must_bool(&sec, "hip25_testnet_seed", false),
            hip25_testnet_seed_password: ini_must(&sec, "hip25_testnet_seed_password", "hip25test"),
            hip25_testnet_demo_periods: ini_must_bool(&sec, "hip25_testnet_demo_periods", false),
        };

        cnf
    }


}
