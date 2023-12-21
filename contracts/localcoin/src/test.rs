#![cfg(test)]
extern crate std;

use crate::{contract::LocalCoin, LocalCoinClient};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal
};

fn create_token<'a>(e: &Env, admin: &Address) -> LocalCoinClient<'a> {
    let token = LocalCoinClient::new(e, &e.register_contract(None, LocalCoin {}));
    token.initialize(admin, &7, &"name".into_val(e), &"symbol".into_val(e));
    token
}

#[test]
fn test() {
    let e = Env::default();
    e.mock_all_auths();

    let admin1 = Address::generate(&e);
    let admin2 = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let token = create_token(&e, &admin1);

    token.mint(&user1, &1000);
    assert_eq!(
        e.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&user1, 1000_i128).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user1), 1000);
    assert_eq!(token.total_supply(), 1000);
    assert_eq!(token.total_burned(), 0);

    token.transfer(&user1, &user2, &600);
    assert_eq!(
        e.auths(),
        std::vec![(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("transfer"),
                    (&user1, &user2, 600_i128).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user1), 400);
    assert_eq!(token.balance(&user2), 600);

    token.set_admin(&admin2);
    assert_eq!(
        e.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("set_admin"),
                    (&admin2,).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

#[test]
fn test_burn() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let user1 = Address::generate(&e);
    let token = create_token(&e, &admin);

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);
    assert_eq!(token.total_supply(), 1000);
    assert_eq!(token.total_burned(), 0);

    token.burn(&user1, &1000);
    assert_eq!(
        e.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("burn"),
                    (&user1, 1000_i128).into_val(&e),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.total_supply(), 0);
    assert_eq!(token.total_burned(), 1000);
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn transfer_insufficient_balance() {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let user1 = Address::generate(&e);
    let user2 = Address::generate(&e);
    let token = create_token(&e, &admin);

    token.mint(&user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.transfer(&user1, &user2, &1001);
}

#[test]
#[should_panic(expected = "already initialized")]
fn initialize_already_initialized() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let token = create_token(&e, &admin);

    token.initialize(&admin, &10, &"name".into_val(&e), &"symbol".into_val(&e));
}

#[test]
fn test_set_issuance_management() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let issuance_management = Address::generate(&e);
    let token = create_token(&e, &admin);

    token.set_issuance_management(&issuance_management);
    assert_eq!(token.get_issuance_management(), issuance_management);

}

#[test]
#[should_panic(expected = "Address already set.")]
fn test_set_already_set_issuance_management() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let issuance_management = Address::generate(&e);
    let token = create_token(&e, &admin);

    token.set_issuance_management(&issuance_management);
    token.set_issuance_management(&issuance_management);

}

#[test]
#[should_panic(expected = "Decimal must fit in a u8")]
fn decimal_is_over_max() {
    let e = Env::default();
    let admin = Address::generate(&e);
    let token = LocalCoinClient::new(&e, &e.register_contract(None, LocalCoin {}));
    token.initialize(
        &admin,
        &(u32::from(u8::MAX) + 1),
        &"name".into_val(&e),
        &"symbol".into_val(&e),
    );
}