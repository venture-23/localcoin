#![cfg(test)]
extern crate std;
use super::*;
use crate::{UserRegisrty, UserRegisrtyClient};
use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Symbol, Address, Env, IntoVal};

fn deploy_user_registry<'a>(env: &Env, super_admin:&Address) -> UserRegisrtyClient<'a> {
    let contract_id = env.register_contract(None, UserRegisrty);
    let client = UserRegisrtyClient::new(env, &contract_id);
    // initialize contract
    client.initialize(&super_admin);
    client
}

#[test]
fn test_valid_super_admin() {
    let env = Env::default();
    let admin1 = Address::generate(&env);
    let user_registry = deploy_user_registry(&env, &admin1);

    // asset valid super admin
    assert_eq!(admin1, user_registry.get_super_admin());
}

#[test]
#[should_panic(expected = "Contract already initialized.")]
fn test_double_initialize() {
    let env = Env::default();
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let user_registry = deploy_user_registry(&env, &admin1);

    // try to initialize contract again
    user_registry.initialize(&admin2);
}

#[test]
#[should_panic]
fn test_invalid_super_admin() {
    let env = Env::default();
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let user_registry = deploy_user_registry(&env, &admin1);

    // asset invalid super admin
    assert_eq!(admin2, user_registry.get_super_admin());
}


#[test]
fn test_set_campaign_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let campaign_management = Address::generate(&env);

    let user_registry = deploy_user_registry(&env, &admin1);


    user_registry.set_campaign_management(&campaign_management);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    user_registry.address.clone(),
                    Symbol::new(&env, "set_campaign_management"),
                    (&campaign_management, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(user_registry.get_campaign_management(), campaign_management);
}

#[test]
#[should_panic]
fn test_non_admin_set_campaign_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let campaign_management = Address::generate(&env);

    let user_registry = deploy_user_registry(&env, &admin1);


    user_registry.set_campaign_management(&campaign_management);
    assert_eq!(
        env.auths(),
        std::vec![(
            non_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    user_registry.address.clone(),
                    Symbol::new(&env, "set_campaign_management"),
                    (&campaign_management, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

#[test]
fn test_set_issunace_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let issuance_management = Address::generate(&env);

    let user_registry = deploy_user_registry(&env, &admin1);


    user_registry.set_issuance_management(&issuance_management);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    user_registry.address.clone(),
                    Symbol::new(&env, "set_issuance_management"),
                    (&issuance_management, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(user_registry.get_issuance_management(), issuance_management);
}

#[test]
#[should_panic]
fn test_non_admin_set_issunace_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let issuance_management = Address::generate(&env);

    let user_registry = deploy_user_registry(&env, &admin1);


    user_registry.set_issuance_management(&issuance_management);
    assert_eq!(
        env.auths(),
        std::vec![(
            non_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    user_registry.address.clone(),
                    Symbol::new(&env, "set_issuance_management"),
                    (&issuance_management, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(user_registry.get_issuance_management(), issuance_management);
}


#[test]
fn test_valid_complete_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let merchant = Address::generate(&env);
    let campaign_management = Address::generate(&env);
    let issuance_management = Address::generate(&env);
    let campaign = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    let user_registry = deploy_user_registry(&env, &admin1);

    // set campaign management address
    user_registry.set_campaign_management(&campaign_management);
    // set issuance management address
    user_registry.set_issuance_management(&issuance_management);

    user_registry.merchant_registration(&merchant, &String::from_str(&env, "Ram"), &String::from_str(&env, "+977-9841123321")
    , &String::from_str(&env, "Medical"), &String::from_str(&env, "Chhauni, Kathmandu"));
    
    user_registry.verify_merchant(&merchant);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    user_registry.address.clone(),
                    Symbol::new(&env, "verify_merchant"),
                    (&merchant, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(user_registry.get_verified_merchants(), vec![&env, merchant.clone()]);

    user_registry.set_campaign_admin(&campaign, &campaign_admin);
    assert_eq!(
        env.auths(),
        std::vec![(
            campaign_management.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    user_registry.address.clone(),
                    Symbol::new(&env, "set_campaign_admin"),
                    (&campaign, &campaign_admin).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(user_registry.get_campaign_admin(&campaign), campaign_admin);

    user_registry.add_deployed_tokens(&token_address);
    assert_eq!(
        env.auths(),
        std::vec![(
            issuance_management.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    user_registry.address.clone(),
                    Symbol::new(&env, "add_deployed_tokens"),
                    (&token_address, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(user_registry.get_available_tokens(), vec![&env, token_address.clone()]);
}

#[test]
#[should_panic(expected = "No registration request.")]
fn test_invalid_merchant_verify() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let merchant = Address::generate(&env);

    let user_registry = deploy_user_registry(&env, &admin1);

    user_registry.verify_merchant(&merchant);
}

#[test]
#[should_panic(expected = "Registration request already sent.")]
fn test_duplicate_merchant_registration_request() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let merchant = Address::generate(&env);

    let user_registry = deploy_user_registry(&env, &admin1);

    user_registry.merchant_registration(&merchant, &String::from_str(&env, "Ram"), &String::from_str(&env, "+977-9841123321")
    , &String::from_str(&env, "Medical"), &String::from_str(&env, "Chhauni, Kathmandu"));
    
    // again try to register with same merchant address, it will panic
    user_registry.merchant_registration(&merchant, &String::from_str(&env, "Ram"), &String::from_str(&env, "+977-9841123321")
    , &String::from_str(&env, "Medical"), &String::from_str(&env, "Chhauni, Kathmandu"));
}

#[test]
#[should_panic]
fn test_invalid_caller_set_campaign_admin() {
    // set_campaign_admin can only be called by campaign_management
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let invalid_caller = Address::generate(&env);
    let campaign_management = Address::generate(&env);
    let campaign = Address::generate(&env);
    let campaign_admin = Address::generate(&env);

    let user_registry = deploy_user_registry(&env, &admin1);
    // set campaign management address
    user_registry.set_campaign_management(&campaign_management);

    user_registry.set_campaign_admin(&campaign, &campaign_admin);
    assert_eq!(
        env.auths(),
        std::vec![(
            invalid_caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    user_registry.address.clone(),
                    Symbol::new(&env, "set_campaign_admin"),
                    (&campaign, &campaign_admin).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

#[test]
#[should_panic]
fn test_invalid_caller_add_deployed_tokens() {
    // add_deployed_tokens can only be called by issuance_management
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let invalid_caller = Address::generate(&env);
    let issuance_management = Address::generate(&env);
    let token_address = Address::generate(&env);

    let user_registry = deploy_user_registry(&env, &admin1);

    // set issuance management address
    user_registry.set_issuance_management(&issuance_management);

    user_registry.add_deployed_tokens(&token_address);
    assert_eq!(
        env.auths(),
        std::vec![(
            invalid_caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    user_registry.address.clone(),
                    Symbol::new(&env, "add_deployed_tokens"),
                    (&token_address, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

