#![cfg(test)]
extern crate std;
use super::*;
use crate::{localcoin, Campaign, CampaignClient};

use soroban_sdk::{testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation}, Symbol, Val, Address, Env, IntoVal};

fn deploy_campaign<'a>(env: &Env, name:String, description:String, no_of_recipients:u32, token_address:Address,
     creator:Address, campaign_management:Address) -> (Address, CampaignClient<'a>) {
        let contract_id = env.register_contract(None, Campaign);
        let client = CampaignClient::new(env, &contract_id);
        // set campaign info
        client.set_campaign_info(&name, &description, &no_of_recipients, &token_address, &creator, &campaign_management);
        (contract_id, client)
}

#[test]
fn test_set_campaign_info() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let campaign_management = Address::generate(&env);
    let token_name = String::from_str(&env, "TEST");

    let localcoin_address = env.register_contract_wasm(None, localcoin::WASM);
    let localcoin_client = localcoin::Client::new(&env, &localcoin_address);
    localcoin_client.initialize(&admin, &7, &token_name, &String::from_str(&env, "TST"));

    // set campaign info
    let name = String::from_str(&env, "Test campaign ");
    let description = String::from_str(&env, "This is test camapaign");
    let no_of_recipients:u32 = 1;

    let (_, campaign) = deploy_campaign(&env, name.clone(), description.clone(), no_of_recipients.clone(),
    localcoin_address.clone(), creator.clone(), campaign_management.clone());

    let mut campaign_info: Map<String, Val> = Map::new(&env);
    campaign_info.set(String::from_str(&env, "name"), name.to_val());
    campaign_info.set(String::from_str(&env, "description"), description.to_val());
    campaign_info.set(String::from_str(&env, "no_of_recipients"), no_of_recipients.into());
    campaign_info.set(String::from_str(&env, "token_address"), localcoin_address.to_val());
    campaign_info.set(String::from_str(&env, "token_name"), token_name.to_val());
    campaign_info.set(String::from_str(&env, "creator"), creator.to_val());
    
    // assert campaign info
    assert_eq!(campaign.get_campaign_info(), campaign_info);

    // assert owner
    assert_eq!(campaign.get_owner(), creator);

    // assert token address
    assert_eq!(campaign.get_token_address(), localcoin_address);

    // assert campaign management address
    assert_eq!(campaign.get_campaign_management(), campaign_management);

    // assert campaign end status
    assert_eq!(campaign.is_ended(), false);
}

#[test]
#[should_panic(expected = "Campaign info already set.")]
fn test_double_set_campaign_info() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let campaign_management = Address::generate(&env);

    let localcoin_address = env.register_contract_wasm(None, localcoin::WASM);
    let localcoin_client = localcoin::Client::new(&env, &localcoin_address);
    localcoin_client.initialize(&admin, &7, &String::from_str(&env, "TEST"), &String::from_str(&env, "TST"));

    // set campaign info
    let name = String::from_str(&env, "Test campaign ");
    let description = String::from_str(&env, "This is test camapaign");
    let no_of_recipients:u32 = 1;

    let (_, campaign) = deploy_campaign(&env, name.clone(), description.clone(), no_of_recipients.clone(),
    localcoin_address.clone(), creator.clone(), campaign_management.clone());
    
    campaign.set_campaign_info(&name, &description, &no_of_recipients, &localcoin_address, &creator, &campaign_management.clone());
}

#[test]
fn test_transfer_token_to_recipient() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let creator = Address::generate(&env);
    let recipient = Address::generate(&env);
    let campaign_management = Address::generate(&env);
    let amount:i128 = 10;

    // set campaign info
    let name = String::from_str(&env, "Test campaign ");
    let description = String::from_str(&env, "This is test camapaign");
    let no_of_recipients:u32 = 1;

    let localcoin_address = env.register_contract_wasm(None, localcoin::WASM);
    let localcoin_client = localcoin::Client::new(&env, &localcoin_address);
    localcoin_client.initialize(&admin1, &7, &String::from_str(&env, "TEST"), &String::from_str(&env, "TST"));

    let (campaign_address, campaign) = deploy_campaign(&env, name.clone(), description.clone(), no_of_recipients.clone(),
     localcoin_address.clone(), creator.clone(), campaign_management.clone());

    localcoin_client.mint(&campaign_address, &100);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    localcoin_address.clone(),
                    Symbol::new(&env, "mint"),
                    (&campaign_address, 100_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    // asssert recipient amount received before receiving localcoin
    assert_eq!(campaign.get_amount_received(&recipient), 0);

    campaign.transfer_tokens_to_recipient(&recipient, &amount);
    assert_eq!(
        env.auths(),
        std::vec![(
            creator.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    campaign_address.clone(),
                    Symbol::new(&env, "transfer_tokens_to_recipient"),
                    (recipient.clone(), amount).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    // asssert recipient amount received after receiving localcoin
    assert_eq!(campaign.get_amount_received(&recipient), amount);

    campaign.transfer_tokens_to_recipient(&recipient, &amount);

    // recipient receives the token again. assert new total amount received
    assert_eq!(campaign.get_amount_received(&recipient), (amount + amount));
}

#[test]
#[should_panic]
fn test_transfer_token_to_recipient_from_non_campaign_creator() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let non_campaign_creator = Address::generate(&env);
    let creator = Address::generate(&env);
    let recipient = Address::generate(&env);
    let campaign_management = Address::generate(&env);
    let amount:i128 = 10;

    // set campaign info
    let name = String::from_str(&env, "Test campaign ");
    let description = String::from_str(&env, "This is test camapaign");
    let no_of_recipients:u32 = 1;

    let localcoin_address = env.register_contract_wasm(None, localcoin::WASM);
    let localcoin_client = localcoin::Client::new(&env, &localcoin_address);

    localcoin_client.initialize(&admin1, &7, &String::from_str(&env, "TEST"), &String::from_str(&env, "TST"));

    let (campaign_address, campaign) = deploy_campaign(&env, name.clone(), description.clone(), no_of_recipients.clone(),
     localcoin_address.clone(), creator.clone(), campaign_management.clone());

    campaign.transfer_tokens_to_recipient(&recipient, &amount);
    assert_eq!(
        env.auths(),
        std::vec![(
            non_campaign_creator.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    campaign_address.clone(),
                    Symbol::new(&env, "transfer_tokens_to_recipient"),
                    (recipient, amount).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}