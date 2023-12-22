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


// Struct to represent a product listing.
#[derive(Serialize, Clone, SchemaType, PartialEq, Eq, Debug)]
pub struct ProductListing {
    pub farmer: AccountAddress,
    pub product: String,
    pub price: Amount,
    pub state: ProductState
}


// Struct to represent an order.
#[derive( Serialize, SchemaType, Eq, PartialEq, PartialOrd, Clone)]
pub struct Order {
    pub buyer: AccountAddress,
    pub product: String,
    pub price: Amount,
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
}

#[derive(Serialize, SchemaType)]
pub struct ListProductParameter{
    pub farmer: AccountAddress,
    pub product: String,
    pub price: Amount,
}

#[derive(Serialize, SchemaType)]
pub struct CancelProductParameter{
    pub product_name: String,
}

#[derive(Serialize, SchemaType)]
pub struct PlaceOrderParameter {
    pub product_name: String,
    pub buyer:AccountAddress,
    pub seller: AccountAddress,
}


/// Smart contract state
#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S = StateApi>  {
    pub product_listings: StateMap<String,ProductListing, S>,
    pub orders: StateMap<String,Order,S>,
}


// Init function to initialize the marketplace state
#[init(contract = "gonana_marketplace")]
fn init(_ctx: &InitContext, state_builder: &mut StateBuilder) -> InitResult<State>{
    Ok(State { 
            product_listings: state_builder.new_map(),
            orders: state_builder.new_map(),
     })
}


/// Function to list a product in the marketplace
#[receive(contract = "gonana_marketplace", name = "list_product", parameter = "ListProductParameter", mutable )]
fn list_product(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), MarketplaceError>{
    let parameter: ListProductParameter = ctx.parameter_cursor().get()?;
   
    // Check if the price is 0
    if parameter.price <= Amount::zero() {
        return Err(MarketplaceError::InvalidPrice);
    }

    // Check if the product name is empty
    if parameter.product.is_empty() {
        return Err(MarketplaceError::ParseParams);
    }

    let listing = ProductListing {
        farmer: parameter.farmer,
        product: parameter.product.clone(),
        price: parameter.price,
        state: ProductState::Listed,
    };
    
    host.state_mut().product_listings.insert(parameter.product.clone(), listing);

    Ok(()) 
}



/// Function to cancel or unlist a product
#[receive(contract = "gonana_marketplace", name = "cancel_product", parameter = "CancelProductParameter", mutable )]
fn cancel_product(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), MarketplaceError>{
    let parameter: CancelProductParameter = ctx.parameter_cursor().get()?;
    let product_name = parameter.product_name.clone();

    // Check if the product is found
    if let Some(mut listing) = host.state_mut().product_listings.get_mut(&product_name) {
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

#[receive(contract = "gonana_marketplace", name="place_order", parameter = "PlaceOrderParameter", mutable, payable)]
fn place_order(ctx: &ReceiveContext, host: &mut Host<State>, amount: Amount) -> Result<(), MarketplaceError> {
    let parameter: PlaceOrderParameter = ctx.parameter_cursor().get()?;

    let state_mut = host.state_mut();

    // Find the product by name
    let mut product = 
        state_mut
        .product_listings
        .get_mut(&parameter.product_name)
        .ok_or(MarketplaceError::ProductNotFound)?;

    // Ensure that the product is in a valid state for placing an order
    ensure!(product.state == ProductState::Listed, MarketplaceError::InvalidProductState);


    // Create an order
    let order = Order {
        buyer:   parameter.buyer,
        product: product.product.clone(),
        price:   amount,
    };

    // Insert the order and update the product state to Escrowed 
    ensure!(state_mut.orders.insert(parameter.product_name.clone(), order).is_none(), MarketplaceError::OrderAlreadyExists);

    // If the insert is successful, update the product state
     // Update the product state to Escrowed
     product.state = ProductState::Escrowed;
    Ok(())

}


//function to confirm an escrow 
#[receive(contract = "gonana_marketplace", name = "confirm_order", parameter = "PlaceOrderParameter", mutable)]
fn confirm_order(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), MarketplaceError> {
     let param:PlaceOrderParameter = ctx.parameter_cursor().get()?;
     let product_name = param.product_name.clone();

     let state_mut = host.state_mut();

      // Find the product by name
    let mut product = 
    state_mut
        .product_listings
        .get_mut(&product_name)
        .ok_or(MarketplaceError::ProductNotFound)?;

    // Ensure that the product is in a valid state for confirming the escrow
    ensure!(product.state == ProductState::Escrowed, MarketplaceError::InvalidProductState);
    
    product.state = ProductState::Confirmed;
    drop(product);

     // Get the order by product_name
     if let Some(order) = host.state_mut().orders.remove_and_get(&param.product_name) {
        // Transfer funds
        host.invoke_transfer(&param.seller, order.price)?;

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