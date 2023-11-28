#![cfg(test)]

use super::*;
use crate::{UserRegisrty, UserRegisrtyClient};
use soroban_sdk::testutils::{Address as _};
use soroban_sdk::{Env};

fn deploy_user_registry<'a>(env: &Env, super_admin: &Address) -> UserRegisrtyClient<'a> {
    let contract_id = env.register_contract(None, UserRegisrty);
    let client = UserRegisrtyClient::new(env, &contract_id);
    // initialize contract
    client.initialize(&super_admin);
    client
}

#[test]
fn test_valid_super_admin() {
    let env = Env::default();
    let admin1 = Address::random(&env);
    let user_registry = deploy_user_registry(&env, &admin1);

    // asset valid super admin
    assert_eq!(admin1, user_registry.get_super_admin());
}

#[test]
#[should_panic(expected = "Contract already initialized.")]
fn test_double_initialize() {
    let env = Env::default();
    let admin1 = Address::random(&env);
    let admin2 = Address::random(&env);
    let user_registry = deploy_user_registry(&env, &admin1);

    // try to initialize contract again
    user_registry.initialize(&admin2);
}

#[test]
#[should_panic]
fn test_invalid_super_admin() {
    let env = Env::default();
    let admin1 = Address::random(&env);
    let admin2 = Address::random(&env);
    let user_registry = deploy_user_registry(&env, &admin1);

    // asset invalid super admin
    assert_eq!(admin2, user_registry.get_super_admin());
}

