use odra::{Variable, contract_env, execution_error};
use odra::types::{Address, U256, Balance};
use crate::erc20::{Erc20, Erc20Ref};
use crate::math::{_sqrt, _min};

#[odra::module]
pub struct AmmContract {
    pub lq_token_address: Variable<Address>,
    pub token0_address: Variable<Address>,
    pub token1_address: Variable<Address>,
    pub reserve0: Variable<U256>,
    pub reserve1: Variable<U256>
}

execution_error! {
    pub enum Error{
        InvalidContribution => 1
    }
}

#[odra::module]
impl AmmContract {
    #[odra(init)]
    pub fn init(&mut self, lq_token_address: Address, token0_address: Address, token1_address: Address) {
        self.lq_token_address.set(lq_token_address);
        self.token0_address.set(token0_address);
        self.token1_address.set(token1_address);
        self.reserve0.set(U256::zero());
        self.reserve1.set(U256::zero());
    }
    pub fn add_liquidity(&mut self, amount0: U256, amount1: U256){
        let caller: Address = contract_env::caller();
        // send token0 from caller to contract (must be approved first)
        Erc20Ref::at(&self.token0_address.get().unwrap()).transfer_from(&caller, &contract_env::self_address(), &amount0);
        // send token1 from caller to contract (must be approved first)
        Erc20Ref::at(&self.token1_address.get().unwrap()).transfer_from(&caller, &contract_env::self_address(), &amount1);
        // verify contribution
        let reserve0 = &self.reserve0.get().unwrap();
        let reserve1 = &self.reserve1.get().unwrap();
        if reserve0 > &U256::zero() || reserve1 > &U256::zero(){
            if (reserve0 * amount1) != (reserve1 * amount0){
                odra::contract_env::revert(Error::InvalidContribution)
            };
        }

        // calculate the amount of shares to be minted
        let totalSupply: U256 = Erc20Ref::at(&self.lq_token_address.get().unwrap()).total_supply();
        let mut shares: U256 = U256::zero();
        if totalSupply == U256::zero(){
            shares = _sqrt(amount0 * amount1);
        }
        else{
            let a: U256 = amount0 * totalSupply / reserve0;
            let b: U256 = amount1 * totalSupply / reserve1;
            shares = _min(a, b);
        }
        Erc20Ref::at(&self.lq_token_address.get().unwrap()).mint(&caller, &shares);

        // update reserve => move this to an internal function
        let contract_balance_0 = Erc20Ref::at(&self.token0_address.get().unwrap()).balance_of(&contract_env::self_address());
        let contract_balance_1 =Erc20Ref::at(&self.token1_address.get().unwrap()).balance_of(&contract_env::self_address());
        self.reserve0.set(contract_balance_0);
        self.reserve1.set(contract_balance_1);
    }

    pub fn remove_liquidity(&mut self, shares: U256){
        let caller: Address = contract_env::caller();
    }
    
    pub fn swap(&mut self, amount: U256, from_token_address: Address){
        let caller: Address = contract_env::caller();
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