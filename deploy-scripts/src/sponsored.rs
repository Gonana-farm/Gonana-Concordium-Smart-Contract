#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

pub mod deployer;
use anyhow::{Context, Error};
use clap::Parser;
use concordium_base::{cis2_types::{AdditionalData, Receiver}, base::Energy};
use concordium_cis2::TokenIdUnit;
use concordium_rust_sdk::{
    common::types::Amount,
    smart_contracts::{
        common::{self as contracts_common, Timestamp, OwnedEntrypointName, AccountSignatures},
        types::{OwnedContractName, OwnedParameter, OwnedReceiveName},
    },
    types::{
        smart_contracts::{ModuleReference, WasmModule},
        transactions,
        transactions::{send::GivenEnergy, InitContractPayload}, ContractAddress,
    },
    v2,
};
use contracts_common::AccountAddress;
use deployer::{DeployResult, Deployer, InitResult};
use gonana_concordium_smart_contract::{ListProductParameter, PermitMessage, PermitParam};
use std::{
    io::Cursor,
    path::{Path, PathBuf}, str::FromStr, u64::MAX,
};
use gona_token::{self, TOKEN_ID_GONA};
//use concordium_base::ed25519::SecretKey;
//use concordium_base::web3id::Web3IdSigner;
//use concordium_contracts_common::CredentialSignatures;
//use concordium_contracts_common::Signature;

/// Reads the wasm module from a given file path.
fn get_wasm_module(file: &Path) -> Result<WasmModule, Error> {
    let wasm_module = std::fs::read(file).context("Could not read the WASM file")?;
    let mut cursor = Cursor::new(wasm_module);
    let wasm_module: WasmModule = concordium_rust_sdk::common::from_bytes(&mut cursor)?;
    Ok(wasm_module)
}

/// Command line flags.
#[derive(clap::Parser, Debug)]
#[clap(author, version, about)]
struct App {
    #[clap(
        long = "node",
        default_value = "http://node.testnet.concordium.com:20000",
        help = "V2 API of the Concordium node."
    )]
    url: v2::Endpoint,
    #[clap(
        long = "account",
        help = "Path to the file containing the Concordium account keys exported from the wallet \
                (e.g. ./myPath/3PXwJYYPf6fyVb4GJquxSZU8puxrHfzc4XogdMVot8MUQK53tW.export)."
    )]
    key_file: PathBuf,
    #[clap(
        long = "module",
        help = "Path of the Concordium smart contract module. Use this flag several times if you \
                have several smart contract modules to be deployed (e.g. --module \
                ./myPath/default.wasm.v1 --module ./default2.wasm.v1)."
    )]
    module: Vec<PathBuf>,
}

/// Main function: It deploys to chain all wasm modules from the command line
/// `--module` flags. Write your own custom deployment/initialization script in
/// this function. An deployment/initialization script example is given in this
/// function for the `default` smart contract.
#[tokio::main]
async fn main() -> Result<(), Error> {
    let app: App = App::parse();

    let concordium_client = v2::Client::new(app.url).await?;

    let mut deployer = Deployer::new(concordium_client, &app.key_file)?;

    // let mut modules_deployed: Vec<ModuleReference> = Vec::new();

    // for contract in app.module {
    //     let wasm_module = get_wasm_module(contract.as_path())?;

    //     let deploy_result = deployer
    //         .deploy_wasm_module(wasm_module, None)
    //         .await
    //         .context("Failed to deploy a module.")?;

    //     match deploy_result {
    //         DeployResult::ModuleDeployed(module_deploy_result) => {
    //             modules_deployed.push(module_deploy_result.module_reference)
    //         }
    //         DeployResult::ModuleExists(module_reference) => modules_deployed.push(module_reference),
    //     }
    // }

    // Write your own deployment/initialization script below. An example is given
    // here.

    //let param: OwnedParameter = OwnedParameter::empty(); // Example

   // let init_method_name: &str = "init_gonana_marketplace"; // Example

    // use gona_token::SetMetadataUrlParams;

    // let meta_data = SetMetadataUrlParams{
    //     url: "https://gateway.pinata.cloud/ipfs/QmZBrF6HuoN12HyAznyk7gwFpnefooDbfxq3JeKTWToL1W".into(),
    //     hash: None
    // };

    // let param = OwnedParameter::from_serial(&init_method_name)?;

    // let payload = InitContractPayload {
    //     init_name: OwnedContractName::new(init_method_name.into())?,
    //     amount: Amount::from_micro_ccd(0),
    //     mod_ref: modules_deployed[0],
    //     param,
    // }; // Example

    
    // let init_result: InitResult = deployer
    //     .init_contract(payload, None, None)
    //     .await
    //     .context("Failed to initialize the contract.")?; // Example

    // This is how you can use a type from your smart contract.
    // use gonana_concordium_smart_contract::{ListProductParameter,PermitMessage,PermitParam}; // Example

    let farmer = contracts_common::AccountAddress::from_str("3UsPQ4MxhGNLEbYac53H7C2JHzE3Xe41zrgCdLVrp5vphx4YSe").unwrap();
    let amount = Amount::from_micro_ccd(100);

    let list_parameter =  ListProductParameter::new( 
        "bagofpotatoes".into(),
        amount,
        Some(farmer),
        "Steven".into()
    ); // Example

    //Time
    //let transaction_expiry_seconds = chrono::Utc::now().timestamp() as u64 + 3600 ;
   

    let permit_message = PermitMessage{
        contract_address: ContractAddress::new(7637, 0),
        nonce: 3,  //should be +1 at rerun
        timestamp: Timestamp::from_timestamp_millis(MAX),
        entry_point: OwnedEntrypointName::new_unchecked("internal_list_product".into()),
        payload: concordium_rust_sdk::smart_contracts::common::to_bytes(&list_parameter),
    };

    //Gona Token==========================================================

    // use gona_token::{WrapParams,ApproveParam,SpendParam};
    // use concordium_cis2::{AdditionalData,Receiver,TokenAmountU64};
    // let wrap_param = WrapParams{
    //     data: AdditionalData::empty(),
    //     to: Receiver::Account(AccountAddress::from_str("3UsPQ4MxhGNLEbYac53H7C2JHzE3Xe41zrgCdLVrp5vphx4YSe").unwrap())
    // };
     
    // let amount=TokenAmountU64(10);
    
    // let spender = contracts_common::Address::Account(AccountAddress::from_str("36J5gb5QVYBvbda4cZkagN4LvVCXejyX8ScuEx8xyAQckVjBMA".into()).unwrap());
    // let approve_param = ApproveParam::new(amount, spender);
    
    // let owner = contracts_common::Address::Account(AccountAddress::from_str("3Yk9hBWCS1wYf5xyuhtPLU22K2YgiHztPXCV43SsH5YV3ZDxKr".into()).unwrap());
    // let spend_param = SpendParam::new(amount, owner);
     
    

    //change secret key to bytes
    // let hex_string = "b5ad8b9e098d81bab8a6c7db970b899e036a4d69ab046c6a66caf84c91ba906f0a79b37eff8a99ad2b6792ab8d560825";
    // let bytes = hex::decode(hex_string).unwrap();
    // let mut byte_array = [0u8; 32];
    // for (index, &byte) in bytes.iter().enumerate() {
    //     byte_array[index] = byte;
    // }
    // let bytes: [u8; 48] = [ 181, 173, 139, 158, 9,141, 129, 186, 184, 166, 199, 219, 151, 11, 137, 158, 3, 106,77, 105, 171, 4, 108, 106, 102,202,248, 76,145,186, 144, 111,10,121, 179,126, 255, 138,153, 173, 43, 103, 146, 171,141,86, 8, 37];
    // // get secret key from byte array
    // let key = SecretKey::from_bytes(&byte_array)?;
    
    // change list_parameter to bytes
    let serialized_list_param = contracts_common::to_bytes(&list_parameter);


    // // sign the list parameter
    // let signature = key.sign(&serialized_list_param);
    let signature = deployer.key.keys.sign_message(&serialized_list_param);

    // // change signature to vec of u8
    // let sig = signature.to_bytes();
    

    // // construct a signature BTreeMap, that will be used to create a C
    // let mut inner_signature_map = std::collections::BTreeMap::new();
    // inner_signature_map.insert(0, Signature::Ed25519(contracts_common::SignatureEd25519(sig)));
  
    // // construct a credential with the signature btree map
    // let mut signature_map = std::collections::BTreeMap::new();
    // signature_map.insert(
    //     0,
    //     CredentialSignatures {
    //         sigs: inner_signature_map,
    //     },
    // );

    // get signer
    //let signer = AccountAddress::from_str("36J5gb5QVYBvbda4cZkagN4LvVCXejyX8ScuEx8xyAQckVjBMA")?;
    
    // construct permit param 
    let param: PermitParam = PermitParam {
        message : permit_message,
        signature,
        // signature: AccountSignatures {
        //     sigs: signature_map,
        // },
        signer: deployer.key.address
    };


    // Create a successful transaction.

    let bytes = contracts_common::to_bytes(&param); // Example
    //let bytes = contracts_common::to_bytes(&spend_param);




    let update_payload = transactions::UpdateContractPayload {
        amount: Amount::from_ccd(0),
        //address: init_result.contract_address, 
        address: ContractAddress::new(7637, 0),  
        receive_name: OwnedReceiveName::new_unchecked("gonana_marketplace.permit".to_string()),
        //receive_name: OwnedReceiveName::new_unchecked("gona_token.transfer_from".to_string()),
        message: bytes.try_into()?,
    }; // Example




    // The transaction costs on Concordium have two components, one is based on the size of the
    // transaction and the number of signatures, and then there is a
    // transaction-specific one for executing the transaction (which is estimated with this function).
    //let mut energy = deployer
        //.estimate_energy(update_payload.clone())
        //.await
        //.context("Failed to estimate the energy.")?; // Example

    // We add 100 energy to be safe.
    //energy.energy += 100; // Example

    // `GivenEnergy::Add(energy)` is the recommended helper function to handle the transaction cost automatically for the first component
    // (based on the size of the transaction and the number of signatures).
    // [GivenEnergy](https://docs.rs/concordium-rust-sdk/latest/concordium_rust_sdk/types/transactions/construct/enum.GivenEnergy.html)
    
    
    
    
    let _update_contract = deployer
         .update_contract(update_payload, Some(GivenEnergy::Add(Energy::from_str("1500000")?)), None)
         .await
         .context("Failed to update the contract.")?; // Example





    // Write your own deployment/initialization script above. An example is given
    // here.
    //let bi = concordium_rust_sdk::v2::BlockIdentifier::LastFinal; 
    //let address =  ContractAddress::new(7603, 0);//init_result.contract_address, 
    //let receive_name = OwnedReceiveName::new_unchecked("gona_id".to_string());

    //let context = concordium_rust_sdk::types::smart_contracts::ContractContext::new(address,receive_name);
    //let _res = client.invoke_instance(bi, &context).await?.response;
    //println!("{res}");
    Ok(())
}



//Gonana_MarketPlace
//contract_address: ContractAddress::new(7572, 0),



//Gona Token
// Initializing contract....
//Sent transaction with hash: 1fade06b697238e3ee6983cf209d018bd6e8ff77572db2ed36cddd2356cfefd8
//Transaction finalized: tx_hash=1fade06b697238e3ee6983cf209d018bd6e8ff77572db2ed36cddd2356cfefd8 contract=(7625, 0)