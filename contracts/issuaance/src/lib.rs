#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec, Val, Symbol};

mod contract {
    soroban_sdk::contractimport!(
        file =
            "../campaign/target/wasm32-unknown-unknown/release/campaign.wasm"
    );
}

// #[derive(Clone)]
// #[contracttype]
// pub enum Val {
//     Name(String),
//     Description(String),
//     NoOfRecipient(u32),
//     Creator(Address)
// }

#[contract]
pub struct Issuance;

#[contractimpl]
impl Issuance {

    pub fn create_campaign(env:Env, name:String, description:String, no_of_recipient:Val,
         creator: Address, function: Symbol) {
            
            let deployer = env.current_contract_address();

            if deployer != env.current_contract_address() {
                deployer.require_auth();
            }
    
            let wasm_hash = env.deployer().upload_contract_wasm(contract::WASM);
            let salt = BytesN::from_array(&env, &[0; 32]);
            let mut init_fn_args: Vec<Val> = Vec::new(&env);
            init_fn_args.push_back(name.to_val());
            init_fn_args.push_back(description.to_val());
            init_fn_args.push_back(no_of_recipient);
            init_fn_args.push_back(creator.to_val());

            let deployed_address = env.deployer().with_address(deployer, salt).deploy(wasm_hash);

            env.invoke_contract(&deployed_address, &function, init_fn_args)
            

    }
}