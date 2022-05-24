use once_cell::race::OnceBox;
use web3::ethabi;

macro_rules! contract_methods(
    ($module:ident, $abi:ident, { $($name:literal => $function:ident),*$(,)? }) => {
        pub mod $module {
            use super::*;

            pub fn abi() -> &'static ethabi::Contract {
                static ABI: OnceBox<ethabi::Contract> = OnceBox::new();
                ABI.get_or_init(|| Box::new(serde_json::from_str($abi).expect("Shouldn't fail")))
            }

            $(pub fn $function() -> &'static ethabi::Function {
                static ABI: OnceBox<ethabi::Function> = OnceBox::new();
                ABI.get_or_init(|| Box::new(abi().function($name).expect("Shouldn't fail").clone()))
            })*
        }
    }
);

contract_methods!(erc_20, ERC_20_ABI, {
    "symbol" => symbol,
    "decimals" => decimals,
    "balanceOf" => balance_of,
});

contract_methods!(vault, VAULT_ABI, {
    "token" => token,
    "totalAssets" => total_assets,
    "withdrawLimitPerPeriod" => withdraw_limit_per_period,
    "withdrawalPeriods" => withdrawal_periods,
});

contract_methods!(bridge, BRIDGE_ABI, {
    "lastRound" => last_round,
    "rounds" => rounds,
});

static ERC_20_ABI: &str = include_str!("ERC20.json");
static VAULT_ABI: &str = include_str!("IVault.json");
static BRIDGE_ABI: &str = include_str!("Bridge.json");
