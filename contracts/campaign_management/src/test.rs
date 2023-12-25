#![cfg(test)]
extern crate std;
use super::*;
use crate::{user_registry, campaign_contract, localcoin, CampaignManagement, CampaignManagementClient};
use crate::user_registry::Client;

use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Symbol, Address, Env, IntoVal};

mod issuance_management {
    soroban_sdk::contractimport!(
        file =
            "../issuance_management/target/wasm32-unknown-unknown/release/issuance_management.wasm"
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
fn test_set_stable_coin() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let stable_coin = Address::generate(&env);

    let (user_registry_address, _) = deploy_user_registry(&env, &super_admin);
    let (campaign_management_address, campaign_management) = deploy_campaign_management(&env, &user_registry_address);

    // set stable coin address
    campaign_management.set_stable_coin_address(&stable_coin);
    assert_eq!(
        env.auths(),
        std::vec![(
            super_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    campaign_management_address.clone(),
                    Symbol::new(&env, "set_stable_coin_address"),
                    (&stable_coin, ).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

#[test]
#[should_panic]
fn test_set_stable_coin_fron_non_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let stable_coin = Address::generate(&env);

    let (user_registry_address, _) = deploy_user_registry(&env, &super_admin);
    let (campaign_management_address, campaign_management) = deploy_campaign_management(&env, &user_registry_address);

    // set stable coin address
    campaign_management.set_stable_coin_address(&stable_coin);
    assert_eq!(
        env.auths(),
        std::vec![(
            non_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    campaign_management_address.clone(),
                    Symbol::new(&env, "set_stable_coin_address"),
                    (&stable_coin, ).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

#[test]
fn test_valid_create_campaign_flow() {
    let env = Env::default();
    env.mock_all_auths();
    // this test costs more budget than the default allocated so need to set to unlimited
    env.budget().reset_unlimited();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let recipient = Address::generate(&env);
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
    // for test we have deployed localcoin as stable coin
    let stablecoin_address = env.deployer().with_address(super_admin.clone(), salt).deploy(wasm_hash);
    let stablecoin_client = localcoin::Client::new(&env, &stablecoin_address);
    stablecoin_client.initialize(&super_admin, &7, &String::from_str(&env, "USDC Coin"), &String::from_str(&env, "USDC"));
    stablecoin_client.mint(&creator, &100);
    assert_eq!(stablecoin_client.balance(&creator), 100);

    // initialize issuance management
    issuance_management.initialize(&user_registry_address);

    let (campaign_management_address, campaign_management) = deploy_campaign_management(&env, &user_registry_address);

    // set campaign management in issuance
    issuance_management.set_campaign_management(&campaign_management_address);

    // set campaign management in user registry
    user_registry.set_campaign_management(&campaign_management_address);
    assert_eq!(user_registry.get_campaign_management(), campaign_management_address);

    // set issuance management in user registry
    user_registry.set_issuance_management(&issuance_management_address);
    assert_eq!(user_registry.get_issuance_management(), issuance_management_address);

    // set stable coin address
    campaign_management.set_stable_coin_address(&stablecoin_address);

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
    
    let campaigns = campaign_management.get_campaigns();

    // assert detailes stored in campaign_management
    let mut campaign_name: Map<Address, String> = Map::new(&env);
    campaign_name.set(campaigns.first_unchecked(), name.clone());

    assert_eq!(campaign_management.get_campaigns_name(), campaign_name);

    // assert stable coin address
    assert_eq!(campaign_management.get_stable_coin(), stablecoin_address);

    // assert stable coin of campaign management after campaign creation
    assert_eq!(campaign_management.get_balance_of_stable_coin(&campaign_management_address), amount);

    // assert all details in new campaign created

    let mut campaign_info: Map<String, Val> = Map::new(&env);
    campaign_info.set(String::from_str(&env, "name"), name.to_val());
    campaign_info.set(String::from_str(&env, "description"), description.to_val());
    campaign_info.set(String::from_str(&env, "no_of_recipients"), no_of_recipients.into());
    campaign_info.set(String::from_str(&env, "token_address"), token_address.to_val());
    campaign_info.set(String::from_str(&env, "creator"), creator.to_val());
    for campaign in campaigns.clone().into_iter() {
        let camapign_client = campaign_contract::Client::new(&env, &campaign);
        
        assert_eq!(camapign_client.get_campaign_info(), campaign_info);

        assert_eq!(camapign_client.get_token_address(), token_address);

        assert_eq!(camapign_client.get_owner(), creator);

        assert_eq!(camapign_client.get_campaign_balance(), amount);

        // now transfer token to recipient
        camapign_client.transfer_tokens_to_recipient(&recipient, &amount);
    }

    let token_client = localcoin::Client::new(&env, &token_address);
    // assert recipient balance before transfer
    assert_eq!(token_client.balance(&recipient), amount);

    // now recipient transfers the token to merchant
    token_client.transfer(&recipient, &merchant, &amount);

    // assert recipient balance after transfer
    assert_eq!(token_client.balance(&recipient), 0);

    // assert merchant balance after it receives token from recipient
    assert_eq!(token_client.balance(&merchant), amount);

    // assert total supply of token before settelment
    assert_eq!(token_client.total_supply(), amount);

    // assert stable coin balance of super admin before settelment of a campaign
    assert_eq!(campaign_management.get_balance_of_stable_coin(&super_admin), 0);

    // now mwechant requests for settelment
    campaign_management.request_campaign_settelment(&merchant, &amount, &token_address);

    // assert merchant balance after settelment
    assert_eq!(token_client.balance(&merchant), 0);

    // asert total supply of token after settelment
    assert_eq!(token_client.total_supply(), 0);

    // assert stable coin balance of campaign management after settelment of a campaign
    assert_eq!(campaign_management.get_balance_of_stable_coin(&campaign_management_address), 0);

    // assert stable coin balance of super admin before settelment of a campaign
    assert_eq!(campaign_management.get_balance_of_stable_coin(&super_admin), amount);
}

#[test]
#[should_panic(expected = "Invalid token passed in param.")]
fn test_request_settelment_of_invalid_token() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let token_address = Address::generate(&env);
    let amount:i128 = 10;

    let (user_registry_address, _) = deploy_user_registry(&env, &super_admin);
    let (_, campaign_management) = deploy_campaign_management(&env, &user_registry_address);

    // requests for settelment with invalid token
    campaign_management.request_campaign_settelment(&merchant, &amount, &token_address);
}

#[test]
#[should_panic(expected = "Amount cannot be equal or less than zero.")]
fn test_request_settelment_of_0_token_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let token_address = Address::generate(&env);
    let amount:i128 = 0;

    let (user_registry_address, _) = deploy_user_registry(&env, &super_admin);
    let (_, campaign_management) = deploy_campaign_management(&env, &user_registry_address);

    // requests for settelment with invalid token
    campaign_management.request_campaign_settelment(&merchant, &amount, &token_address);
}

#[test]
#[should_panic(expected = "Amount cannot be equal or less than zero.")]
fn test_request_settelment_of_negative_token_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let token_address = Address::generate(&env);
    let amount:i128 = -10;

    let (user_registry_address, _) = deploy_user_registry(&env, &super_admin);
    let (_, campaign_management) = deploy_campaign_management(&env, &user_registry_address);

    // requests for settelment with invalid token
    campaign_management.request_campaign_settelment(&merchant, &amount, &token_address);
}

#[test]
#[should_panic(expected = "Insufficient token for settelment.")]
fn test_request_settelment_for_insufficient_amount() {
    let env = Env::default();
    env.mock_all_auths();
    // this test costs more budget than the default allocated so need to set to unlimited
    env.budget().reset_unlimited();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let amount:i128 = 10;
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
    // for test we have deployed localcoin as stable coin
    let stablecoin_address = env.deployer().with_address(super_admin.clone(), salt).deploy(wasm_hash);
    let stablecoin_client = localcoin::Client::new(&env, &stablecoin_address);
    stablecoin_client.initialize(&super_admin, &7, &String::from_str(&env, "USDC Coin"), &String::from_str(&env, "USDC"));
    stablecoin_client.mint(&creator, &100);
    assert_eq!(stablecoin_client.balance(&creator), 100);

    // initialize issuance management
    issuance_management.initialize(&user_registry_address);

    let (campaign_management_address, campaign_management) = deploy_campaign_management(&env, &user_registry_address);

    // set campaign management in issuance
    issuance_management.set_campaign_management(&campaign_management_address);

    // set campaign management in user registry
    user_registry.set_campaign_management(&campaign_management_address);
    assert_eq!(user_registry.get_campaign_management(), campaign_management_address);

    // set issuance management in user registry
    user_registry.set_issuance_management(&issuance_management_address);
    assert_eq!(user_registry.get_issuance_management(), issuance_management_address);

    // set stable coin address
    campaign_management.set_stable_coin_address(&stablecoin_address);

    let items_associated = vec![&env, String::from_str(&env, "Medicine")];
    let merchants_associated = vec![&env, merchant.clone()];
    issuance_management.issue_new_token(&7, &String::from_str(&env, "Token1"), &String::from_str(&env, "TKN1"),
    &items_associated,  &merchants_associated);

    let token_list = user_registry.get_available_tokens();
    let token_address = token_list.first_unchecked();

    // requests for settelment with invalid token
    campaign_management.request_campaign_settelment(&merchant, &amount, &token_address);
}