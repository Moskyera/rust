
/**
 * HIP-2 v2.1: HACD system mortgage loan (HAC IOU against bid-burn collateral).
 */

/// ~35 days per committed period at mainnet cadence (~5 min/block).
pub const MORTGAGE_PERIOD_BLOCKS: u64 = 10_000;

/// ~1 year of blocks at 5 min/block (365 × 24 × 12).
pub const MORTGAGE_BLOCKS_PER_YEAR: u64 = 105_120;

/// 1% origination fee on loan principal → burn (v2.1).
pub const MORTGAGE_ORIGINATION_FEE_BPS: u64 = 100;

/// Private grace: zero ransom interest for this many elapsed periods (~3.5 months).
pub const MORTGAGE_EARLY_GRACE_PERIODS: u64 = 3;

/// After grace, before private midpoint: 0.1% of principal per elapsed period.
pub const MORTGAGE_EARLY_INTEREST_BPS_PER_PERIOD: u64 = 10;

/// Flat annual rate (3%); borrow_period T sets windows only, not the rate multiplier.
pub const MORTGAGE_APR_BPS: u64 = 300;

/// Dutch auction floor: 103% of principal (v2.1).
pub const MORTGAGE_AUCTION_FLOOR_BPS: u64 = 10_300;

/// Default max outstanding IOU (zhu); governance may lower before mainnet.
pub const MORTGAGE_DEFAULT_MAX_OUTSTANDING_ZHU: u64 = 800_000_000_000_000; // 8M HAC @ 8 decimals scale

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MortgageRedeemPhase {
    Private,
    Public,
    Auction,
}

impl MortgageRedeemPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Public => "public",
            Self::Auction => "auction",
        }
    }
}

StructFieldStruct!(DiamondSystemLending,
    is_ransomed           : Uint1
    create_block_height   : BlockHeight
    main_address          : Address
    mortgage_diamonds     : DiamondNameListMax200
    loan_principal        : Amount
    borrow_period         : Uint1
    ransom_block_height   : BlockHeight
    ransom_address        : Address
);

impl DiamondSystemLending {
    pub fn redeemed(&self) -> bool {
        self.is_ransomed.uint() != 0
    }

    pub fn mark_ransomed(&mut self, height: u64, redeemer: &Address) {
        self.is_ransomed = Uint1::from(1);
        self.ransom_block_height = BlockHeight::from(height);
        self.ransom_address = redeemer.clone();
    }
}

StructFieldStruct!(GlobalMortgageState,
    outstanding_ioo_zhu              : Uint8
    cumulative_loan_zhu              : Uint8
    cumulative_ransom_burn_zhu       : Uint8
    cumulative_origination_burn_zhu  : Uint8
    active_contracts                 : Uint5
    activation_height                : BlockHeight
    max_outstanding_ioo_zhu          : Uint8
    demo_period_blocks               : Uint5
);

impl GlobalMortgageState {
    pub fn is_active_at(&self, height: u64) -> bool {
        let act = self.activation_height.uint();
        act > 0 && height >= act
    }

    pub fn effective_period_blocks(&self) -> u64 {
        let demo = self.demo_period_blocks.uint();
        if demo > 0 {
            demo
        } else {
            MORTGAGE_PERIOD_BLOCKS
        }
    }
}

pub fn mortgage_validate_lending_id(id: &DiamondSyslendId) -> Ret<()> {
    let bytes = id.as_ref();
    if bytes.len() != DIAMOND_SYSLEND_ID_SIZE {
        return errf!("mortgage lending id length must be {}", DIAMOND_SYSLEND_ID_SIZE);
    }
    if bytes[0] == 0 || bytes[bytes.len() - 1] == 0 {
        return errf!("mortgage lending id format error");
    }
    Ok(())
}