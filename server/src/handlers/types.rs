use std::{sync::Arc, path::Path};
use concordium_rust_sdk::{v2, types::WalletAccount};
use concordium_std::Amount;
use serde::{Deserialize, Serialize};
use validator::Validate;
use anyhow::{Context,Error};
use concordium_rust_sdk::smart_contracts::common as concordium_std;
//use concordium_rust_sdk::smart_contracts::common::{self as contracts_common};
use std::str::FromStr;


// The product parameters used to list a product on the blockchain
#[derive(Validate, Deserialize, Serialize)]
pub struct ListProduct {
    /// The product_id generated off-chain, that signifies the product on chain.
    #[validate(length(min = 1, message = "product id is required"))]
    pub product_id: String, 
    /// Amount of the product.
    #[validate(length(min = 1, message = "amount is required"))]
    pub amount: String,
    /// Wallet address of the creator, could be None.
    pub wallet: Option<String>,
    /// Hash of the product parameters to prove intergrity. 
    pub hash: Option<String>,
    /// Farmer_id generated offchain that shows the id of a user.
    pub farmer_id: String
}


/// The product parameters used to list a product on the blockchain
#[derive(Debug, concordium_std::Serial)]
pub struct ListProductParam {
    /// The product_id generated off-chain, that signifies the product on chain.
    pub product_id: String, 
    /// Amount of the product.
    pub amount: Amount,
    /// Wallet address of the creator, could be None.
    pub wallet: Option<concordium_std::AccountAddress>,
    /// Hash of the product parameters to prove intergrity. 
    pub hash: Option<String>,
    /// Farmer_id generated offchain that shows the id of a user.
    pub farmer_id: String
}

impl ListProductParam {
    pub fn new(
        product_id:String,
        amount:String,
        wallet:Option<String>,
        farmer_id:String
    ) -> Self {
        let micro_ccd = amount.parse::<u64>()
            .context("interger could not be passed")
            .unwrap();
        let amount = Amount::from_micro_ccd(micro_ccd);
        match wallet {
            Some(wallet) => {
                let farmer = concordium_std::AccountAddress::from_str(&wallet).unwrap();
                Self{
                    product_id,
                    amount,
                    wallet:Some(farmer),
                    hash:None,
                    farmer_id}

            },
            None => {
                Self{product_id,amount,wallet:None,hash:None,farmer_id}

            }
        }
    }
}

#[derive(Debug)]
pub struct Deployer {
    /// The client to establish a connection to a Concordium node (V2 API).
    pub client: v2::Client,
    /// The account keys to be used for sending transactions.
    pub key: Arc<WalletAccount>,
}

impl Deployer {
    /// A function to create a new deployer instance from a network client and a path to the wallet.
    pub fn new(client: v2::Client, wallet_account_file: &Path) -> Result<Deployer, Error> {
        let key_data = WalletAccount::from_json_file(wallet_account_file)
            .context("Unable to read wallet file.")?;

        Ok(Deployer {
            client,
            key: key_data.into(),
        })
    }
}