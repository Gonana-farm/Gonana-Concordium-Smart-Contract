use std::str::FromStr;
use actix_web::{post,Responder,HttpResponse, web::Json};
//use actix_web::web::Path;
use concordium_base::base::Energy;
use validator::Validate;
use log::info;
use concordium_rust_sdk::{
    common::types::Amount,
    types::{
        smart_contracts::{self, ContractContext},
        transactions,ContractAddress,
    },
    v2::{self, Endpoint, Client},
};
use concordium_rust_sdk::smart_contracts::types::InvokeContractResult;


const ENERGY: u64 = 60000;
use crate::handlers::types::{ListProductParam,Deployer,ListProduct};

#[post("/product/list")]
pub async fn list_product(
    body: Json<ListProduct>,
) -> impl Responder {
    //env_logger::builder().filter_level(LevelFilter::Info).init();
    let is_valid = body.validate();
    match is_valid {
        Ok(_) => {
            let id = body.product_id.clone();
            info!("{id} has been ordered");
            let (deployer, mut client) = get_deployer().await.expect("error while getting deployer");
            log::info!("Acquired keys from path.");
            let param = body.0;
            info!("sponsored contract interaction has started");
            let nonce_response = client
                .get_next_account_sequence_number(&deployer.key.address)
                .await
                .map_err(|e| {
                    log::warn!("NonceQueryError {:#?}.", e);
                }).unwrap();
                
            info!("Process Started..........................");
            log::info!("Create payload.");
            let payload = ListProductParam::new(param.product_id, param.amount, param.wallet, param.farmer_id);
            //serialize to bytes
            let bytes = concordium_rust_sdk::smart_contracts::common::to_bytes(&payload);
            // check to owned parameter
            let parameter = smart_contracts::OwnedParameter::try_from(bytes)
                .expect("could not unwrap parameter");
            // receive method name on the contract
            let receive_name = smart_contracts::OwnedReceiveName::try_from("gonana_marketplace.list_product".to_string()).unwrap();
            log::info!("Simulate transaction to check its validity.");
            
            //Simulate Transaction
            let context = ContractContext {
                invoker: Some(concordium_rust_sdk::types::Address::Account(deployer.key.address)),
                contract: ContractAddress::new(7630, 0),
                amount: Amount::zero(),
                method: receive_name.clone(),
                parameter: parameter.clone(),
                energy: Energy { energy: 60000000 },
            };

            let info = client
            .invoke_instance(&concordium_rust_sdk::v2::BlockIdentifier::Best, &context)
            .await;

            match &info.as_ref().unwrap().response {
                InvokeContractResult::Success {
                    return_value: _,
                    events: _,
                    used_energy: _,
                } => log::info!("TransactionSimulationSuccess"),
                InvokeContractResult::Failure {
                    return_value: _,
                    reason,
                    used_energy: _,
                } => {
                    log::info!("TransactionSimulationError {:#?}.", reason);
                    return HttpResponse::BadRequest().body("simulation failed for some reason")                 

                }
            }
            
            //if simuation was succesfull, send transaction
                
                log::info!("Transaction simulation was successful");
                log::info!("Create transaction.");
                let payload = transactions::Payload::Update {
                    payload: transactions::UpdateContractPayload {
                        amount: Amount::from_micro_ccd(0),
                        address: ContractAddress::new(7630, 0),
                        receive_name,
                        message: parameter,
                    },
                };

                let transaction_expiry_seconds = chrono::Utc::now().timestamp() as u64 + 3600;
                
                
                let tx = transactions::send::make_and_sign_transaction(
                    &deployer.key.keys,
                    deployer.key.address,
                    nonce_response.nonce,
                    concordium_base::common::types::TransactionTime {
                        seconds: transaction_expiry_seconds,
                    },
                    concordium_rust_sdk::types::transactions::send::GivenEnergy::Absolute(Energy {
                        energy: ENERGY,
                    }),
                    payload,
                );

                let bi = transactions::BlockItem::AccountTransaction(tx);
                log::info!("Submit transaction.");
                match client.send_block_item(&bi).await {
                    Ok(hash) => {
                        HttpResponse::Ok().json(format!("id: {id}, hash:{hash}"))
                    }
                    Err(e) => {
                        log::error!("SubmitSponsoredTransactionError {:#?}.", e);
                        HttpResponse::BadRequest().body("request failed for some reason")                 
                    }
                }
        },
        Err(_) => {
            HttpResponse::BadRequest().body("request has no pizza name")
        }
    }
}

async fn get_deployer()->Result<(Deployer,Client),anyhow::Error>{
    let node = "http://node.testnet.concordium.com:20000";
    let endpoint = Endpoint::from_str(node)?;
    let concordium_client = v2::Client::new(endpoint).await?;
    let client_transfer = concordium_client.clone();
    let key = std::path::Path::new("./key/3UsPQ4MxhGNLEbYac53H7C2JHzE3Xe41zrgCdLVrp5vphx4YSe.export");
    let deployer = Deployer::new(concordium_client,key)?;
    Ok((deployer,client_transfer))
}

