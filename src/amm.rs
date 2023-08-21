use odra::{Variable, contract_env, execution_error};
use odra::types::{Address, Balance};
use crate::erc20::{Erc20, Erc20Ref};
use crate::math::{_sqrt, _min};

#[odra::module]
pub struct AmmContract {
    pub lq_token_address: Variable<Address>,
    pub token0_address: Variable<Address>,
    pub token1_address: Variable<Address>,
    pub reserve0: Variable<Balance>,
    pub reserve1: Variable<Balance>
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
        self.reserve0.set(Balance::zero());
        self.reserve1.set(Balance::zero());
    }
    pub fn add_liquidity(&mut self, amount0: Balance, amount1: Balance){
        let caller: Address = contract_env::caller();
        // transfer approved tokens from caller to contract
        Erc20Ref::at(&self.token0_address.get().unwrap()).transfer_from(&caller, &contract_env::self_address(), &amount0);
        Erc20Ref::at(&self.token1_address.get().unwrap()).transfer_from(&caller, &contract_env::self_address(), &amount1);
        
        // get reserves and total supply of LQ token
        let reserve0: &Balance = &self.reserve0.get().unwrap();
        let reserve1: &Balance = &self.reserve1.get().unwrap();
        let totalSupply: Balance = Erc20Ref::at(&self.lq_token_address.get().unwrap()).total_supply();
        // verify contribution
        if reserve0 > &Balance::zero() || reserve1 > &Balance::zero(){
            if (*reserve0 * amount1) != (*reserve1 * amount0){
                odra::contract_env::revert(Error::InvalidContribution)
            };
        }

        // calculate the amount of shares to be minted
        let mut shares: Balance = Balance::zero();
        if totalSupply == Balance::zero(){
            shares = _sqrt(amount0 * amount1);
        }
        else{
            let a: Balance = amount0 * totalSupply / *reserve0;
            let b: Balance = amount1 * totalSupply / *reserve1;
            shares = _min(a, b);
        }
        Erc20Ref::at(&self.lq_token_address.get().unwrap()).mint(&caller, &shares);

        // update reserve => move this to an internal function
        let contract_balance_0 = Erc20Ref::at(&self.token0_address.get().unwrap()).balance_of(&contract_env::self_address());
        let contract_balance_1 =Erc20Ref::at(&self.token1_address.get().unwrap()).balance_of(&contract_env::self_address());
        self.reserve0.set(contract_balance_0);
        self.reserve1.set(contract_balance_1);
    }

    pub fn remove_liquidity(&mut self, shares: Balance){
        let caller: Address = contract_env::caller();
        // get balances and total supply of LQ token
        let balance0: Balance = Erc20Ref::at(&self.token0_address.get().unwrap()).balance_of(&contract_env::self_address());
        let balance1: Balance = Erc20Ref::at(&self.token1_address.get().unwrap()).balance_of(&contract_env::self_address());
        let totalSupply: Balance = Erc20Ref::at(&self.lq_token_address.get().unwrap()).total_supply();
        // calculate output amounts
        let amount0: Balance = shares * balance0 / totalSupply;
        let amount1: Balance = shares * balance1 / totalSupply;
        // transfer output amounts and burn LQ token
        Erc20Ref::at(&self.lq_token_address.get().unwrap()).burn(&caller, &shares);
        Erc20Ref::at(&self.token0_address.get().unwrap()).transfer(&caller, &amount0);
        Erc20Ref::at(&self.token1_address.get().unwrap()).transfer(&caller, &amount1);
        // update reserve => move this to an internal function
        let contract_balance_0: Balance = Erc20Ref::at(&self.token0_address.get().unwrap()).balance_of(&contract_env::self_address());
        let contract_balance_1: Balance =Erc20Ref::at(&self.token1_address.get().unwrap()).balance_of(&contract_env::self_address());
        self.reserve0.set(contract_balance_0);
        self.reserve1.set(contract_balance_1);
    }
    
    pub fn swap(&mut self, amount: Balance, from_token_address: Address){
        let caller: Address = contract_env::caller();
        let balance0: Balance = Erc20Ref::at(&self.token0_address.get().unwrap()).balance_of(&contract_env::self_address());
        let balance1: Balance = Erc20Ref::at(&self.token1_address.get().unwrap()).balance_of(&contract_env::self_address());
        let token0_address: &Address = &self.token0_address.get().unwrap();
        let token1_address: &Address = &self.token1_address.get().unwrap();
        let mut tokenIn: &Address = token0_address;
        let mut tokenOut: &Address = token1_address;
        let mut reserveIn: Balance = balance0;
        let mut reserveOut: Balance = balance1;
        if &from_token_address == token1_address{
            tokenIn = token1_address;
            tokenOut = token0_address;
            reserveIn = balance1;
            reserveOut = balance0;
        }
        // transfer tokens to contract
        Erc20Ref::at(tokenIn).transfer(&contract_env::caller(), &amount);
        // calculate output amount with 0.3% fee
        let amountInWithFee: Balance = (amount * Balance::from(997)) / Balance::from(1000);
        let amountOut: Balance = (reserveOut * amountInWithFee) / (reserveIn + amountInWithFee);
        Erc20Ref::at(tokenOut).transfer(&caller, &amountOut);
        // update reserve => move this to an internal function
        let contract_balance_0: Balance = Erc20Ref::at(&self.token0_address.get().unwrap()).balance_of(&contract_env::self_address());
        let contract_balance_1: Balance = Erc20Ref::at(&self.token1_address.get().unwrap()).balance_of(&contract_env::self_address());
        self.reserve0.set(contract_balance_0);
        self.reserve1.set(contract_balance_1);
    }

    pub fn reserve0(&self) -> Balance{
        *&self.reserve0.get().unwrap()
    }

    pub fn reserve1(&self) -> Balance{
        *&self.reserve1.get().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use odra::types::{Address, Balance};
    use crate::erc20::{Erc20, Erc20Ref, Erc20Deployer};
    use super::{AmmContractDeployer, AmmContractRef};
    #[test]
    fn test_erc20(){
        let user: Address = odra::test_env::get_account(1);
        let lq_token_address: Address = Erc20Deployer::init("TOKEN".to_string(), "TKN".to_string(), 18u8, &Balance::from(0u128)).address().to_owned();
        //odra::test_env::set_caller();
        Erc20Ref::at(&lq_token_address).mint(&user, &Balance::from(10u128));
        let user_balance: Balance = Erc20Ref::at(&lq_token_address).balance_of(&user);
        assert_eq!(user_balance, Balance::from(10u128));
    }
    #[test]
    fn add_Liquidity() {
        let user: Address = odra::test_env::get_account(1);
        let lq_token_address: Address = Erc20Deployer::init("TOKEN".to_string(), "TKN".to_string(), 18u8, &Balance::from(0u128)).address().to_owned();
        let token0_address: Address = Erc20Deployer::init("TOKEN0".to_string(), "TKN0".to_string(), 18u8, &Balance::from(0u128)).address().to_owned();
        let token1_address: Address = Erc20Deployer::init("TOKEN1".to_string(), "TKN1".to_string(), 18u8, &Balance::from(0u128)).address().to_owned();
        let amm_contract: Address = AmmContractDeployer::init(lq_token_address, token0_address, token1_address).address().to_owned();
        // fund user with token0 and token1
        Erc20Ref::at(&token0_address).mint(&user, &Balance::from(1000u128));
        Erc20Ref::at(&token1_address).mint(&user, &Balance::from(1000u128));
        change_caller(user);
        // approve contract as spender
        Erc20Ref::at(&token0_address).approve(&amm_contract, &Balance::from(1000u128));
        Erc20Ref::at(&token1_address).approve(&amm_contract, &Balance::from(1000u128));
        // add liquidity
        AmmContractRef::at(&amm_contract).add_liquidity(Balance::from(1000u128), Balance::from(1000u128));
        // verify reserve balance
        let reserve0: Balance = AmmContractRef::at(&amm_contract).reserve0();
        let reserve1: Balance = AmmContractRef::at(&amm_contract).reserve1();

        assert_eq!(reserve0, reserve1);
        assert_eq!(reserve0, Balance::from(1000u128));
    }
    #[test]
    fn remove_Liquidity(){
        let user: Address = odra::test_env::get_account(1);
        let lq_token_address: Address = Erc20Deployer::init("TOKEN".to_string(), "TKN".to_string(), 18u8, &Balance::from(0u128)).address().to_owned();
        let token0_address: Address = Erc20Deployer::init("TOKEN0".to_string(), "TKN0".to_string(), 18u8, &Balance::from(0u128)).address().to_owned();
        let token1_address: Address = Erc20Deployer::init("TOKEN1".to_string(), "TKN1".to_string(), 18u8, &Balance::from(0u128)).address().to_owned();
        let amm_contract: Address = AmmContractDeployer::init(lq_token_address, token0_address, token1_address).address().to_owned();
        { /* ADD LIQUIDITY */
            // fund user with token0 and token1
            Erc20Ref::at(&token0_address).mint(&user, &Balance::from(1000u128));
            Erc20Ref::at(&token1_address).mint(&user, &Balance::from(1000u128));
            change_caller(user);
            // approve contract as spender
            Erc20Ref::at(&token0_address).approve(&amm_contract, &Balance::from(1000u128));
            Erc20Ref::at(&token1_address).approve(&amm_contract, &Balance::from(1000u128));
            // add liquidity
            AmmContractRef::at(&amm_contract).add_liquidity(Balance::from(1000u128), Balance::from(1000u128));
            // verify reserve balance
            let reserve0: Balance = AmmContractRef::at(&amm_contract).reserve0();
            let reserve1: Balance = AmmContractRef::at(&amm_contract).reserve1();
    
            assert_eq!(reserve0, reserve1);
            assert_eq!(reserve0, Balance::from(1000u128));
        };
        // get shares
        let shares: Balance = Erc20Ref::at(&lq_token_address).balance_of(&user);
        assert_eq!(shares, Balance::from(1000));
        // remove liquidity
        change_caller(user);
        AmmContractRef::at(&amm_contract).remove_liquidity(shares);
        // check redeemed balance
        let user_balance0: Balance = Erc20Ref::at(&token0_address).balance_of(&user);
        let user_balance1: Balance = Erc20Ref::at(&token1_address).balance_of(&user);
        assert_eq!(user_balance0, Balance::from(1000));
        assert_eq!(user_balance1, Balance::from(1000));

    }
    #[test]
    fn swap(){
        let user: Address = odra::test_env::get_account(1);
        let lq_token_address: Address = Erc20Deployer::init("TOKEN".to_string(), "TKN".to_string(), 18u8, &Balance::from(0u128)).address().to_owned();
        let token0_address: Address = Erc20Deployer::init("TOKEN0".to_string(), "TKN0".to_string(), 18u8, &Balance::from(0u128)).address().to_owned();
        let token1_address: Address = Erc20Deployer::init("TOKEN1".to_string(), "TKN1".to_string(), 18u8, &Balance::from(0u128)).address().to_owned();
        let amm_contract: Address = AmmContractDeployer::init(lq_token_address, token0_address, token1_address).address().to_owned();
        { /* ADD LIQUIDITY */
            // fund user with token0 and token1
            Erc20Ref::at(&token0_address).mint(&user, &Balance::from(5000u128));
            Erc20Ref::at(&token1_address).mint(&user, &Balance::from(5000u128));
            change_caller(user);
            // approve contract as spender
            Erc20Ref::at(&token0_address).approve(&amm_contract, &Balance::from(5000u128));
            Erc20Ref::at(&token1_address).approve(&amm_contract, &Balance::from(5000u128));
            // add liquidity
            AmmContractRef::at(&amm_contract).add_liquidity(Balance::from(5000u128), Balance::from(5000u128));
            // verify reserve balance
            let reserve0: Balance = AmmContractRef::at(&amm_contract).reserve0();
            let reserve1: Balance = AmmContractRef::at(&amm_contract).reserve1();
    
            assert_eq!(reserve0, reserve1);
            assert_eq!(reserve0, Balance::from(5000u128));
            assert_eq!(Balance::from(0u128), Erc20Ref::at(&token0_address).balance_of(&user));
            assert_eq!(Balance::from(0u128), Erc20Ref::at(&token1_address).balance_of(&user));
        };
        // perform a swap
        Erc20Ref::at(&token0_address).mint(&user, &Balance::from(1000u128));
        // approve the contract to spend user's token0
        change_caller(user);
        Erc20Ref::at(&token0_address).approve(&amm_contract, &Balance::from(1000u128));
        // swap token0 for token1
        AmmContractRef::at(&amm_contract).swap(Balance::from(1000u128), token0_address);
        // check balances
        assert_eq!(Balance::from(831u128), Erc20Ref::at(&token1_address).balance_of(&user));

    }
    fn change_caller(caller: Address){
        odra::test_env::set_caller(caller);
    }
}