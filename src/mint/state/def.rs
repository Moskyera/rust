
/**
 * block store
 */
 defineChainStateOperationInstance!{
    Store, MintStore,
    (
        &[2, 1], testttt          , Uint1
    )
    (
        &[2, 11], diamond_smelt   , DiamondName    , DiamondSmelt
    )
}



/**
 * chain state
 */
defineChainStateOperationInstance!{
    State, MintState,
    (
        &[2, 1], total_count      , TotalCount
        &[2, 2], latest_diamond   , DiamondSmelt
        &[2, 3], staking_global   , GlobalStakingState
        &[2, 4], mortgage_global  , GlobalMortgageState
    )
    (
        &[2, 21], diamond_ptr   , DiamondNumber    , DiamondName
        &[2, 22], diamond       , DiamondName      , DiamondSto
        &[2, 23], diamond_owned , Address          , DiamondOwnedForm
        &[2, 24], channel            , ChannelId        , ChannelSto
        &[2, 25], staking_record     , DiamondName      , StakingRecord
        &[2, 26], staking_unlock_entry, Uint5           , StakingUnlockEntry
        &[2, 27], staking_event        , Uint5           , StakingEvent
        &[2, 28], diamond_syslend      , DiamondSyslendId , DiamondSystemLending
    )
}

