#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, String, Vec};

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

#[derive(Clone)]
#[contracttype]
pub enum DataKeys{
    UserRegistry,
    ItemsAssociated(Address),
    MerchantsAssociated(Address)
}

#[contract]
pub struct IssuanceManagement;

#[contractimpl]
impl IssuanceManagement {
    // initialize contract
    pub fn initialize(env:Env, address:Address) {
        if Self::has_user_registry(env.clone()) {
            panic!("Contract already initialized.")
        }
        let key = DataKeys::UserRegistry;
        env.storage().instance().set(&key, &address)
    }

    pub fn set_user_registry(env:Env, address:Address) {
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();
        
        let key = DataKeys::UserRegistry;
        env.storage().instance().set(&key, &address)
    }

    pub fn issue_new_token(env:Env, items:Vec<String>, merchants:Vec<Address>) {
        let super_admin = Self::get_super_admin(env.clone());
        super_admin.require_auth();

        let wasm_hash = env.deployer().upload_contract_wasm(localcoin::WASM);
        let salt = BytesN::from_array(&env, &[0; 32]);
        let deployed_token = env.deployer().with_address(super_admin, salt).deploy(wasm_hash);

        let item_key = DataKeys::ItemsAssociated(deployed_token.clone());
        env.storage().instance().set(&item_key, &items);

        let merchant_key = DataKeys::MerchantsAssociated(deployed_token.clone());
        env.storage().instance().set(&merchant_key, &merchants)
    }

    pub fn get_user_registry(env:Env) -> Address {
        let key = DataKeys::UserRegistry;
        if let Some(user_registry_addr) = env.storage().instance().get::<DataKeys, Address>(&key) {
            user_registry_addr
        } else {
            panic!("Address not set.")
        }
    }

    pub fn get_super_admin(env:Env) -> Address {
        let user_registry_addr = Self::get_user_registry(env.clone());
        let client = user_registry::Client::new(&env, &user_registry_addr);
        client.get_super_admin()
    }

    pub fn has_user_registry(e:Env) -> bool {
        let key = DataKeys::UserRegistry;
        e.storage().instance().has(&key)
    }
}