use odra::Variable;
use odra::types::{Address, U256};
use crate::erc20::{Erc20, Erc20Ref};
use crate::math::{_sqrt, _min};

#[odra::module]
pub struct AmmContract {
    pub lq_token_address: Variable<Address>,
    //pub token0_address: Variable<Address>,
    //pub token1_address: Variable<Address>
}

#[odra::module]
impl AmmContract {
    #[odra(init)]
    pub fn init(&mut self, lq_token_address: Address) {
        self.lq_token_address.set(lq_token_address);
    }
    pub fn setup(&self) {
        let NAME: &str = "lq_token";
        let SYMBOL: &str = "lqt";
        let DECIMALS: u8 = 18u8;
        let INITIAL_SUPPLY: u128 = 0u128;
        Erc20Ref::at(&self.lq_token_address.get().unwrap()).init(
            String::from(NAME),
            String::from(SYMBOL),
            DECIMALS,
            &U256::from(INITIAL_SUPPLY)
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::erc20::{Erc20Deployer};
    use super::{AmmContractDeployer};
    #[test]
    fn add_Liquidity() {
        
    }
    #[test]
    fn remove_Liquidity(){

    }
    #[test]
    fn swap(){

    }
}