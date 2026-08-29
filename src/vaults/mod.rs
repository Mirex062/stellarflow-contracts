use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    pub fn extend_vault_user_ttl(env: Env, user_address: Address) {
        let key = (symbol_short!("Balance"), user_address);
        env.storage().persistent().extend_ttl(&key, 100_000, 100_000);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Env, testutils::Ledger};

    #[test]
    fn test_extend_vault_user_ttl() {
        let env = Env::default();
        let contract_id = env.register_contract(None, VaultContract);
        let client = VaultContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);

        let key = (symbol_short!("Balance"), user.clone());
        env.storage().persistent().set(&key, &1000i128);

        client.extend_vault_user_ttl(&user);

        let ttl = env.storage().persistent().get_ttl(&key);
        assert!(ttl >= 100_000);
    }
}
