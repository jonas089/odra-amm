use odra::Variable;
use odra::types::{Address, U256, Balance};
use crate::erc20::{Erc20, Erc20Ref};
use crate::math::{_sqrt, _min};

#[odra::module]
pub struct AmmContract {
    pub lq_token_address: Variable<Address>,
    pub token0_address: Variable<Address>,
    pub token1_address: Variable<Address>
}

#[odra::module]
impl AmmContract {
    #[odra(init)]
    pub fn init(&mut self, lq_token_address: Address, token0_address: Address, token1_address: Address) {
        self.lq_token_address.set(lq_token_address);
        self.token0_address.set(token0_address);
        self.token1_address.set(token1_address);
    }
    pub fn add_liquidity(&mut self, amount0: U256, amount1: U256){

    }
    pub fn remove_liquidity(&mut self, shares: U256){

    }
    pub fn swap(&mut self, amount: U256, from_token_address: Address){
        
    }
}

#[cfg(test)]
mod tests {
    use odra::types::{Address, U256, Balance};
    use crate::erc20::{Erc20, Erc20Ref, Erc20Deployer};
    use super::{AmmContractDeployer};
    #[test]
    fn test_erc20(){
        let user: Address = odra::test_env::get_account(1);
        let lq_token_address: Address = Erc20Deployer::init("TOKEN".to_string(), "TKN".to_string(), 18u8, &U256::from(0u128)).address().to_owned();
        //odra::test_env::set_caller();
        Erc20Ref::at(&lq_token_address).mint(&user, &U256::from(10u128));
        let user_balance: U256 = Erc20Ref::at(&lq_token_address).balance_of(&user);
        assert_eq!(user_balance, U256::from(10u128));
    }
    #[test]
    fn add_Liquidity() {
        let user: Address = odra::test_env::get_account(1);
        let lq_token_address: Address = Erc20Deployer::init("TOKEN".to_string(), "TKN".to_string(), 18u8, &U256::from(0u128)).address().to_owned();
        let token0_address: Address = Erc20Deployer::init("TOKEN0".to_string(), "TKN0".to_string(), 18u8, &U256::from(0u128)).address().to_owned();
        let token1_address: Address = Erc20Deployer::init("TOKEN1".to_string(), "TKN1".to_string(), 18u8, &U256::from(0u128)).address().to_owned();

    }
    #[test]
    fn remove_Liquidity(){

    }
    #[test]
    fn swap(){

    }
    fn change_caller(caller: Address){
        odra::test_env::set_caller(caller);
    }
}