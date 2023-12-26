#![cfg(test)]
extern crate std;
use super::*;
use crate::{Regisrty, RegisrtyClient};
use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Symbol, Address, Env, IntoVal};

fn deploy_registry<'a>(env: &Env, super_admin:&Address) -> RegisrtyClient<'a> {
    let contract_id = env.register_contract(None, Regisrty);
    let client = RegisrtyClient::new(env, &contract_id);
    // initialize contract
    client.initialize(&super_admin);
    client
}

#[test]
fn test_valid_super_admin() {
    let env = Env::default();
    let admin1 = Address::generate(&env);
    let registry = deploy_registry(&env, &admin1);

    // asset valid super admin
    assert_eq!(admin1, registry.get_super_admin());
}

#[test]
#[should_panic(expected = "Contract already initialized.")]
fn test_double_initialize() {
    let env = Env::default();
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let registry = deploy_registry(&env, &admin1);

    // try to initialize contract again
    registry.initialize(&admin2);
}

#[test]
#[should_panic]
fn test_invalid_super_admin() {
    let env = Env::default();
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let registry = deploy_registry(&env, &admin1);

    // asset invalid super admin
    assert_eq!(admin2, registry.get_super_admin());
}


#[test]
fn test_set_campaign_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let campaign_management = Address::generate(&env);

    let registry = deploy_registry(&env, &admin1);


    registry.set_campaign_management(&campaign_management);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    registry.address.clone(),
                    Symbol::new(&env, "set_campaign_management"),
                    (&campaign_management, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(registry.get_campaign_management(), campaign_management);
}

#[test]
#[should_panic]
fn test_non_admin_set_campaign_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let campaign_management = Address::generate(&env);

    let registry = deploy_registry(&env, &admin1);


    registry.set_campaign_management(&campaign_management);
    assert_eq!(
        env.auths(),
        std::vec![(
            non_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    registry.address.clone(),
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

    let registry = deploy_registry(&env, &admin1);


    registry.set_issuance_management(&issuance_management);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    registry.address.clone(),
                    Symbol::new(&env, "set_issuance_management"),
                    (&issuance_management, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(registry.get_issuance_management(), issuance_management);
}

#[test]
#[should_panic]
fn test_non_admin_set_issunace_management() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let issuance_management = Address::generate(&env);

    let registry = deploy_registry(&env, &admin1);


    registry.set_issuance_management(&issuance_management);
    assert_eq!(
        env.auths(),
        std::vec![(
            non_admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    registry.address.clone(),
                    Symbol::new(&env, "set_issuance_management"),
                    (&issuance_management, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(registry.get_issuance_management(), issuance_management);
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

    let registry = deploy_registry(&env, &admin1);

    // set campaign management address
    registry.set_campaign_management(&campaign_management);
    // set issuance management address
    registry.set_issuance_management(&issuance_management);

    registry.merchant_registration(&merchant, &String::from_str(&env, "Ram"), &String::from_str(&env, "+977-9841123321")
    , &String::from_str(&env, "Medical"), &String::from_str(&env, "Chhauni, Kathmandu"));
    
    assert_eq!(registry.get_unverified_merchants(), vec![&env, merchant.clone()]);

    registry.verify_merchant(&merchant);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    registry.address.clone(),
                    Symbol::new(&env, "verify_merchant"),
                    (&merchant, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(registry.get_verified_merchants(), vec![&env, merchant.clone()]);

    registry.set_campaign_admin(&campaign, &campaign_admin);
    assert_eq!(
        env.auths(),
        std::vec![(
            campaign_management.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    registry.address.clone(),
                    Symbol::new(&env, "set_campaign_admin"),
                    (&campaign, &campaign_admin).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(registry.get_campaign_admin(&campaign), campaign_admin);

    registry.add_deployed_tokens(&token_address);
    assert_eq!(
        env.auths(),
        std::vec![(
            issuance_management.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    registry.address.clone(),
                    Symbol::new(&env, "add_deployed_tokens"),
                    (&token_address, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(registry.get_available_tokens(), vec![&env, token_address.clone()]);
}

#[test]
fn test_update_merchant_info() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let merchant = Address::generate(&env);
    let campaign_management = Address::generate(&env);
    let issuance_management = Address::generate(&env);

    let registry = deploy_registry(&env, &admin1);

    // set campaign management address
    registry.set_campaign_management(&campaign_management);
    // set issuance management address
    registry.set_issuance_management(&issuance_management);

    let proprietor = String::from_str(&env, "Ram");
    let phone_no = String::from_str(&env, "+977-9841123321");
    let store_name = String::from_str(&env, "Medical");
    let location = String::from_str(&env, "Chhauni, Kathmandu");

    registry.merchant_registration(&merchant, &proprietor, &phone_no, &store_name, &location);
    assert_eq!(registry.get_unverified_merchants(), vec![&env, merchant.clone()]);

    registry.verify_merchant(&merchant);

    let merchant_info: Map<String, Val> = map![
            &env,
            (String::from_str(&env, "verified_status"), true.into()),
            (String::from_str(&env, "proprietor"), proprietor.to_val()),
            (String::from_str(&env, "phone_no"), phone_no.to_val()),
            (String::from_str(&env, "store_name"), store_name.to_val()),
            (String::from_str(&env, "location"), location.to_val())
            ];
    
    assert_eq!(registry.get_merchant_info(&merchant), merchant_info);

    assert_eq!(registry.get_verified_merchants(), vec![&env, merchant.clone()]);
    // unverified list needs to be empty
    assert_eq!(registry.get_unverified_merchants(), vec![&env]);

    // admin update merchant info with true verified status
    registry.update_merchant_info(&merchant, &true, &proprietor, &phone_no, &store_name, &location);

    assert_eq!(registry.get_verified_merchants(), vec![&env, merchant.clone()]);
    // unverified list needs to be empty
    assert_eq!(registry.get_unverified_merchants(), vec![&env]);

    // admin update merchant info with false verified status
    registry.update_merchant_info(&merchant, &false, &proprietor, &phone_no, &store_name, &location);

    let merchant_info_after: Map<String, Val> = map![
        &env,
        (String::from_str(&env, "verified_status"), false.into()),
        (String::from_str(&env, "proprietor"), proprietor.to_val()),
        (String::from_str(&env, "phone_no"), phone_no.to_val()),
        (String::from_str(&env, "store_name"), store_name.to_val()),
        (String::from_str(&env, "location"), location.to_val())
        ];

    assert_eq!(registry.get_merchant_info(&merchant), merchant_info_after);

    // verified list needs to be empty
    assert_eq!(registry.get_verified_merchants(), vec![&env]);
    assert_eq!(registry.get_unverified_merchants(), vec![&env, merchant.clone()]);

}

#[test]
#[should_panic(expected = "No registration request.")]
fn test_invalid_merchant_verify() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let merchant = Address::generate(&env);

    let registry = deploy_registry(&env, &admin1);

    registry.verify_merchant(&merchant);
}

#[test]
#[should_panic(expected = "Registration request already sent.")]
fn test_duplicate_merchant_registration_request() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let merchant = Address::generate(&env);

    let registry = deploy_registry(&env, &admin1);

    registry.merchant_registration(&merchant, &String::from_str(&env, "Ram"), &String::from_str(&env, "+977-9841123321")
    , &String::from_str(&env, "Medical"), &String::from_str(&env, "Chhauni, Kathmandu"));
    
    // again try to register with same merchant address, it will panic
    registry.merchant_registration(&merchant, &String::from_str(&env, "Ram"), &String::from_str(&env, "+977-9841123321")
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

    let registry = deploy_registry(&env, &admin1);
    // set campaign management address
    registry.set_campaign_management(&campaign_management);

    registry.set_campaign_admin(&campaign, &campaign_admin);
    assert_eq!(
        env.auths(),
        std::vec![(
            invalid_caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    registry.address.clone(),
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

    let registry = deploy_registry(&env, &admin1);

    // set issuance management address
    registry.set_issuance_management(&issuance_management);

    registry.add_deployed_tokens(&token_address);
    assert_eq!(
        env.auths(),
        std::vec![(
            invalid_caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    registry.address.clone(),
                    Symbol::new(&env, "add_deployed_tokens"),
                    (&token_address, ).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

