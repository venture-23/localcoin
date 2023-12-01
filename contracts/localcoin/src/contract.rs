
use crate::admin::{has_administrator, read_administrator, write_administrator, has_issuance, read_issuance, write_issuance};
use crate::balance::{read_balance, receive_balance, spend_balance, read_total_supply, read_token_burnt, update_mint_supply, update_burn_supply};
use crate::metadata::{read_decimal, read_name, read_symbol, write_metadata};
use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use soroban_sdk::{contract, contractimpl, Address, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;

mod issuance {
    soroban_sdk::contractimport!(
        file =
            "../issuance_management/target/wasm32-unknown-unknown/release/issuance_management.wasm"
    );
}

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

#[contract]
pub struct LocalCoin;

#[contractimpl]
impl LocalCoin {
    pub fn initialize(e: Env, admin: Address, decimal: u32, name: String, symbol: String) {
        if has_administrator(&e) {
            panic!("already initialized")
        }
        write_administrator(&e, &admin);
        if decimal > u8::MAX.into() {
            panic!("Decimal must fit in a u8");
        }

        write_metadata(
            &e,
            TokenMetadata {
                decimal,
                name,
                symbol,
            },
        )
    }

    pub fn mint(e: Env, to: Address, amount: i128) {
        check_nonnegative_amount(amount);
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        receive_balance(&e, to.clone(), amount);
        update_mint_supply(&e, amount);
        TokenUtils::new(&e).events().mint(admin, to, amount);
    }

    pub fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_administrator(&e, &new_admin);
        TokenUtils::new(&e).events().set_admin(admin, new_admin);
    }

    pub fn set_issuance_management(e: Env, issuance_addr: Address) {
        if has_issuance(&e) {
            panic!("Address already set.")
        }
        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        
        write_issuance(&e, &issuance_addr);
    }

    pub fn balance_of(e: Env, id: Address) -> i128 {
        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_balance(&e, id)
    }

    pub fn total_supply(e: Env) ->i128 {
        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_total_supply(&e)
    }

    pub fn total_burned(e: Env) ->i128 {
        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_token_burnt(&e)
    }

    pub fn recipient_to_merchant_transfer(e: Env, from: Address, to: Address, amount: i128) {
        // this function is called by recepient while transfering tokens to merchants
        // this checks if the token recipient is sending has the merchant(receiver) associated with that token.
        
        let current_contract = e.current_contract_address();
        let issuance_addr = Self::get_issuance_management(e.clone());
        let issuance_client = issuance::Client::new(&e, &issuance_addr);
        
        let merchants_associated = issuance_client.get_merchants_assocoated(&current_contract);
        if !merchants_associated.contains(to.clone()) {
            panic!("This token's item is not accepted by the merchant.")
        }
        Self::transfer(e, from, to, amount);
    }

    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount);
    }

    pub fn burn(e: Env, from: Address, amount: i128) {
        let admin = read_administrator(&e);
        admin.require_auth();

        check_nonnegative_amount(amount);

        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&e, from.clone(), amount);
        update_burn_supply(&e, amount);
        TokenUtils::new(&e).events().burn(from, amount);
    }

    pub fn get_issuance_management(e: Env) -> Address {
        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_issuance(&e)
    }

    pub fn get_admin(e: Env) -> Address {
        e.storage()
            .instance()
            .bump(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_administrator(&e)
    }

    pub fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    pub fn name(e: Env) -> String {
        read_name(&e)
    }

    pub fn symbol(e: Env) -> String {
        read_symbol(&e)
    }
}