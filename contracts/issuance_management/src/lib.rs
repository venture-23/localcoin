#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, String, Vec, Map, vec};
mod test;

mod localcoin {
    soroban_sdk::contractimport!(
        file =
            "../localcoin/target/wasm32-unknown-unknown/release/localcoin.wasm"
    );
}

mod registry {
    soroban_sdk::contractimport!(
        file =
            "../registry/target/wasm32-unknown-unknown/release/registry.wasm"
    );
}

#[derive(Clone)]
#[contracttype]
pub enum DataKeys{
    Registry,
    SaltCounter,
    CampaignManagement,
    ItemsAssociated(Address),
    MerchantsAssociated(Address),
    TokenNameAndAddress
}

#[contract]
pub struct IssuanceManagement;

#[contractimpl]
impl IssuanceManagement {
    // initialize contract
    pub fn initialize(env:Env, registry:Address) {
        if Self::has_registry(env.clone()) {
            panic!("Contract already initialized.")
        }
        let key = DataKeys::Registry;
        env.storage().instance().set(&key, &registry)
    }

    pub fn set_registry(env:Env, address:Address) {
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();
        
        let key = DataKeys::Registry;
        env.storage().instance().set(&key, &address)
    }

    pub fn set_campaign_management(env:Env, address:Address) {
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();
        
        let key = DataKeys::CampaignManagement;
        env.storage().instance().set(&key, &address)
    }

    pub fn issue_new_token(env:Env, decimal:u32, name:String, symbol:String, 
        items:Vec<String>, merchants:Vec<Address>) 
        {
            let super_admin = Self::get_super_admin(env.clone());
            super_admin.require_auth();

            let mut existing_token_name_addr = Self::get_token_name_address(env.clone());
            if existing_token_name_addr.contains_key(symbol.clone()) {
                panic!("Token with given symbol already exists.")
            }

            let campaign_management_addr = Self::get_campaign_management(env.clone());
            let registry_addr = Self::get_registry(env.clone());
            let registry_client = registry::Client::new(&env, &registry_addr);

            let verified_merchants = registry_client.get_verified_merchants();
            for merchant in merchants.iter() {
                if !(verified_merchants.contains(merchant)) {
                    panic!("Merchants list contains unverified merchant.")
                }
            }

            let wasm_hash = env.deployer().upload_contract_wasm(localcoin::WASM);
            // dynamic salt count to deploy multiple token contracts
            let salt_key = DataKeys::SaltCounter;
            let salt_count: u32 = env.storage().instance().get::<DataKeys, u32>(&salt_key).unwrap_or(0);
            let salt = BytesN::from_array(&env, &[salt_count.try_into().unwrap(); 32]);

            let deployed_token = env.deployer().with_address(super_admin, salt).deploy(wasm_hash);
            
            let token_client = localcoin::Client::new(&env, &deployed_token);
            let current_contract = env.current_contract_address();

            // set issuance management contract in deployed token 
            token_client.set_issuance_management(&current_contract);
            // initialize deployed token
            token_client.initialize(&campaign_management_addr, &decimal, &name, &symbol);

            // increase salt counter 
            env.storage().instance().set(&salt_key, &(salt_count + 1));

            // store items associated with token
            let item_key = DataKeys::ItemsAssociated(deployed_token.clone());
            env.storage().instance().set(&item_key, &items);
            
            // store merchants associated with token
            let merchant_key = DataKeys::MerchantsAssociated(deployed_token.clone());
            env.storage().instance().set(&merchant_key, &merchants);

            // store token address of the token symbol
            let token_addr_key = DataKeys::TokenNameAndAddress;
            existing_token_name_addr.set(symbol.clone(), deployed_token.clone());
            env.storage().instance().set(&token_addr_key, &existing_token_name_addr);

            //  send deployed tokens to registry contract
            registry_client.add_deployed_tokens(&deployed_token);

            // emit event
            env.events().publish((deployed_token, (name, symbol, decimal), (items, merchants)), "New token issued.");
    }

    // adds new items for an existing token
    pub fn add_token_items(env:Env, token_address:Address, items:Vec<String>) {
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();

        let key = DataKeys::ItemsAssociated(token_address.clone());
        if !(env.storage().instance().has(&key)) {
            panic!("Token doesn't exist.")
        }

        let existig_items = Self::get_items_associated(env.clone(), token_address.clone());
        for item in items.iter() {
            if existig_items.contains(item) {
                panic!("Item provided already exist.")
            }
        }
        let updated_items_list = vec![&env, existig_items, items.clone()].concat();
        env.storage().instance().set(&key, &updated_items_list);
        // emit event
        env.events().publish((token_address, items), "Token's items list updated.");

    }

    // adds new merchants for an existing token
    pub fn add_token_merchants(env:Env, token_address:Address, merchants:Vec<Address>) {
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();

        let key = DataKeys::MerchantsAssociated(token_address.clone());
        if !(env.storage().instance().has(&key)) {
            panic!("Token doesn't exist.")
        }

        let registry_addr = Self::get_registry(env.clone());
        let registry_client = registry::Client::new(&env, &registry_addr);
        let verified_merchants = registry_client.get_verified_merchants();
        for merchant in merchants.iter() {
            if !(verified_merchants.contains(merchant)) {
                panic!("Merchants list contains unverified merchant.")
            }
        }
        let existig_merchants = Self::get_merchants_associated(env.clone(), token_address.clone());
        for merchant in merchants.iter() {
            if existig_merchants.contains(merchant) {
                panic!("Merchant provided already exist.")
            }
        }
        let updated_merchants_list = vec![&env, existig_merchants, merchants.clone()].concat();
        env.storage().instance().set(&key, &updated_merchants_list);
        // emit event
        env.events().publish((token_address, merchants), "Token's merchants list updated.");
    }

    pub fn get_balance_of_batch(env:Env, user:Address) -> Map<String, (i128, Address)> {
        let registry_addr = Self::get_registry(env.clone());
        let registry_client = registry::Client::new(&env, &registry_addr);
        let tokens =  registry_client.get_available_tokens();
        
        let mut tokens_balance: Map<String, (i128, Address)> = Map::new(&env);
        for token in tokens.iter() {
            let token_client = localcoin::Client::new(&env, &token);
            let balance = token_client.balance(&user);
            let name = token_client.name();
            if balance > 0 {
                tokens_balance.set(name, (balance, token));
            }
        }
        tokens_balance
    }

    pub fn get_registry(env:Env) -> Address {
        let key = DataKeys::Registry;
        if let Some(registry_addr) = env.storage().instance().get::<DataKeys, Address>(&key) {
            registry_addr
        } else {
            panic!("Address not set.")
        }
    }

    pub fn get_campaign_management(env:Env) -> Address {
        let key = DataKeys::CampaignManagement;
        if let Some(campaign_management_addr) = env.storage().instance().get::<DataKeys, Address>(&key) {
            campaign_management_addr
        } else {
            panic!("Address not set.")
        }
    }

    pub fn get_super_admin(env:Env) -> Address {
        let registry_addr = Self::get_registry(env.clone());
        let registry_client = registry::Client::new(&env, &registry_addr);
        registry_client.get_super_admin()
    }

    pub fn get_merchants_associated(env:Env, token_address:Address) -> Vec<Address> {
        let key = DataKeys::MerchantsAssociated(token_address);
        if let Some(merchants) = env.storage().instance().get::<DataKeys, Vec<Address>>(&key) {
            merchants
        } else {
            vec![&env]
        }
    }

    pub fn get_items_associated(env:Env, token_address:Address) -> Vec<String> {
        let key = DataKeys::ItemsAssociated(token_address);
        if let Some(items) = env.storage().instance().get::<DataKeys, Vec<String>>(&key) {
            items
        } else {
            vec![&env]
        }
    }

    pub fn get_token_name_address(env:Env) -> Map<String, Address> {
        let key = DataKeys::TokenNameAndAddress;
        if let Some(token_name_addr) = env.storage().instance().get::<DataKeys, Map<String, Address>>(&key) {
            token_name_addr
        } else {
            Map::new(&env)
        }
    }

    pub fn has_registry(e:Env) -> bool {
        let key = DataKeys::Registry;
        e.storage().instance().has(&key)
    }
}