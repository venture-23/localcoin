#![cfg(test)]
extern crate std;
use super::*;
use crate::{user_registry, CampaignManagement, CampaignManagementClient};
use crate::user_registry::Client;

use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Symbol, Address, Env, IntoVal};

mod issuance_management {
    soroban_sdk::contractimport!(
        file =
            "../issuance_management/target/wasm32-unknown-unknown/release/issuance_management.wasm"
    );
}

mod localcoin {
    soroban_sdk::contractimport!(
        file =
            "../localcoin/target/wasm32-unknown-unknown/release/localcoin.wasm"
    );
}

fn deploy_campaign_management<'a>(env: &Env, user_registry_addr:&Address) -> (Address, CampaignManagementClient<'a>) {
    let contract_id = env.register_contract(None, CampaignManagement);
    let client = CampaignManagementClient::new(env, &contract_id);
    // initialize contract
    client.initialize(&user_registry_addr);
    (contract_id, client)
}

fn deploy_user_registry<'a>(env: &Env, super_admin:&Address) -> (Address, Client<'a>) {
    let contract_id = env.register_contract_wasm(None, user_registry::WASM);
    let user_registry_client = user_registry::Client::new(&env, &contract_id);
    // initialize contract
    user_registry_client.initialize(&super_admin);
    (contract_id ,user_registry_client)
}

#[test]
fn test_valid_super_admin() {
    let env = Env::default();
    let super_admin = Address::generate(&env);

    let (user_registry_address, user_registry) = deploy_user_registry(&env, &super_admin);
    let (_, campaign_management) = deploy_campaign_management(&env, &user_registry_address);
    
    // asset valid super admin
    assert_eq!(campaign_management.get_super_admin(), user_registry.get_super_admin());
}

#[test]
#[should_panic(expected = "Contract already initialized.")]
fn test_double_initialize() {
    let env = Env::default();
    let super_admin = Address::generate(&env);

    let (user_registry_address, _) = deploy_user_registry(&env, &super_admin);
    let (_, campaign_management) = deploy_campaign_management(&env, &user_registry_address);

    // try to initialize contract again
    campaign_management.initialize(&user_registry_address);
}

#[test]
fn test_valid_create_campaign_flow() {
    let env = Env::default();
    env.mock_all_auths();
    // this test costs more budget than the default allocated so need to set to unlimited
    env.budget().reset_unlimited();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    // let stable_coin_address = Address::generate(&env);

    let creator = Address::generate(&env);

    let (user_registry_address, user_registry) = deploy_user_registry(&env, &super_admin);

    // request merchant registration
    let proprietor = String::from_str(&env, "Ram");
    let phone_no = String::from_str(&env, "+977-9841123321");
    let store_name = String::from_str(&env, "Medical");
    let location = String::from_str(&env, "Chhauni, Kathmandu");
    user_registry.merchant_registration(&merchant, &proprietor, &phone_no, &store_name, &location);
    // verify merchant
    user_registry.verify_merchant(&merchant);

    assert_eq!(user_registry.get_super_admin(), super_admin);

    assert_eq!(user_registry.get_verified_merchants(), vec![&env, merchant.clone()]);

    // deploy issuance management
    let wasm_hash = env.deployer().upload_contract_wasm(issuance_management::WASM);
    let salt = BytesN::from_array(&env, &[1; 32]);
    let issuance_management_address = env.deployer().with_address(super_admin.clone(), salt).deploy(wasm_hash);
    let issuance_management = issuance_management::Client::new(&env, &issuance_management_address);

    let wasm_hash = env.deployer().upload_contract_wasm(localcoin::WASM);
    let salt = BytesN::from_array(&env, &[2; 32]);
    let localcoin_address = env.deployer().with_address(super_admin.clone(), salt).deploy(wasm_hash);
    let localcoin_client = localcoin::Client::new(&env, &localcoin_address);
    localcoin_client.initialize(&super_admin, &7, &String::from_str(&env, "USDC Coin"), &String::from_str(&env, "USDC"));
    localcoin_client.mint(&creator, &100);
    assert_eq!(localcoin_client.balance_of(&creator), 100);

    // initialize issuance management
    issuance_management.initialize(&user_registry_address);

    let (campaign_management_address, campaign_management) = deploy_campaign_management(&env, &user_registry_address);

    // set campaign management in issuance
    issuance_management.set_campaign_management(&campaign_management_address);

    // set campaign management in user registry
    user_registry.set_campaign_managment(&campaign_management_address);
    assert_eq!(user_registry.get_campaign_management(), campaign_management_address);

    // set issuance management in user registry
    user_registry.set_issuance_managment(&issuance_management_address);
    assert_eq!(user_registry.get_issuance_management(), issuance_management_address);

    // set stable coin address
    campaign_management.set_stable_coin_address(&localcoin_address);

    let items_associated = vec![&env, String::from_str(&env, "Medicine")];
    let merchants_associated = vec![&env, merchant.clone()];
    issuance_management.issue_new_token(&7, &String::from_str(&env, "Token1"), &String::from_str(&env, "TKN1"),
    &items_associated,  &merchants_associated);

    // create campaign
    let name = String::from_str(&env, "Test campaign ");
    let description = String::from_str(&env, "This is test camapaign");
    let no_of_recipients:u32 = 1; 
    let amount:i128 = 10;

    let token_list = user_registry.get_available_tokens();
    let token_address = token_list.first_unchecked();

    campaign_management.create_campaign(&name, &description, &no_of_recipients, &token_address, &amount, &creator);

}