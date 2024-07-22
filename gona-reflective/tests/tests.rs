//! Tests for the `gona_reflective` contract.
use gona_reflective::*;
use concordium_cis2::*;
use concordium_smart_contract_testing::*;
use primitive_types::*;

/// The tests accounts.
const ALICE: AccountAddress = AccountAddress([3; 32]);
const ALICE_ADDR: Address = Address::Account(ALICE);
const BOB: AccountAddress = AccountAddress([1; 32]);
const BOB_ADDR: Address = Address::Account(BOB);
//const CHARLIE: AccountAddress = Address::Account(AccountAddress([2; 32]));

/// Initial balance of the accounts.
const ACC_INITIAL_BALANCE: Amount = Amount::from_ccd(10000);

/// A signer for all the transactions.
const SIGNER: Signer = Signer::with_one_key();

/// The metadata url for testing.
const METADATA_URL: &str = "https://example.com";

/// Test that init produces the correct logs.
#[test]
fn test_init() {
    let (_chain, init) = initialize_chain_and_contract();
    // Check that the logs are correct.

    let events = init
        .events
        .iter()
        .map(|e| e.parse().expect("Parsing GonaEvent."))
        .collect::<Vec<GonaEvent>>();

    assert_eq!(events, [
        GonaEvent::Cis2Event(Cis2Event::Mint(MintEvent {
            token_id: TOKEN_ID_Gona,
            amount:   ContractTokenAmount::from(U256::zero()),
            owner:    ALICE_ADDR,
        })),
        GonaEvent::Cis2Event(Cis2Event::TokenMetadata(TokenMetadataEvent {
            token_id:     TOKEN_ID_Gona,
            metadata_url: MetadataUrl {
                url:  METADATA_URL.to_string(),
                hash: None,
            },
        })),
        GonaEvent::NewAdmin {
            new_admin: NewAdminEvent {
                new_admin: ALICE_ADDR,
            },
        }
    ]);
}

/// Test that only the admin can set the metadata URL.
#[test]
fn test_set_metadata_url() {
    let (mut chain, contract_address, _) = initialize_contract_with_alice_tokens();

    let new_metadata_url = "https://new-url.com".to_string();

    // Construct the parameters.
    let params = SetMetadataUrlParams {
        url:  new_metadata_url.clone(),
        hash: None,
    };

    // Try to set the metadata URL from Bob's account, who is not the admin.
    let update = chain
        .contract_update(SIGNER, BOB, BOB_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.setMetadataUrl".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&params).expect("SetMetadataUrl params"),
        })
        .expect_err("SetMetadataUrl");

    // Check that the return value is correct.
    let rv: ContractError = update.parse_return_value().expect("Parsing ContractError");
    assert_eq!(rv, ContractError::Unauthorized);

    // Set the metadata URL from Alice's account, who is the admin.
    let update = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.setMetadataUrl".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&params).expect("SetMetadataUrl params"),
        })
        .expect("SetMetadataUrl");

    // Check that the logs are correct.
    let events = deserialize_update_events(&update);

    assert_eq!(events, [GonaEvent::Cis2Event(Cis2Event::TokenMetadata(TokenMetadataEvent {
        token_id:     TOKEN_ID_Gona,
        metadata_url: MetadataUrl {
            url:  new_metadata_url,
            hash: None,
        },
    })),]);
}

/// Test regular transfer where sender is the owner.
#[test]
fn test_account_transfer() {
    let (mut chain, contract_address, _update) = initialize_contract_with_alice_tokens();

    // Transfer one token from Alice to Bob.
    let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
        from:     ALICE_ADDR,
        to:       Receiver::Account(BOB),
        token_id: TOKEN_ID_Gona,
        amount:   TokenAmountU256(U256::from(1000_u64)),
        data:     AdditionalData::empty(),
    }]);

    let update = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.transfer".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
        })
        .expect("Transfer tokens");

    // Check that Alice now has 99 wCCD and Bob has 1.
    let balances = get_balances(&chain, contract_address);
    assert_eq!(balances.0, [TokenAmountU256(U256::from(900)), TokenAmountU256(U256::from(100))]);

    // Check that a single transfer event occurred.
    let events = deserialize_update_events(&update);
    assert_eq!(events, [GonaEvent::Cis2Event(Cis2Event::Transfer(TransferEvent {
        token_id: TOKEN_ID_Gona,
        amount:   TokenAmountU256(U256::from(1)),
        from:     ALICE_ADDR,
        to:       BOB_ADDR,
    })),]);
}


/// Test regular transfer where sender is the owner.
#[test]
fn test_alice_balance() {
    let (chain, contract_address, _update) = initialize_contract_with_alice_tokens();
    // Check that Alice now has 99 wCCD and Bob has 1.
    let balances = get_balances(&chain, contract_address);
    assert_eq!(balances.0, [TokenAmountU256(U256::from(50000000000000000_u64)), TokenAmountU256(U256::from(0))]);

}



/// Test that you can add an operator.
/// Initialize the contract with one token owned by Alice.
/// Then add Bob as an operator for Alice.
#[test]
fn test_add_operator() {
    let (mut chain, contract_address, _update) = initialize_contract_with_alice_tokens();

    // Add Bob as an operator for Alice.
    let params = UpdateOperatorParams(vec![UpdateOperator {
        update:   OperatorUpdate::Add,
        operator: BOB_ADDR,
    }]);

    let update = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.updateOperator".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&params).expect("UpdateOperator params"),
        })
        .expect("Update operator");

    // Check that an operator event occurred.
    let events = deserialize_update_events(&update);
    assert_eq!(events, [GonaEvent::Cis2Event(Cis2Event::UpdateOperator(UpdateOperatorEvent {
        operator: BOB_ADDR,
        owner:    ALICE_ADDR,
        update:   OperatorUpdate::Add,
    })),]);

    // Construct a query parameter to check whether Bob is an operator for Alice.
    let query_params = OperatorOfQueryParams {
        queries: vec![OperatorOfQuery {
            owner:   ALICE_ADDR,
            address: BOB_ADDR,
        }],
    };

    // Invoke the operatorOf view entrypoint and check that Bob is an operator for
    // Alice.
    let invoke = chain
        .contract_invoke(ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.operatorOf".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&query_params).expect("OperatorOf params"),
        })
        .expect("Invoke view");

    let rv: OperatorOfQueryResponse = invoke.parse_return_value().expect("OperatorOf return value");
    assert_eq!(rv, OperatorOfQueryResponse(vec![true]));
}

/// Test that a transfer fails when the sender is neither an operator or the
/// owner. In particular, Bob will attempt to transfer some of Alice's tokens to
/// himself.
#[test]
fn test_unauthorized_sender() {
    let (mut chain, contract_address, _update) = initialize_contract_with_alice_tokens();

    // Construct a transfer of `TOKEN_ID_Gona` from Alice to Bob, which will be
    // submitted by Bob.
    let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
        from:     ALICE_ADDR,
        to:       Receiver::Account(BOB),
        token_id: TOKEN_ID_Gona,
        amount:   TokenAmountU64(1),
        data:     AdditionalData::empty(),
    }]);

    // Notice that Bob is the sender/invoker.
    let update = chain
        .contract_update(SIGNER, BOB, BOB_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.transfer".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
        })
        .expect_err("Transfer tokens");

    // Check that the correct error is returned.
    let rv: ContractError = update.parse_return_value().expect("ContractError return value");
    assert_eq!(rv, ContractError::Unauthorized);
}

/// Test that an operator can make a transfer.
#[test]
fn test_operator_can_transfer() {
    let (mut chain, contract_address, _update) = initialize_contract_with_alice_tokens();

    // Add Bob as an operator for Alice.
    let params = UpdateOperatorParams(vec![UpdateOperator {
        update:   OperatorUpdate::Add,
        operator: BOB_ADDR,
    }]);
    chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.updateOperator".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&params).expect("UpdateOperator params"),
        })
        .expect("Update operator");

    // Let Bob make a transfer to himself on behalf of Alice.
    let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
        from:     ALICE_ADDR,
        to:       Receiver::Account(BOB),
        token_id: TOKEN_ID_Gona,
        amount:   TokenAmountU256(U256::from(1000000)),
        data:     AdditionalData::empty(),
    }]);

    chain
        .contract_update(SIGNER, BOB, BOB_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.transfer".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
        })
        .expect("Transfer tokens");

    // Check that Bob now has 1 wCCD and Alice has 99.
    let balances = get_balances(&chain, contract_address);
    assert_eq!(balances.0, [TokenAmountU256(U256::from(99)), TokenAmountU256(U256::from(1))]);
}


/// Test that you can update the admin account.
#[test]
fn test_update_admin() {
    let (mut chain, contract_address, _update) = initialize_contract_with_alice_tokens();

    // Update the admin account to Bob.
    let params = BOB_ADDR;

    let update = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.updateAdmin".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&params).expect("UpdateAdmin params"),
        })
        .expect("Update admin");

    // Invoke the view function to check that the new admin is Bob.
    assert_eq!(invoke_view(&mut chain, contract_address).admin, BOB_ADDR);

    // Check that the logs are correct.
    let events = deserialize_update_events(&update);
    assert_eq!(events, [GonaEvent::NewAdmin {
        new_admin: NewAdminEvent {
            new_admin: BOB_ADDR,
        },
    },]);
}

/// Test that only the current admin can update the admin account.
#[test]
fn test_update_admin_unauthorized() {
    let (mut chain, contract_address, _update) = initialize_contract_with_alice_tokens();

    // Update the admin account to Bob.
    let params = BOB_ADDR;

    let update = chain
        .contract_update(
            SIGNER,
            BOB,
            BOB_ADDR, // Bob is not the admin.
            Energy::from(10000),
            UpdateContractPayload {
                amount:       Amount::zero(),
                receive_name: OwnedReceiveName::new_unchecked("gona_reflective.updateAdmin".to_string()),
                address:      contract_address,
                message:      OwnedParameter::from_serial(&params).expect("UpdateAdmin params"),
            },
        )
        .expect_err("Update admin");

    // Check that the correct error is returned.
    let rv: ContractError = update.parse_return_value().expect("ContractError return value");
    assert_eq!(rv, ContractError::Unauthorized);
}

/// Test that the pause/unpause entrypoints correctly sets the pause value in
/// the state.
#[test]
fn test_pause_functionality() {
    let (mut chain, contract_address, _update) = initialize_contract_with_alice_tokens();

    // Pause the contract.
    chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.setPaused".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&true).expect("Pause params"),
        })
        .expect("Pause");

    // Check that the contract is now paused.
    assert_eq!(invoke_view(&mut chain, contract_address).paused, true);

    // Unpause the contract.
    chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.setPaused".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&false).expect("Unpause params"),
        })
        .expect("Unpause");
    // Check that the contract is now unpaused.
    assert_eq!(invoke_view(&mut chain, contract_address).paused, false);
}

/// Test that only the admin can pause/unpause the contract.
#[test]
fn test_pause_unpause_unauthorized() {
    let (mut chain, contract_address, _update) = initialize_contract_with_alice_tokens();

    // Pause the contract as Bob, who is not the admin.
    let update = chain
        .contract_update(SIGNER, BOB, BOB_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.setPaused".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&true).expect("Pause params"),
        })
        .expect_err("Pause");

    // Check that the correct error is returned.
    let rv: ContractError = update.parse_return_value().expect("ContractError return value");
    assert_eq!(rv, ContractError::Unauthorized);
}

/// Test that one can NOT call non-admin state-mutative functions (wrap,
/// unwrap, transfer, updateOperator) when the contract is paused.
#[test]
fn test_no_execution_of_state_mutative_functions_when_paused() {
    let (mut chain, contract_address, _update) = initialize_contract_with_alice_tokens();

    // Pause the contract.
    chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.setPaused".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&true).expect("Pause params"),
        })
        .expect("Pause");

    // Try to wrap 100 microCCD into wCCD for Bob.
    let mint_params = MintParams {
        to:   Address::Account(BOB),
        };
    let update_wrap = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.mint".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&mint_params).expect("Wrap params"),
        })
        .expect_err("Wrap CCD");

    // Check that the correct error is returned.
    assert_contract_paused_error(&update_wrap);

    // // Try to unwrap 1 wCCD for Alice.
    // let unmint_params = UnWrapParams {
    //     amount:   TokenAmountU256(U256::from(1)),
    //     owner:    ALICE_ADDR,
    //     receiver: Receiver::Account(ALICE),
    //     data:     AdditionalData::empty(),
    // };
    // let update_unwrap = chain
    //     .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
    //         amount:       Amount::zero(),
    //         receive_name: OwnedReceiveName::new_unchecked("gona_reflective.unwrap".to_string()),
    //         address:      contract_address,
    //         message:      OwnedParameter::from_serial(&unmint_params).expect("Unwrap params"),
    //     })
    //     .expect_err("Unwrap wCCD");
    // assert_contract_paused_error(&update_unwrap);

    // Try to transfer 1 wCCD from Alice to Bob.
    let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
        from:     ALICE_ADDR,
        to:       Receiver::Account(BOB),
        token_id: TOKEN_ID_Gona,
        amount:   TokenAmountU64(1),
        data:     AdditionalData::empty(),
    }]);
    let update_transfer = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.transfer".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
        })
        .expect_err("Transfer tokens");
    assert_contract_paused_error(&update_transfer);

    // Try to add Bob as an operator for Alice.
    let params = UpdateOperatorParams(vec![UpdateOperator {
        update:   OperatorUpdate::Add,
        operator: BOB_ADDR,
    }]);
    let update_operator = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.updateOperator".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&params).expect("UpdateOperator params"),
        })
        .expect_err("Update operator");
    assert_contract_paused_error(&update_operator);
}

// Helpers:

/// Helper function that initializes the contract and wraps 100 microCCD into
/// wCCD for Alice.
fn initialize_contract_with_alice_tokens() -> (Chain, ContractAddress, ContractInvokeSuccess) {
    let (mut chain, init) = initialize_chain_and_contract();

    let mint_params = MintParams {
        to:   Address::Account(ALICE),
          };

    // Wrap 100 CCD into wCCD for Alice.
    let update = chain
        .contract_update(SIGNER, ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.mint".to_string()),
            address:      init.contract_address,
            message:      OwnedParameter::from_serial(&mint_params).expect("mint params"),
        })
        .expect("MINT Gona");
    (chain, init.contract_address, update)
}

/// Setup chain and contract.
///
/// Also creates the two accounts, Alice and Bob.
///
/// Alice is the owner of the contract.
fn initialize_chain_and_contract() -> (Chain, ContractInitSuccess) {
    let mut chain = Chain::new();

    // Create some accounts on the chain.
    chain.create_account(Account::new(ALICE, ACC_INITIAL_BALANCE));
    chain.create_account(Account::new(BOB, ACC_INITIAL_BALANCE));

    // Load and deploy the module.
    let module = module_load_v1("dist/module.wasm.v1").expect("Module exists");
    let deployment = chain.module_deploy_v1(SIGNER, ALICE, module).expect("Deploy valid module");

    // Construct the initial parameters.
    let params = SetMetadataUrlParams {
        url:  METADATA_URL.to_string(),
        hash: None,
    };

    // Initialize the auction contract.
    let init = chain
        .contract_init(SIGNER, ALICE, Energy::from(10000), InitContractPayload {
            amount:    Amount::zero(),
            mod_ref:   deployment.module_reference,
            init_name: OwnedContractName::new_unchecked("init_gona_reflective".to_string()),
            param:     OwnedParameter::from_serial(&params).expect("Init params"),
        })
        .expect("Initialize contract");

    (chain, init)
}

/// Get the result of the view entrypoint.
fn invoke_view(chain: &mut Chain, contract_address: ContractAddress) -> ReturnBasicState {
    let invoke = chain
        .contract_invoke(ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.view".to_string()),
            address:      contract_address,
            message:      OwnedParameter::empty(),
        })
        .expect("Invoke view");
    invoke.parse_return_value().expect("Return value")
}

/// Get the balances for Alice and Bob.
fn get_balances(
    chain: &Chain,
    contract_address: ContractAddress,
) -> ContractBalanceOfQueryResponse {
    let balance_of_params = ContractBalanceOfQueryParams {
        queries: vec![
            BalanceOfQuery {
                token_id: TOKEN_ID_Gona,
                address:  ALICE_ADDR,
            },
            BalanceOfQuery {
                token_id: TOKEN_ID_Gona,
                address:  BOB_ADDR,
            },
        ],
    };
    let invoke = chain
        .contract_invoke(ALICE, ALICE_ADDR, Energy::from(10000), UpdateContractPayload {
            amount:       Amount::zero(),
            receive_name: OwnedReceiveName::new_unchecked("gona_reflective.balanceOf".to_string()),
            address:      contract_address,
            message:      OwnedParameter::from_serial(&balance_of_params)
                .expect("BalanceOf params"),
        })
        .expect("Invoke balanceOf");
    let rv: ContractBalanceOfQueryResponse =
        invoke.parse_return_value().expect("BalanceOf return value");
    rv
}

/// Deserialize the events from an update.
fn deserialize_update_events(update: &ContractInvokeSuccess) -> Vec<GonaEvent> {
    update
        .events()
        .flat_map(|(_addr, events)| events.iter().map(|e| e.parse().expect("Deserialize event")))
        .collect()
}

/// Check that the returned error is `ContractPaused`.
fn assert_contract_paused_error(update: &ContractInvokeError) {
    let rv: ContractError = update.parse_return_value().expect("ContractError return value");
    assert_eq!(rv, ContractError::Custom(CustomContractError::ContractPaused));
}