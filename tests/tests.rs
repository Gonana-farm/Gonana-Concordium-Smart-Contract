use concordium_smart_contract_testing::*;
use gonana_concordium_smart_contract::*;

/// A test account.
const ALICE: AccountAddress = AccountAddress([0u8; 32]);
const ALICE_ADDR: Address = Address::Account(ALICE);

/// The initial balance of the ALICE test account.
const ACC_INITIAL_BALANCE: Amount = Amount::from_ccd(10_000);

/// A [`Signer`] with one set of keys, used for signing transactions.
const SIGNER: Signer = Signer::with_one_key();


fn setup_chain_and_contract() -> (Chain, ContractInitSuccess) {
    let mut chain = Chain::new();

    chain.create_account(Account::new(ALICE, ACC_INITIAL_BALANCE));

    let module = module_load_v1("gonana_marketplace.wasm.v1").expect("Module is valid and exists");
    let deployment = chain.module_deploy_v1(SIGNER,ALICE , module).expect("Deploying valid module should succeed");

    let initialization = chain
    .contract_init(
        SIGNER,
        ALICE,
        Energy::from(10000),
        InitContractPayload {
            mod_ref: deployment.module_reference,
            init_name: OwnedContractName::new_unchecked("init_gonana_marketplace".to_string()),
            param: OwnedParameter::empty(),
            amount: Amount::zero(),
        }
    )
    .expect("Initialization should always succeed"); 

    (chain, initialization)
}
/// Test listing a product in the marketplace.
#[test]
fn test_list_product() {
    let (mut chain, init) = setup_chain_and_contract();

    // List a product in the marketplace.
    chain
        .contract_update(
            SIGNER,
            ALICE,
            ALICE_ADDR,
            Energy::from(10_000),
            UpdateContractPayload {
                address:      init.contract_address,
                amount:       Amount::zero(),
                receive_name: OwnedReceiveName::new_unchecked("gonana_marketplace.list_product".to_string()),
                message:      OwnedParameter::from_serial(&ListProductParameter {
                    farmer: ALICE,
                    product: "Apples".to_string(),
                    price: Amount::from_ccd(100),
                })
                .expect("Parameter within size bounds"),
            },
        )
        .expect("List product succeeds.");

    // Verify that the product is listed.
    let product_listings: Vec<ProductListing> =
        chain.contract_update(SIGNER, ALICE,  Address::Account(ALICE),  Energy::from(10000), UpdateContractPayload {
            amount: Amount::zero(),
            address: init.contract_address,
            receive_name: OwnedReceiveName::new_unchecked("ccdpiggybank.insert".to_string()),
            message: OwnedParameter::empty(),
        },)
            .expect("View product listings succeeds.")
            .parse_return_value()
            .expect("Deserialize product listings");

    assert_eq!(product_listings.len(), 1);
    assert_eq!(product_listings[0].product, "Apples");
}