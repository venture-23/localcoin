#![cfg(test)]
extern crate std;
use super::*;
use crate::{registry, IssuanceManagement, IssuanceManagementClient};
use crate::registry::Client;

use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Symbol, Address, Env, IntoVal};

fn deploy_issuance_management<'a>(env: &Env, registry_addr:&Address) -> (Address, IssuanceManagementClient<'a>) {
    let contract_id = env.register_contract(None, IssuanceManagement);
    let client = IssuanceManagementClient::new(env, &contract_id);
    // initialize contract
    client.initialize(&registry_addr);
    (contract_id, client)
}

fn deploy_registry<'a>(env: &Env, super_admin:&Address) -> (Address, Client<'a>) {
    let contract_id = env.register_contract_wasm(None, registry::WASM);
    let registry_client = registry::Client::new(&env, &contract_id);
    // initialize contract
    registry_client.initialize(&super_admin);
    (contract_id, registry_client)

}

#[test]
fn test_valid_super_admin() {
    let env = Env::default();
    let super_admin = Address::generate(&env);
    let (registry_address, registry) = deploy_registry(&env, &super_admin);

    let (_, issuance_management) = deploy_issuance_management(&env, &registry_address);
    // asset valid super admin
    assert_eq!(issuance_management.get_super_admin(), registry.get_super_admin());
}

#[test]
#[should_panic(expected = "Contract already initialized.")]
fn test_double_initialize() {
    let env = Env::default();
    let super_admin = Address::generate(&env);

    let (registry_address, _) = deploy_registry(&env, &super_admin);
    let (_, issuance_management) = deploy_issuance_management(&env, &registry_address);
    // try to initialize contract again
    issuance_management.initialize(&registry_address);
}

#[test]
fn test_set_campaign_management() {
    let env = Env::default();
    env.mock_all_auths();
    let super_admin = Address::generate(&env);
    let campaign_management = Address::generate(&env);

    let (registry_address, _) = deploy_registry(&env, &super_admin);
    let (_, issuance_management) = deploy_issuance_management(&env, &registry_address);

    issuance_management.set_campaign_management(&campaign_management);
    assert_eq!(
        env.auths(),
        std::vec![(
            super_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    issuance_management.address.clone(),
                    Symbol::new(&env, "set_campaign_management"),
                    (campaign_management.clone(), ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(issuance_management.get_campaign_management(), campaign_management);
}

#[test]
fn test_valid_complete_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let new_merchant = Address::generate(&env);
    let campaign_management = Address::generate(&env);

    let (registry_address, registry) = deploy_registry(&env, &super_admin);
    // request merchant registration
    let proprietor = String::from_str(&env, "Ram");
    let phone_no = String::from_str(&env, "+977-9841123321");
    let store_name = String::from_str(&env, "Medical");
    let location = String::from_str(&env, "Chhauni, Kathmandu");
    registry.merchant_registration(&merchant, &proprietor, &phone_no, &store_name, &location);
    // verify merchant
    registry.verify_merchant(&merchant);

    let (issuance_management_address, issuance_management) = deploy_issuance_management(&env, &registry_address);

    // set campaign management in issuance
    issuance_management.set_campaign_management(&campaign_management);

    // set campaign management in user registry
    registry.set_campaign_management(&campaign_management);
    // set issuance management in user registry
    registry.set_issuance_management(&issuance_management_address);

    let items_associated = vec![&env, String::from_str(&env, "Medicine")];
    let merchants_associated = vec![&env, merchant.clone()];
    let name = String::from_str(&env, "Token1");
    let symbol = String::from_str(&env, "TKN1");

    issuance_management.issue_new_token(&7, &name, &symbol, &items_associated,  &merchants_associated);

    let token_address = issuance_management.get_token_name_address().get_unchecked(String::from_str(&env, "TKN1"));
    assert_eq!(issuance_management.get_items_associated(&token_address), items_associated);

    assert_eq!(issuance_management.get_merchants_associated(&token_address), merchants_associated);

    // add token's item
    let new_items = vec![&env, String::from_str(&env, "Food")];
    issuance_management.add_token_items(&token_address, &new_items);
    assert_eq!(
        env.auths(),
        std::vec![(
            super_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    issuance_management.address.clone(),
                    Symbol::new(&env, "add_token_items"),
                    (&token_address, new_items).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    let updated_items_associated = vec![&env, String::from_str(&env, "Medicine"), String::from_str(&env, "Food")];
    assert_eq!(issuance_management.get_items_associated(&token_address), updated_items_associated);

    // first verify new merchant
    registry.merchant_registration(&new_merchant, &proprietor, &phone_no, &store_name, &location);
    // verify merchant
    registry.verify_merchant(&new_merchant);

    let new_merchant_list = vec![&env, new_merchant.clone()];
    issuance_management.add_token_merchants(&token_address, &new_merchant_list);
    assert_eq!(
        env.auths(),
        std::vec![(
            super_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    issuance_management.address.clone(),
                    Symbol::new(&env, "add_token_merchants"),
                    (token_address.clone(), new_merchant_list).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    let updated_merchants_associated = vec![&env, merchant, new_merchant];
    assert_eq!(issuance_management.get_merchants_associated(&token_address), updated_merchants_associated);

}

#[test]
#[should_panic]
fn test_non_admin_call_issue_new_token() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let non_admin = Address::generate(&env);

    let (registry_address, _) = deploy_registry(&env, &super_admin);
    let (_, issuance_management) = deploy_issuance_management(&env, &registry_address);
    
    let items_associated = vec![&env, String::from_str(&env, "Medicine")];
    let merchants_associated = vec![&env, merchant];

    issuance_management.issue_new_token(&7, &String::from_str(&env, "Token2"), &String::from_str(&env, "TKN2"),
    &items_associated,  &merchants_associated);
    assert_eq!(
        env.auths(),
        std::vec![(
            non_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    issuance_management.address.clone(),
                    Symbol::new(&env, "issue_new_token"),
                    (7, String::from_str(&env, "Token2"), String::from_str(&env, "TKN2"), 
                    items_associated, merchants_associated).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

#[test]
#[should_panic(expected = "Merchants list contains unverified merchant.")]
fn test_unregistered_mechant() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let campaign_management = Address::generate(&env);

    let (registry_address, registry) = deploy_registry(&env, &super_admin);
    let (issuance_management_address, issuance_management) = deploy_issuance_management(&env, &registry_address);

    // set campaign management in issuance
    issuance_management.set_campaign_management(&campaign_management);

    // set campaign management in user registry
    registry.set_campaign_management(&campaign_management);
    // set issuance management in user registry
    registry.set_issuance_management(&issuance_management_address);

    let items_associated = vec![&env, String::from_str(&env, "Medicine")];
    let merchants_associated = vec![&env, merchant];

    // try to issue new token with unregistered merchant in user registry
    issuance_management.issue_new_token(&7, &String::from_str(&env, "Token2"), &String::from_str(&env, "TKN2"),
    &items_associated,  &merchants_associated);
}

#[test]
#[should_panic]
fn test_non_admin_call_add_token_item() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let non_admin = Address::generate(&env);

    let (registry_address, _) = deploy_registry(&env, &super_admin);
    let (_, issuance_management) = deploy_issuance_management(&env, &registry_address);

    let items_associated = vec![&env, String::from_str(&env, "Medicine")];
    let merchants_associated = vec![&env, merchant];

    issuance_management.issue_new_token(&7, &String::from_str(&env, "Token3"), &String::from_str(&env, "TKN3"),
    &items_associated,  &merchants_associated);

    let token_address = issuance_management.get_token_name_address().get_unchecked(String::from_str(&env, "TKN3"));

    let new_items_associated = vec![&env, String::from_str(&env, "Food")];
    issuance_management.add_token_items(&token_address, &new_items_associated);
    assert_eq!(
        env.auths(),
        std::vec![(
            non_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    issuance_management.address.clone(),
                    Symbol::new(&env, "add_token_items"),
                    (token_address, new_items_associated).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

#[test]
#[should_panic(expected = "Token doesn't exist.")]
fn test_add_items_to_non_existing_token() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let non_existing_token = Address::generate(&env);

    let (registry_address, _) = deploy_registry(&env, &super_admin);
    let (_, issuance_management) = deploy_issuance_management(&env, &registry_address);

    // try to add items to non existing token 
    let new_items_associated = vec![&env, String::from_str(&env, "Food")];
    issuance_management.add_token_items(&non_existing_token, &new_items_associated);
}

#[test]
#[should_panic(expected = "Item provided already exist.")]
fn test_add_duplicate_item() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let campaign_management = Address::generate(&env);

    let (registry_address, registry) = deploy_registry(&env, &super_admin);
    // request merchant registration
    let proprietor = String::from_str(&env, "Ram");
    let phone_no = String::from_str(&env, "+977-9841123321");
    let store_name = String::from_str(&env, "Medical");
    let location = String::from_str(&env, "Chhauni, Kathmandu");
    registry.merchant_registration(&merchant, &proprietor, &phone_no, &store_name, &location);
    // verify merchant
    registry.verify_merchant(&merchant);

    let (issuance_management_address, issuance_management) = deploy_issuance_management(&env, &registry_address);

    // set campaign management in issuance
    issuance_management.set_campaign_management(&campaign_management);

    // set campaign management in user registry
    registry.set_campaign_management(&campaign_management);
    // set issuance management in user registry
    registry.set_issuance_management(&issuance_management_address);

    let items_associated = vec![&env, String::from_str(&env, "Medicine")];
    let merchants_associated = vec![&env, merchant];

    issuance_management.issue_new_token(&7, &String::from_str(&env, "Token4"), &String::from_str(&env, "TKN4"),
    &items_associated,  &merchants_associated);

    let token_address = issuance_management.get_token_name_address().get_unchecked(String::from_str(&env, "TKN4"));

    // try to add already existing item
    let new_items_associated = vec![&env, String::from_str(&env, "Medicine")];
    issuance_management.add_token_items(&token_address, &new_items_associated);
}


#[test]
#[should_panic]
fn test_non_admin_call_add_token_merchant() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let non_admin = Address::generate(&env);

    let (registry_address, _) = deploy_registry(&env, &super_admin);
    let (_, issuance_management) = deploy_issuance_management(&env, &registry_address);

    let items_associated = vec![&env, String::from_str(&env, "Medicine")];
    let merchants_associated = vec![&env, merchant.clone()];

    issuance_management.issue_new_token(&7, &String::from_str(&env, "Token5"), &String::from_str(&env, "TKN5"),
    &items_associated,  &merchants_associated);

    let token_address = issuance_management.get_token_name_address().get_unchecked(String::from_str(&env, "TKN5"));

    let new_merchant_list = vec![&env, merchant];
    issuance_management.add_token_merchants(&token_address, &new_merchant_list);
    assert_eq!(
        env.auths(),
        std::vec![(
            non_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    issuance_management.address.clone(),
                    Symbol::new(&env, "add_token_merchants"),
                    (token_address, new_merchant_list).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

#[test]
#[should_panic(expected = "Merchants list contains unverified merchant.")]
fn test_add_unverified_merchant() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let campaign_management = Address::generate(&env);
    let new_merchant = Address::generate(&env);

    let (registry_address, registry) = deploy_registry(&env, &super_admin);
    // request merchant registration
    let proprietor = String::from_str(&env, "Ram");
    let phone_no = String::from_str(&env, "+977-9841123321");
    let store_name = String::from_str(&env, "Medical");
    let location = String::from_str(&env, "Chhauni, Kathmandu");
    registry.merchant_registration(&merchant, &proprietor, &phone_no, &store_name, &location);
    // verify merchant
    registry.verify_merchant(&merchant);
    let (issuance_management_address, issuance_management) = deploy_issuance_management(&env, &registry_address);

    // set campaign management in issuance
    issuance_management.set_campaign_management(&campaign_management);

    // set campaign management in user registry
    registry.set_campaign_management(&campaign_management);
    // set issuance management in user registry
    registry.set_issuance_management(&issuance_management_address);

    let items_associated = vec![&env, String::from_str(&env, "Medicine")];
    let merchants_associated = vec![&env, merchant.clone()];

    issuance_management.issue_new_token(&7, &String::from_str(&env, "Token6"), &String::from_str(&env, "TKN6"),
    &items_associated,  &merchants_associated);

    let token_address = issuance_management.get_token_name_address().get_unchecked(String::from_str(&env, "TKN6"));

    // try to add unverified merchant
    let new_merchants = vec![&env, new_merchant];
    issuance_management.add_token_merchants(&token_address, &new_merchants);
}

#[test]
#[should_panic(expected = "Merchant provided already exist.")]
fn test_add_duplicate_merchant() {
    let env = Env::default();
    env.mock_all_auths();

    let super_admin = Address::generate(&env);
    let merchant = Address::generate(&env);
    let campaign_management = Address::generate(&env);

    let (registry_address, registry) = deploy_registry(&env, &super_admin);
    // request merchant registration
    let proprietor = String::from_str(&env, "Ram");
    let phone_no = String::from_str(&env, "+977-9841123321");
    let store_name = String::from_str(&env, "Medical");
    let location = String::from_str(&env, "Chhauni, Kathmandu");
    registry.merchant_registration(&merchant, &proprietor, &phone_no, &store_name, &location);
    // verify merchant
    registry.verify_merchant(&merchant);
    let (issuance_management_address, issuance_management) = deploy_issuance_management(&env, &registry_address);

    // set campaign management in issuance
    issuance_management.set_campaign_management(&campaign_management);

    // set campaign management in user registry
    registry.set_campaign_management(&campaign_management);
    // set issuance management in user registry
    registry.set_issuance_management(&issuance_management_address);

    let items_associated = vec![&env, String::from_str(&env, "Medicine")];
    let merchants_associated = vec![&env, merchant.clone()];

    issuance_management.issue_new_token(&7, &String::from_str(&env, "Token6"), &String::from_str(&env, "TKN6"),
    &items_associated,  &merchants_associated);

    let token_address = issuance_management.get_token_name_address().get_unchecked(String::from_str(&env, "TKN6"));

    // try to add already existing merchant
    let new_merchant = vec![&env, merchant];
    issuance_management.add_token_merchants(&token_address, &new_merchant);
}

