#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract

use concordium_std::*;
use core::fmt::Debug;
use concordium_std::Amount;


/// Enum representing the possible states of a product
#[derive(Debug, Serialize, SchemaType, PartialEq, Eq, Clone)]
pub enum ProductState {
    Listed,
    Escrowed,
    Confirmed,
    Cancelled,
}


/// The product_id generated off-chain, that signifies the product on chain.
#[derive(Serialize, Clone, SchemaType, PartialEq, Eq, Debug)]
pub struct ProductListing {
    product_id: String, 
    /// Amount of the product.
    amount: Amount,
    /// Wallet address of the creator, could be None.
    wallet: Option<AccountAddress>,
    /// Hash of the product parameters to prove intergrity. 
    hash: Option<String>,
    /// Farmer_id generated offchain that shows the id of a user.
    merchant_id: String,
    /// The State of Product
    state: ProductState
}


// Struct to represent an order.
#[derive( Serialize, SchemaType, Eq, PartialEq, PartialOrd, Clone)]
pub struct Order {
    pub product_id: String,
    pub amount: Amount,
    pub buyer_address: Option<AccountAddress>,
    pub buyer_id: String
}



#[derive(Serialize, SchemaType)]
pub struct PlaceOrderParameter {
    pub product_id: String,
    pub buyer_address:Option<AccountAddress>,
    pub buyer_id: String,
    pub amount: Amount,

}

/// Error types
#[derive(Debug, PartialEq, Eq, Clone, Reject, Serialize, SchemaType)]
pub enum MarketplaceError {
    ProductNotFound,
    OrderNotFound,
    OrderAlreadyExists,
    InvalidProductState,
    InsufficientFunds,
    InvalidPrice,
    #[from(ParseError)]
    ParseParams,
    #[from(TransferError)]
    TransferError,
    NonceAlreadyUsed,
    WrongContract,
    Expired,
    WrongFunctionCall,
    WrongSignature
}

/// The parameter used to list product on the blockchain.
#[derive(Serialize, SchemaType)]
pub struct ListProductParameter{
    product_id: String, 
    /// Amount of the product.
    amount: Amount,
    /// Wallet address of the creator, could be None.
    wallet: Option<AccountAddress>,
    /// Hash of the product parameters to prove intergrity. 
    hash: Option<String>,
    /// Farmer_id generated offchain that shows the id of a user.
    merchant_id: String
}

impl ListProductParameter {
    pub fn new(
        product_id:String,
        amount:Amount,
        wallet:Option<AccountAddress>,
        merchant_id:String
    ) -> Self {
        Self{product_id,amount,wallet,hash:None,merchant_id}
    }
    // just a stupid implementation, in reality all the information will be hashed and sent to the blockchain
    pub fn hash(&self,crypto_primitives: &impl HasCryptoPrimitives) -> Self {
        let payload = self;
        let bytes = to_bytes(payload);
        let hash = crypto_primitives.hash_sha2_256(&bytes).to_string();
        Self {
            product_id: self.product_id.clone(),
            amount: self.amount,
            wallet: self.wallet,
            hash: Some(hash),
            merchant_id: self.merchant_id.clone(),
        }

    }
}



#[derive(Serialize, SchemaType)]
pub struct CancelProductParameter{
    pub product_id: String,
    pub merchant_id: String
}






/// List of supported entrypoints by the `permit` function (CIS3 standard).
//const SUPPORTS_PERMIT_ENTRYPOINTS: [EntrypointName; 2] =
    //[EntrypointName::new_unchecked("updateOperator"), EntrypointName::new_unchecked("list_product")];

/// Tag for the CIS3 Nonce event.
pub const NONCE_EVENT_TAG: u8 = u8::MAX - 5;

/// Tagged events to be serialized for the event log.
#[derive(Debug, Serial, Deserial, PartialEq, Eq)]
#[concordium(repr(u8))]
pub enum Event {
    /// The event tracks the nonce used by the signer of the `PermitMessage`
    /// whenever the `permit` function is invoked.
    #[concordium(tag = 250)]
    Nonce(NonceEvent),
}

/// The NonceEvent is logged when the `permit` function is invoked. The event
/// tracks the nonce used by the signer of the `PermitMessage`.
#[derive(Debug, Serialize, SchemaType, PartialEq, Eq)]
pub struct NonceEvent {
    /// Account that signed the `PermitMessage`.
    pub account: AccountAddress,
    /// The nonce that was used in the `PermitMessage`.
    pub nonce:   u64,
}


// Implementing a custom schemaType to the `Event` combining all CIS2/CIS3
// events.
impl schema::SchemaType for Event {
    fn get_type() -> schema::Type {
        let mut event_map = collections::BTreeMap::new();
        event_map.insert(
            NONCE_EVENT_TAG,
            (
                "Nonce".to_string(),
                schema::Fields::Named(vec![
                    (String::from("account"), AccountAddress::get_type()),
                    (String::from("nonce"), u64::get_type()),
                ]),
            ),
        );
        schema::Type::TaggedEnum(event_map)
    }
}


/// Part of the parameter type for the contract function `permit`.
/// Specifies the message that is signed.
#[derive(SchemaType, Serialize)]
pub struct PermitMessage {
    /// The contract_address that the signature is intended for.
    pub contract_address: ContractAddress,
    /// A nonce to prevent replay attacks.
    pub nonce:            u64,
    /// A timestamp to make signatures expire.
    pub timestamp:        Timestamp,
    /// The entry_point that the signature is intended for.
    pub entry_point:      OwnedEntrypointName,
    /// The serialized payload that should be forwarded to either the `transfer`
    /// or the `updateOperator` function.
    #[concordium(size_length = 2)]
    pub payload:          Vec<u8>,
}

/// The parameter type for the contract function `permit`.
/// Takes a signature, the signer, and the message that was signed.
#[derive(Serialize, SchemaType)]
pub struct PermitParam {
    /// Signature/s. The CIS3 standard supports multi-sig accounts.
    pub signature: AccountSignatures,
    /// Account that created the above signature.
    pub signer:    AccountAddress,
    /// Message that was signed.
    pub message:   PermitMessage,
}

#[derive(Serialize)]
pub struct PermitParamPartial {
    /// Signature/s. The CIS3 standard supports multi-sig accounts.
    signature: AccountSignatures,
    /// Account that created the above signature.
    signer:    AccountAddress,
}















/// Smart contract state
#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S = StateApi>  {
    pub product_listings: StateMap<String,ProductListing, S>,
    pub orders: StateMap<String,Order,S>,
    nonces_registry:  StateMap<AccountAddress, u64, S>,

}

// impl State{

//     fn list_product(&mut self, farmer: AccountAddress, product: String, price: Amount) -> Result<(),MarketplaceError>{
//         let listing = ProductListing {
//             farmer,
//             product: product.clone(),
//             price,
//             state:ProductState::Listed
//         };
//         self.product_listings.insert(product,listing);
//         Ok(())
//     }

// }




// Init function to initialize the marketplace state
#[init(contract = "gonana_marketplace")]
fn init(_ctx: &InitContext, state_builder: &mut StateBuilder) -> InitResult<State>{
    Ok(State { 
            product_listings: state_builder.new_map(),
            orders: state_builder.new_map(),
            nonces_registry:  state_builder.new_map(),
     })
}

// internal list function that will be executed by the permit message
fn internal_list_product(host: &mut Host<State>,params:ListProductParameter) -> Result<(),MarketplaceError>{
    let (state, _builder) = host.state_and_builder();
    
    let listing = ProductListing {
        merchant_id: params.merchant_id,
        product_id: params.product_id.clone(),
        amount: params.amount,
        wallet: params.wallet,
        hash: params.hash,
        state:ProductState::Listed
    };
    state.product_listings.insert(params.product_id,listing);
    Ok(())
}

/// Response type for the function `publicKeyOf`.
#[derive(Debug, Serialize, SchemaType)]
#[concordium(transparent)]
pub struct PublicKeyOfQueryResponse(
    #[concordium(size_length = 2)] pub Vec<Option<AccountPublicKeys>>,
);

impl From<Vec<Option<AccountPublicKeys>>> for PublicKeyOfQueryResponse {
    fn from(results: concordium_std::Vec<Option<AccountPublicKeys>>) -> Self {
        PublicKeyOfQueryResponse(results)
    }
}

/// The parameter type for the contract functions `publicKeyOf/noneOf`. A query
/// for the public key/nonce of a given account.
#[derive(Debug, Serialize, SchemaType)]
#[concordium(transparent)]
pub struct VecOfAccountAddresses {
    /// List of queries.
    #[concordium(size_length = 2)]
    pub queries: Vec<AccountAddress>,
}

/// Get the public keys of accounts. `None` is returned if the account does not
/// exist on chain.
///
/// It rejects if:
/// - It fails to parse the parameter.
#[receive(
    contract = "gonana_marketplace",
    name = "publicKeyOf",
    parameter = "VecOfAccountAddresses",
    return_value = "PublicKeyOfQueryResponse",
    error = "ContractError"
)]
fn contract_public_key_of(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> Result<PublicKeyOfQueryResponse,MarketplaceError> {
    // Parse the parameter.
    let params: VecOfAccountAddresses = ctx.parameter_cursor().get()?;
    // Build the response.
    let mut response: Vec<Option<AccountPublicKeys>> = Vec::with_capacity(params.queries.len());
    for account in params.queries {
        // Query the public_key.
        let public_keys = host.account_public_keys(account).ok();

        response.push(public_keys);
    }
    let result = PublicKeyOfQueryResponse::from(response);
    Ok(result)
}

/// Response type for the function `nonceOf`.
#[derive(Debug, Serialize, SchemaType)]
#[concordium(transparent)]
pub struct NonceOfQueryResponse(#[concordium(size_length = 2)] pub Vec<u64>);

impl From<Vec<u64>> for NonceOfQueryResponse {
    fn from(results: concordium_std::Vec<u64>) -> Self { NonceOfQueryResponse(results) }
}

/// Get the nonces of accounts.
///
/// It rejects if:
/// - It fails to parse the parameter.
#[receive(
    contract = "gonana_marketplace",
    name = "nonceOf",
    parameter = "VecOfAccountAddresses",
    return_value = "NonceOfQueryResponse",
    error = "MaketplaceError"
)]
fn contract_nonce_of(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> Result<NonceOfQueryResponse,MarketplaceError> {
    // Parse the parameter.
    let params: VecOfAccountAddresses = ctx.parameter_cursor().get()?;
    // Build the response.
    let mut response: Vec<u64> = Vec::with_capacity(params.queries.len());
    for account in params.queries {
        // Query the next nonce.
        let nonce = host.state().nonces_registry.get(&account).map(|nonce| *nonce).unwrap_or(0);

        response.push(nonce);
    }
    Ok(NonceOfQueryResponse::from(response))
}



/// Helper function to calculate the `message_hash`.
#[receive(
    contract = "gonana_marketplace",
    name = "viewMessageHash",
    parameter = "PermitParam",
    return_value = "[u8;32]",
    crypto_primitives,
    mutable
)]
fn contract_view_message_hash(
    ctx: &ReceiveContext,
    _host: &mut Host<State>,
    crypto_primitives: &impl HasCryptoPrimitives,
) -> Result<[u8; 32],MarketplaceError> {
    // Parse the parameter.
    let mut cursor = ctx.parameter_cursor();
    // The input parameter is `PermitParam` but we only read the initial part of it
    // with `PermitParamPartial`. I.e. we read the `signature` and the
    // `signer`, but not the `message` here.
    let param: PermitParamPartial = cursor.get()?;

    // The input parameter is `PermitParam` but we have only read the initial part
    // of it with `PermitParamPartial` so far. We read in the `message` now.
    // `(cursor.size() - cursor.cursor_position()` is the length of the message in
    // bytes.
    let mut message_bytes = vec![0; (cursor.size() - cursor.cursor_position()) as usize];

    cursor.read_exact(&mut message_bytes)?;

    // The message signed in the Concordium browser wallet is prepended with the
    // `account` address and 8 zero bytes. Accounts in the Concordium browser wallet
    // can either sign a regular transaction (in that case the prepend is
    // `account` address and the nonce of the account which is by design >= 1)
    // or sign a message (in that case the prepend is `account` address and 8 zero
    // bytes). Hence, the 8 zero bytes ensure that the user does not accidentally
    // sign a transaction. The account nonce is of type u64 (8 bytes).
    let mut msg_prepend = [0; 32 + 8];
    // Prepend the `account` address of the signer.
    msg_prepend[0..32].copy_from_slice(param.signer.as_ref());
    // Prepend 8 zero bytes.
    msg_prepend[32..40].copy_from_slice(&[0u8; 8]);
    // Calculate the message hash.
    let message_hash =
        crypto_primitives.hash_sha2_256(&[&msg_prepend[0..40], &message_bytes].concat()).0;

    Ok(message_hash)
}


#[receive(
    contract = "gonana_marketplace",
    name = "permit",
    parameter = "PermitParam",
    crypto_primitives,
    mutable,
    enable_logger
)]
fn contract_permit(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut impl HasLogger,
    crypto_primitives: &impl HasCryptoPrimitives,
) -> Result<(),MarketplaceError> {

    // Parse the parameter.
    let param: PermitParam = ctx.parameter_cursor().get()?;

    // Update the nonce.
    let mut entry = host.state_mut().nonces_registry.entry(param.signer).or_insert_with(|| 0);

    // Get the current nonce.
    let nonce = *entry;
    // Bump nonce.
    *entry += 1;
    drop(entry);

    let message = param.message;
    let list_param = message.payload.clone();

    // Check the nonce to prevent replay attacks.
    ensure_eq!(message.nonce, nonce, MarketplaceError::NonceAlreadyUsed);

     // Check that the signature was intended for this contract.
     ensure_eq!(
        message.contract_address,
        ctx.self_address(),
        MarketplaceError::WrongContract.into()
    );
    // Check signature is not expired.
    ensure!(message.timestamp > ctx.metadata().slot_time(), MarketplaceError::Expired.into());
    let _message_hash = contract_view_message_hash(ctx, host, crypto_primitives)?;

    // Check signature.
    let valid_signature = host.check_account_signature(param.signer, &param.signature, &list_param)
        .expect("account signature was incorrect");
    ensure!(valid_signature, MarketplaceError::WrongSignature);
    
    //Execute Function Calls
    if message.entry_point.as_entrypoint_name() == EntrypointName::new_unchecked("internal_list_product") {
        let params: ListProductParameter = from_bytes(&message.payload).expect("could not unwrap payload");
        let response = internal_list_product(host, params)?;
        // Log the nonce event.
        logger.log(&Event::Nonce(NonceEvent {
            account: param.signer,
            nonce,
        })).expect("events could not be logged");
        Ok(response)

    }else if message.entry_point.as_entrypoint_name() == EntrypointName::new_unchecked("place_order") {
        let params: PlaceOrderParameter = from_bytes(&message.payload).expect("could not unwrap payload");
        let state_mut = host.state_mut();
        // Find the product by name
        let mut product = 
            state_mut
            .product_listings
            .get_mut(&params.product_id)
            .ok_or(MarketplaceError::ProductNotFound)?;
    
        // Ensure that the product is in a valid state for placing an order
        ensure!(product.state == ProductState::Listed, MarketplaceError::InvalidProductState);    
        let order = Order {
            buyer_address:   params.buyer_address,
            product_id: product.product_id.clone(),
            amount: product.amount,
            buyer_id: params.buyer_id
        }; 
        // Insert the order and update the product state to Escrowed 
        ensure!(state_mut.orders.insert(params.product_id.clone(), order).is_none(), MarketplaceError::OrderAlreadyExists);    
        product.state = ProductState::Escrowed;    
        Ok(())

        // CANCEL PLACED ORDERS!!!!!
    }else if message.entry_point.as_entrypoint_name() == EntrypointName::new_unchecked("cancel_order") {
        let params: CancelProductParameter = from_bytes(&message.payload).expect("could not unwrap payload");
        let product_name = params.product_id.clone();

    // Check if the product is found
    if let Some(mut listing) = host.state_mut().product_listings.get_mut(&product_name) {
        // Check if the product is in a cancellable state
        ensure!(params.merchant_id == listing.merchant_id, MarketplaceError::WrongSignature);

        match listing.state {
            ProductState::Listed | ProductState::Escrowed => {
                listing.state = ProductState::Cancelled;
                Ok(())
            }
            _ => Err(MarketplaceError::InvalidProductState),
            }
        } else {
            // Product not found
            Err(MarketplaceError::ProductNotFound)
        }

    }
    else{
        Err(MarketplaceError::WrongFunctionCall)
    }

}



/// Function to list a product in the marketplace
#[receive(contract = "gonana_marketplace", name = "list_product", parameter = "ListProductParameter", mutable )]
fn list_product(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), MarketplaceError>{
    let parameter: ListProductParameter = ctx.parameter_cursor().get()?;
   
    //ensure product has not been listed before
    //can be handled by the web2 backend though
    //todo!()


    // Check if the price is 0
    if parameter.amount <= Amount::zero() {
        return Err(MarketplaceError::InvalidPrice);
    }

    // Check if the product name is empty
    if parameter.product_id.is_empty() {
        return Err(MarketplaceError::ParseParams);
    }

    let listing = ProductListing {
            merchant_id: parameter.merchant_id,
            product_id: parameter.product_id.clone(),
            amount: parameter.amount,
            wallet: parameter.wallet,
            hash: parameter.hash,
            state:ProductState::Listed
        };
        
    host.state_mut().product_listings.insert(parameter.product_id.clone(), listing);
    Ok(()) 
}



/// Function to cancel or unlist a product
#[receive(contract = "gonana_marketplace", name = "cancel_product", parameter = "CancelProductParameter", mutable )]
fn cancel_product(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), MarketplaceError>{
    let parameter: CancelProductParameter = ctx.parameter_cursor().get()?;
    let product_name = parameter.product_id.clone();

    // Check if the product is found
    if let Some(mut listing) = host.state_mut().product_listings.get_mut(&product_name) {
        ensure!(parameter.merchant_id == listing.merchant_id, MarketplaceError::WrongSignature);
        // Check if the product is in a cancellable state
        match listing.state {
            ProductState::Listed | ProductState::Escrowed => {
                listing.state = ProductState::Cancelled;
                Ok(())
            }
            _ => Err(MarketplaceError::InvalidProductState),
        }
    } else {
        // Product not found
        Err(MarketplaceError::ProductNotFound)
    }
}

// buy a product
#[receive(contract = "gonana_marketplace", name="place_order", parameter = "PlaceOrderParameter", mutable, payable)]
fn place_order(ctx: &ReceiveContext, host: &mut Host<State>, amount: Amount) -> Result<(), MarketplaceError> {
    let parameter: PlaceOrderParameter = ctx.parameter_cursor().get()?;

    let state_mut = host.state_mut();

    // Find the product by name
    let mut product = 
        state_mut
        .product_listings
        .get_mut(&parameter.product_id)
        .ok_or(MarketplaceError::ProductNotFound)?;

    // Ensure that the product is in a valid state for placing an order
    ensure!(product.state == ProductState::Listed, MarketplaceError::InvalidProductState);    
    // Ensure that the full amount was paid
    ensure!(amount >= product.amount, MarketplaceError::InvalidPrice);

    // Create an order
    let order = Order {
        buyer_address:   parameter.buyer_address,
        product_id: product.product_id.clone(),
        amount,
        buyer_id: parameter.buyer_id
    };

    // Insert the order and update the product state to Escrowed 
    ensure!(state_mut.orders.insert(parameter.product_id.clone(), order).is_none(), MarketplaceError::OrderAlreadyExists);

    // If the insert is successful, update the product state
     // Update the product state to Escrowed
     product.state = ProductState::Escrowed;
    Ok(())

}



//function to confirm an escrow 
#[receive(contract = "gonana_marketplace", name = "confirm_order", parameter = "PlaceOrderParameter", mutable)]
fn confirm_order(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), MarketplaceError> {
     let param:PlaceOrderParameter = ctx.parameter_cursor().get()?;
     let product_id = param.product_id.clone();

     let state_mut = host.state_mut();

      // Find the product by name
    let mut product = 
    state_mut
        .product_listings
        .remove_and_get(&product_id) //once and order is confirmed, remove from product listing???
        .ok_or(MarketplaceError::ProductNotFound)?;

    // Ensure that the product is in a valid state for confirming the escrow
    ensure!(product.state == ProductState::Escrowed, MarketplaceError::InvalidProductState);
    
    product.state = ProductState::Confirmed;
    let _merchant = product.merchant_id.clone();
    let amount = product.amount.clone();
    let merchant_address = product.wallet.clone();
    drop(product);


     //Get the order by product_name
     if let Some(order) = host.state_mut().orders.remove_and_get(&param.product_id) {
        // Transfer funds
        match merchant_address {
            Some(wallet) => {
                host.invoke_transfer(&wallet, amount)?;
            },
            None => ()
        }
        // It's important to call `Deletable::delete` on the value
        order.delete();
        Ok(())
    } else {
        // Escrow not found
        Err(MarketplaceError::OrderNotFound)
    }
}




// // View function to get all product listings
#[receive(contract = "gonana_marketplace", name = "view_product_listings", return_value = "Vec<ProductListing>")]
fn view_product_listings(_ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<Vec<ProductListing>> {
    let state = host.state();
    let product_listings: Vec<ProductListing> = state.product_listings.iter().map(|(_, product)| product.clone()).collect();
    Ok(product_listings)
}

// // View function to get all orders
#[receive(contract = "gonana_marketplace", name = "view_orders", return_value = "Vec<Order>")]
fn view_orders(_ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<Vec<Order>> {
    let state = host.state();
    let orders: Vec<Order> = state.orders.iter().map(|(_, order)| order.clone()).collect();
    Ok(orders)
}


//list_product
//concordium-client contract update  gonana_marketplace_instance --entrypoint list_product --parameter-json ./list_product.json --schema ./schema.bin --sender TimConcordiumWallet  --energy 2000 --grpc-port 20000 --grpc-ip node.testnet.concordium.com

//cancel_product
//concordium-client contract update  gonana_marketplace_instance --entrypoint cancel_product --parameter-json ./cancel_product.json --schema ./schema.bin --sender TimConcordiumWallet  --energy 2000 --grpc-port 20000 --grpc-ip node.testnet.concordium.com

//view product_listings
//concordium-client contract invoke   gonana_marketplace_instance --entrypoint view_product_listings --energy 2000 --grpc-port 20000 --grpc-ip node.testnet.concordium.com

//view orders
// concordium-client contract invoke   gonana_marketplace_instance --entrypoint view_orders --energy 2000 --grpc-port 20000 --grpc-ip node.testnet.concordium.com

//place_orders
//concordium-client contract update  gonana_marketplace_instance --entrypoint place_order --parameter-json ./place_order.json --amount 1000 --schema ./schema.bin --sender TimConcordiumWallet  --energy 3000 --grpc-port 20000 --grpc-ip node.testnet.concordium.com


//confirm_orders
//concordium-client contract update  gonana_marketplace_instance --entrypoint confirm_order --parameter-json ./place_order.json  --schema ./schema.bin --sender TimConcordiumWallet  --energy 3000 --grpc-port 20000 --grpc-ip node.testnet.concordium.com