#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, BytesN, Env, String, Vec, Val, vec};

mod campaign_contract {
    soroban_sdk::contractimport!(
        file =
            "../campaign/target/wasm32-unknown-unknown/release/campaign.wasm"
    );
}

mod localcoin {
    soroban_sdk::contractimport!(
        file =
            "../localcoin/target/wasm32-unknown-unknown/release/localcoin.wasm"
    );
}

mod user_registry {
    soroban_sdk::contractimport!(
        file =
            "../user_registry/target/wasm32-unknown-unknown/release/user_registry.wasm"
    );
}

#[contracttype]
pub struct CampaignDetail {
    pub campaign: Address,
    pub token: Address,
    pub token_minted: i128,
    pub info: Vec<Val>
}

#[derive(Clone)]
#[contracttype]
pub enum DataKeys {
    UserRegistry,
    StableCoin,
    Campaigns,
    SaltCounter,
    CreatorCampaigns(Address)
}

#[contract]
pub struct CampaignManagement;

#[contractimpl]
impl CampaignManagement {
    // initialize contract
    pub fn initialize(env:Env, user_registey:Address) {
        if Self::has_user_registry(env.clone()) {
            panic!("Contract already initialized.")
        }
        let key = DataKeys::UserRegistry;
        env.storage().instance().set(&key, &user_registey)
    }

    pub fn set_user_registry(env:Env, address:Address) {
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();
        
        let key = DataKeys::UserRegistry;
        env.storage().instance().set(&key, &address)
    }

    pub fn set_stable_coin_address(env:Env, address:Address) {
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();
        
        let key = DataKeys::StableCoin;
        env.storage().instance().set(&key, &address)
    }

    pub fn create_campaign(env:Env, name:String, description:String, no_of_recipients:u32,
         token_address:Address, amount:i128, creator: Address) {

            creator.require_auth();

            let user_registry_addr = Self::get_user_registry(env.clone());
            let user_registry_client = user_registry::Client::new(&env, &user_registry_addr);

            let valid_tokens = user_registry_client.get_available_tokens();
            if !valid_tokens.contains(token_address.clone()) {
                panic!("Invalid token passed in param.")
            }

            // transfer stable coin to campaign management from 'creator' address
            let stable_coin_addr = Self::get_stable_coin(env.clone());
            let stable_coin_client = token::Client::new(&env, &stable_coin_addr);
            stable_coin_client.transfer(&creator, &env.current_contract_address(), &amount);
            
            let wasm_hash = env.deployer().upload_contract_wasm(campaign_contract::WASM);

            // dynamic salt count to deploy multiple contracts
            let salt_key = DataKeys::SaltCounter;
            let salt_count: u32 = env.storage().instance().get::<DataKeys, u32>(&salt_key).unwrap_or(0);
            let salt = BytesN::from_array(&env, &[salt_count.try_into().unwrap(); 32]);

            // deploy campaign contract
            let campaign_contract_addr = env.deployer().with_address(creator.clone(), salt).deploy(wasm_hash);

            // increase salt counter 
            env.storage().instance().set(&salt_key, &(salt_count + 1));

            // call set_campaign_info on camapign contract through client
            let campaign_client = campaign_contract::Client::new(&env, &campaign_contract_addr);
            campaign_client.set_campaign_info(&name, &description, &no_of_recipients, &token_address, &creator); 

            // mint stable coin equivalent tokens to campaign contract
            let token_client = localcoin::Client::new(&env, &token_address);
            token_client.mint(&campaign_contract_addr, &amount);

            // set campaign admin in user_registry contract
            user_registry_client.set_campaign_admin(&campaign_contract_addr, &creator);
            
            // store all campaign
            let campaign_key = DataKeys::Campaigns;
            let mut campaign_list = Self::get_campaigns(env.clone());
            campaign_list.push_back(campaign_contract_addr.clone());
            env.storage().instance().set(&campaign_key, &campaign_list);

            // store campaigns info of the creator
            let key = DataKeys:: CreatorCampaigns(creator.clone());
            let mut creator_campaigns = Self::get_creator_campaigns(env.clone(), creator);

            let mut new_campaign_info: Vec<Val> = Vec::new(&env);
            new_campaign_info.push_back(name.to_val());
            new_campaign_info.push_back(description.to_val());
            new_campaign_info.push_back(no_of_recipients.into());

            let campaign_value = CampaignDetail {
                campaign: campaign_contract_addr,
                token: token_address,
                token_minted: amount,
                info: new_campaign_info
            };
            creator_campaigns.push_back(campaign_value);
            env.storage().instance().set(&key, &creator_campaigns)
    }

    pub fn request_campaign_settelment(env:Env, from:Address, amount:i128, token_address:Address) {
        // transaction should be sent from 'from' addesss
        from.require_auth();

        let user_registry = Self::get_user_registry(env.clone());
        let user_registry_client = user_registry::Client::new(&env, &user_registry);

        let valid_tokens = user_registry_client.get_available_tokens();
        if !valid_tokens.contains(token_address.clone()) {
            panic!("Invalid token passed in param.")
        }

        let merchants = user_registry_client.get_verified_merchants();
        if !merchants.contains(&from) {
            panic!("Caller not merchant.")
        }

        let token_client = localcoin::Client::new(&env, &token_address);
        // verify balance of merchant
        let balance = token_client.balance_of(&from);
        if !balance >= amount {
            panic!("Insufficient token for settelment.")
        }

        // campaign_management burns the token from merchant 'from'
        token_client.burn(&from, &amount);

        // transfer stable coin to super admin from campaign management address (current contract)
        let stable_coin_addr = Self::get_stable_coin(env.clone());
        let super_admin = Self::get_super_admin(env.clone());
        let stable_coin_client = token::Client::new(&env, &stable_coin_addr);
        stable_coin_client.transfer(&env.current_contract_address(), &super_admin, &amount);
    }

    pub fn get_campaigns(env:Env) -> Vec<Address> {
        let key = DataKeys::Campaigns;
        if let Some(campaigns) = env.storage().instance().get::<DataKeys, Vec<Address>>(&key) {
            campaigns
        } else {
            vec![&env]
        }
    }

    pub fn get_creator_campaigns(env:Env, creator:Address) -> Vec<CampaignDetail> {
        let key = DataKeys:: CreatorCampaigns(creator);
        if let Some(campaigns_info) = env.storage().instance().get::<DataKeys, Vec<CampaignDetail>>(&key) {
            campaigns_info
        } else {
            vec![&env] 
        }
    }

    pub fn get_user_registry(env:Env) -> Address {
        let key = DataKeys::UserRegistry;
        if let Some(user_registry_addr) = env.storage().instance().get::<DataKeys, Address>(&key) {
            user_registry_addr
        } else {
            panic!("Address not set.")
        }
    }

    pub fn get_stable_coin(env:Env) -> Address {
        let key = DataKeys::StableCoin;
        if let Some(stable_coin_addr) = env.storage().instance().get::<DataKeys, Address>(&key) {
            stable_coin_addr
        } else {
            panic!("Address not set.")
        }
    }

    pub fn get_super_admin(env:Env) -> Address {
        let user_registry_addr = Self::get_user_registry(env.clone());
        let client = user_registry::Client::new(&env, &user_registry_addr);
        client.get_super_admin()
    }

    pub fn has_user_registry(env:Env) -> bool {
        let key = DataKeys::UserRegistry;
        env.storage().instance().has(&key)
    }
}

// soroban contract deploy \
//   --wasm target/wasm32-unknown-unknown/release/user_registry.wasm \
//   --source alice \
//   --network testnet

//   soroban contract invoke \
//   --id CDEXXZ7GUOFKYOA7VKA7S4LO23EHIBRSDCRAAJCDCOPLL7CVTDS5SI7E \
//   --source alice \
//   --network testnet \
//   -- \
//   issue_new_token \
//   --items '["food"]' \
//   --merchants '["GB5JVYGERUEATTCBA4PTOC7CKINQPBBROFU75SZ5CFG6UDC5Y3MKNP2D"]'


// soroban contract invoke \
//  --id CC5OZSMIKXJHBUI2PDM55N7TLCJRHDNXMS27GYZLAW7UKB5QRXYK6DU5 \
//  --source bob \
//  --network testnet \
//  -- \
//  create_campaign \
//  --name "Donate Medicine" \
//  --description "I want to donate medicine" \
//  --no_of_recipients 1 \
//  --token_address CB7LGIQUPE26SXIE4XNAWLBTYB46VXQBUN4MG6LH24NED3DCPL3XGA6E \
//  --amount 10 \
//  --creator bob

//   localcoin addr - CA6NUSK2W5GR5PAGGLOUUYZ7E67DQJTZFRDZRXY4TS3AF3ZZYTUMTN3T
  
//   soroban contract invoke \
//   --id CDZMT6TRDSE4WS3DLJSUEKEUAPXHBXDTGAUW4AICR2LG2QW5O4ASZMYV \
//   --source jack \
//   --network testnet \
//   -- \
//   merchant_registration \
//   --merchant_wallet carol \
//   --proprietor "carol" \
//   --phone_no "+977-94488888888" \
//   --store_name "cbs4atore" \
//   --location "ccc"
  
//   soroban contract invoke \
//   --id CDZMT6TRDSE4WS3DLJSUEKEUAPXHBXDTGAUW4AICR2LG2QW5O4ASZMYV \
//   --source alice \
//   --network testnet \
//   -- \
//   set_campaign_admin \
//   --campaign_management bob \
//   --campaign alice\
//   --admin bob