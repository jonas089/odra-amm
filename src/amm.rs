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
        // transfer approved tokens from caller to contract
        Erc20Ref::at(&self.token0_address.get().unwrap()).transfer_from(&caller, &contract_env::self_address(), &amount0);
        Erc20Ref::at(&self.token1_address.get().unwrap()).transfer_from(&caller, &contract_env::self_address(), &amount1);
        
        // get reserves and total supply of LQ token
        let reserve0: &U256 = &self.reserve0.get().unwrap();
        let reserve1: &U256 = &self.reserve1.get().unwrap();
        let totalSupply: U256 = Erc20Ref::at(&self.lq_token_address.get().unwrap()).total_supply();
        // verify contribution
        if reserve0 > &U256::zero() || reserve1 > &U256::zero(){
            if (reserve0 * amount1) != (reserve1 * amount0){
                odra::contract_env::revert(Error::InvalidContribution)
            };
        }

        // calculate the amount of shares to be minted
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
        // get balances and total supply of LQ token
        let balance0: U256 = Erc20Ref::at(&self.token0_address.get().unwrap()).balance_of(&contract_env::self_address());
        let balance1: U256 = Erc20Ref::at(&self.token1_address.get().unwrap()).balance_of(&contract_env::self_address());
        let totalSupply: U256 = Erc20Ref::at(&self.lq_token_address.get().unwrap()).total_supply();
        // calculate output amounts
        let amount0: U256 = shares * balance0 / totalSupply;
        let amount1: U256 = shares * balance1 / totalSupply;
        // transfer output amounts and burn LQ token
        Erc20Ref::at(&self.lq_token_address.get().unwrap()).burn(&caller, &shares);
        Erc20Ref::at(&self.token0_address.get().unwrap()).transfer(&caller, &amount0);
        Erc20Ref::at(&self.token1_address.get().unwrap()).transfer(&caller, &amount1);
        // update reserve => move this to an internal function
        let contract_balance_0: U256 = Erc20Ref::at(&self.token0_address.get().unwrap()).balance_of(&contract_env::self_address());
        let contract_balance_1: U256 =Erc20Ref::at(&self.token1_address.get().unwrap()).balance_of(&contract_env::self_address());
        self.reserve0.set(contract_balance_0);
        self.reserve1.set(contract_balance_1);
    }
    
    pub fn swap(&mut self, amount: U256, from_token_address: Address){
        let caller: Address = contract_env::caller();
        let balance0: U256 = Erc20Ref::at(&self.token0_address.get().unwrap()).balance_of(&contract_env::self_address());
        let balance1: U256 = Erc20Ref::at(&self.token1_address.get().unwrap()).balance_of(&contract_env::self_address());
        let token0_address: &Address = &self.token0_address.get().unwrap();
        let token1_address: &Address = &self.token1_address.get().unwrap();
        let mut tokenIn: &Address = token0_address;
        let mut tokenOut: &Address = token1_address;
        let mut reserveIn: U256 = balance0;
        let mut reserveOut: U256 = balance1;
        if &from_token_address == token1_address{
            tokenIn = token1_address;
            tokenOut = token0_address;
            reserveIn = balance1;
            reserveOut = balance0;
        }
        // transfer tokens to contract
        Erc20Ref::at(tokenIn).transfer(&contract_env::caller(), &amount);
        // calculate output amount with 0.3% fee
        let amountInWithFee: U256 = (amount * 997) / 1000;
        let amountOut: U256 = (reserveOut * amountInWithFee) / (reserveIn + amountInWithFee);
        Erc20Ref::at(tokenOut).transfer(&caller, &amountOut);
        // update reserve => move this to an internal function
        let contract_balance_0: U256 = Erc20Ref::at(&self.token0_address.get().unwrap()).balance_of(&contract_env::self_address());
        let contract_balance_1: U256 =Erc20Ref::at(&self.token1_address.get().unwrap()).balance_of(&contract_env::self_address());
        self.reserve0.set(contract_balance_0);
        self.reserve1.set(contract_balance_1);
    }

    pub fn reserve0(&self) -> U256{
        *&self.reserve0.get().unwrap()
    }

    pub fn reserve1(&self) -> U256{
        *&self.reserve1.get().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use odra::types::{Address, U256, Balance};
    use crate::erc20::{Erc20, Erc20Ref, Erc20Deployer};
    use super::{AmmContractDeployer, AmmContractRef};
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
        let amm_contract: Address = AmmContractDeployer::init(lq_token_address, token0_address, token1_address).address().to_owned();
        // fund user with token0 and token1
        Erc20Ref::at(&token0_address).mint(&user, &U256::from(1000u128));
        Erc20Ref::at(&token1_address).mint(&user, &U256::from(1000u128));
        change_caller(user);
        // approve contract as spender
        Erc20Ref::at(&token0_address).approve(&amm_contract, &U256::from(1000u128));
        Erc20Ref::at(&token1_address).approve(&amm_contract, &U256::from(1000u128));
        // add liquidity
        AmmContractRef::at(&amm_contract).add_liquidity(U256::from(1000u128), U256::from(1000u128));
        // verify reserve balance
        let reserve0: U256 = AmmContractRef::at(&amm_contract).reserve0();
        let reserve1: U256 = AmmContractRef::at(&amm_contract).reserve1();

        assert_eq!(reserve0, reserve1);
        assert_eq!(reserve0, U256::from(1000u128));
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