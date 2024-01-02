// use concordium_smart_contract_testing::*;
// use gonana_concordium_smart_contract::*;

// /// A test account.
// const ALICE: AccountAddress = AccountAddress([0u8; 32]);
// const BOB: AccountAddress = AccountAddress([1u8; 32]);
// const ALICE_ADDR: Address = Address::Account(ALICE);
// const BOB_ADDR: Address = Address::Account(BOB);


// /// The initial balance of the ALICE test account.
// const ACC_INITIAL_BALANCE: Amount = Amount::from_ccd(10_000);

// /// A [`Signer`] with one set of keys, used for signing transactions.
// const SIGNER: Signer = Signer::with_one_key();


// fn setup_chain_and_contract() -> (Chain, ContractInitSuccess) {
//     let mut chain = Chain::new();

//     chain.create_account(Account::new(ALICE, ACC_INITIAL_BALANCE));
//     chain.create_account(Account::new(BOB, ACC_INITIAL_BALANCE));

//     let module = module_load_v1("gonana_marketplace.wasm.v1").expect("Module is valid and exists");
//     let deployment = chain.module_deploy_v1(SIGNER,ALICE , module).expect("Deploying valid module should succeed");

//     let initialization = chain
//     .contract_init(
//         SIGNER,
//         ALICE,
//         Energy::from(10000),
//         InitContractPayload {
//             mod_ref: deployment.module_reference,
//             init_name: OwnedContractName::new_unchecked("init_gonana_marketplace".to_string()),
//             param: OwnedParameter::empty(),
//             amount: Amount::zero(),
//         }
//     )
//     .expect("Initialization should always succeed");

//     (chain, initialization)
// }


// // Test confirming an escrow in the marketplace.
// #[test]
// fn test_confirm_escrow() {
//     let (mut chain, init) = setup_chain_and_contract();

//     //List a product in the marketplace.
//     chain
//         .contract_update(
//             SIGNER,
//             ALICE,
//             ALICE_ADDR,
//             Energy::from(10_000),
//             UpdateContractPayload {
//                 address: init.contract_address,
//                 amount: Amount::zero(),
//                 receive_name: OwnedReceiveName::new_unchecked("gonana_marketplace.list_product".to_string()),
//                 message: OwnedParameter::from_serial(&ListProductParameter {
//                     farmer: ALICE,
//                     product: "Oranges".to_string(),
//                     price: Amount::from_ccd(200),
//                 })
//                 .expect("Parameter within size bounds"),
//             },
//         )
//         .expect("List product succeeds.");

//     //View the product listings
//     let product_listings: Vec<ProductListing> =
//         chain
//             .contract_update(
//                 SIGNER,
//                 ALICE,
//                 Address::Account(ALICE),
//                 Energy::from(10000),
//                 UpdateContractPayload {
//                     amount: Amount::zero(),
//                     address: init.contract_address,
//                     receive_name: OwnedReceiveName::new_unchecked("gonana_marketplace.view_product_listings".to_string()),
//                     message: OwnedParameter::empty(),
//                 },
//             )
//             .expect("View product listings succeeds.")
//             .parse_return_value()
//             .expect("Deserialize product listings");

//     assert_eq!(product_listings.len(), 1);
//     assert_eq!(product_listings[0].product, "Oranges");

//     let alice_balance_before = chain.account_balance(ALICE).unwrap();
//     let bob_balance_before = chain.account_balance(BOB).unwrap();
//     let purchase_amount = Amount::from_ccd(200);

//     // Place an order in the marketplace.
//    let update = chain
//         .contract_update(
//             SIGNER,
//             BOB,
//             BOB_ADDR,
//             Energy::from(10_000),
//             UpdateContractPayload {
//                 address: init.contract_address,
//                 amount: purchase_amount,
//                 receive_name: OwnedReceiveName::new_unchecked("gonana_marketplace.place_order".to_string()),
//                 message: OwnedParameter::from_serial(&PlaceOrderParameter {
//                     product_name: "Oranges".to_string(),
//                     buyer: BOB,
//                     seller: ALICE,
//                 })
//                 .expect("Parameter within size bounds"),
//             },
//         );
        
//         assert!(update.is_ok());
//     assert_eq!(chain.account_balance(BOB).unwrap().total, (bob_balance_before.total - purchase_amount - update.as_ref().unwrap().transaction_fee));
//     assert_eq!(chain.contract_balance(init.contract_address), Some(purchase_amount)); 

//     // Confirm the escrow in the marketplace.
//  let update = chain
//  .contract_update(
//      SIGNER,
//      BOB,
//      BOB_ADDR,
//      Energy::from(10_000),
//      UpdateContractPayload {
//          address: init.contract_address,
//          amount: Amount::zero(),
//          receive_name: OwnedReceiveName::new_unchecked("gonana_marketplace.confirm_order".to_string()),
//          message: OwnedParameter::from_serial(&PlaceOrderParameter {
//              product_name: "Oranges".to_string(),
//              buyer: BOB,
//              seller: ALICE,
//          })
//          .expect("Parameter within size bounds"),
//      },
//  );
//         assert!(update.is_ok());
//         assert_eq!(chain.account_balance(ALICE).unwrap().total, (alice_balance_before.total + purchase_amount));


// //     // Verify that the product is in the Confirmed state.
//     let product_listings_after_confirm: Vec<ProductListing> =
//         chain
//             .contract_update(
//                 SIGNER,
//                 ALICE,
//                 Address::Account(ALICE),
//                 Energy::from(10000),
//                 UpdateContractPayload {
//                     amount: Amount::zero(),
//                     address: init.contract_address,
//                     receive_name: OwnedReceiveName::new_unchecked("gonana_marketplace.view_product_listings".to_string()),
//                     message: OwnedParameter::empty(),
//                 },
//             )
//             .expect("View product listings after confirm succeeds.")
//             .parse_return_value()
//             .expect("Deserialize product listings");

//     assert_eq!(product_listings_after_confirm.len(), 1);
//     assert_eq!(product_listings_after_confirm[0].state, ProductState::Confirmed);
// }
