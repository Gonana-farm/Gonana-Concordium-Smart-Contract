#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract

use concordium_std::*;
use core::fmt::Debug;
use concordium_std::Amount;
use rand::prelude::*;



/// Enum representing the possible states of a product
#[derive(Debug, Serialize, SchemaType, PartialEq, Eq, Clone)]
pub enum ProductState {
    Listed,
    Released,
    Shipped,
    Cancelled,
}

// Struct to represent a product listing.
#[derive(Serialize, SchemaType,  Clone)]
pub struct ProductListing {
    pub id: u128,  // Unique identifier for the listing
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

#[derive(Serialize, SchemaType, Eq, PartialEq, PartialOrd, Clone)]
pub struct Escrow {
    pub funds: Amount,
    pub product: String,
    pub buyer: AccountAddress,
}


/// Smart contract state
#[derive( Serialize, SchemaType,  Clone)]
pub struct State {
    pub product_listings: Vec<ProductListing>,
    pub orders: Vec<Order>,
    pub escrows: Vec<Escrow>,
}

/// Error types
#[derive(Debug, PartialEq, Eq, Clone, Reject, Serialize, SchemaType)]
pub enum MarketplaceError {
    ProductNotFound,
    OrderNotFound,
    EscrowNotFound,
    InvalidCaller,
    InvalidProductState,
    InsufficientFunds,
    InvalidPrice,
    RandomGenerationError,
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
    pub product: String,
}

#[derive(Serialize, SchemaType)]
pub struct PlaceOrderParameter {
    pub product_id: u128,
    pub buyer:AccountAddress,
    pub price: Amount
}

// Init function to initialize the marketplace state
#[init(contract = "gonana_marketplace")]
fn init(_ctx: &InitContext, _state_builder: &mut StateBuilder) -> InitResult<State>{
    Ok(State { 
         product_listings: Vec::new(),
        orders: Vec::new(),
        escrows: Vec::new(),
     })
}


/// Function to list a product in the marketplace
#[receive(contract = "gonana_marketplace", name = "list_product", parameter = "ListProductParameter", mutable )]
fn list_product(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), MarketplaceError>{
    let parameter: ListProductParameter= ctx.parameter_cursor().get()?;
   
 // Generate a unique ID for the product listing using the rand crate
 let id: u128 = random();

    //check if the price is 0
    if parameter.price <= Amount::zero() {
        return Err(MarketplaceError::InvalidPrice);
    }

    // Check if the product name is empty
    if parameter.product.is_empty() {
        return Err(MarketplaceError::ParseParams);
    }

    let listing = ProductListing {
        id,
        farmer: parameter.farmer,
        product: parameter.product,
        price: parameter.price,
        state: ProductState::Listed
    };
        host.state_mut().product_listings.push(listing);
        Ok(()) 
}


/// Function to cancel or unlist a product
#[receive(contract = "gonana_marketplace", name = "cancel_product", parameter = "CancelProductParameter", mutable )]
fn cancel_product(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), MarketplaceError>{
    let parameter: CancelProductParameter= ctx.parameter_cursor().get()?;
    let product = parameter.product;


    let mut listing = host.state_mut().product_listings.iter_mut().find(|l| l.product == product)
    .ok_or(MarketplaceError::ProductNotFound)?;

     // Check if the product is in a cancellable state
     match listing.state {
        ProductState::Listed | ProductState::Released => {
            listing.state = ProductState::Cancelled;
            Ok(())
        }
        _ => Err(MarketplaceError::InvalidProductState),
    }
}


/// Function to place an order and create an escrow
#[receive(contract = "gonana_marketplace", name="place_order", parameter = "PlaceOrderParameter", mutable, payable)]
fn place_order(ctx: &ReceiveContext, host: &mut Host<State>, _amount: Amount) -> Result<(), MarketplaceError>{
    let parameter : PlaceOrderParameter = ctx.parameter_cursor().get()?;

     // Check if the product ID is valid
    // if host.state_mut().product_listings.iter().find(|listing| listing.id == parameter.product_id)
    // .is_none() {
    //     return Err(MarketplaceError::ProductNotFound);
    // }

    // Find the product by ID
    if let Some(product) = host.state_mut().product_listings.clone().iter_mut().find(|p| p.id == parameter.product_id) {
        // Check if the product is in a valid state for placing an order
        if product.state != ProductState::Listed {
            return Err(MarketplaceError::InvalidProductState);
        }
        
         // Create an order
         let order = Order {
            buyer: parameter.buyer,
            product: product.product.clone(),
            price: parameter.price,
        };

         // Deduct the order amount from the buyer's account
         host.invoke_transfer(&product.farmer, parameter.price)?;

          // Add the order to the orders list
        host.state_mut().orders.push(order);

          // Update the product state to Released
          product.state = ProductState::Released;

           // Create an escrow
        let escrow = Escrow {
            funds: parameter.price,
            product: product.product.clone(),
            buyer: parameter.buyer,
        };

        // Add the escrow to the escrows list
        host.state_mut().escrows.push(escrow);

        Ok(())
    } else {
        // Product not found
        Err(MarketplaceError::ProductNotFound)
    }

 
}



/// Function to release a product in the marketplace
#[receive(contract = "gonana_marketplace", name = "release_product", parameter = "CancelProductParameter", mutable)]
fn release_product(ctx:&ReceiveContext, host: &mut Host<State>) -> Result<(), MarketplaceError>  {
    let parameter: CancelProductParameter = ctx.parameter_cursor().get()?;

    // Check if the product name is not empty
    if parameter.product.is_empty() {
        return Err(MarketplaceError::ParseParams);
    }

     // Find the product by name
     if let Some(product) = host.state_mut().product_listings.iter_mut().find(|p| p.product == parameter.product) {
        // Check if the product is in a valid state for releasing
        if product.state != ProductState::Released {
            return Err(MarketplaceError::InvalidProductState);
        }

        // Update the product state to Shipped
        product.state = ProductState::Shipped;

        Ok(())
    } else {
        // Product not found
        Err(MarketplaceError::ProductNotFound)
    }
    
}



/// Function to confirm an escrow in the marketplace
#[receive( contract = "gonana_marketplace", name = "confirm_escrow", parameter = "CancelProductParameter", mutable )]
fn confirm_escrow(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), MarketplaceError>{
    let parameter: CancelProductParameter = ctx.parameter_cursor().get()?;

    // Check if the product name is not empty
    if parameter.product.is_empty() {
        return Err(MarketplaceError::ParseParams);
    }

      // Find the product by name
    if let Some(product) = host.state_mut().product_listings.clone().iter_mut().find(|p| p.product == parameter.product) {
        // Check if the product is in a valid state for confirming the escrow
        if product.state != ProductState::Cancelled {
            return Err(MarketplaceError::InvalidProductState);
        }

        // Find the corresponding escrow
        if let Some(escrow_index) = host.state_mut().escrows.iter().position(|e| e.product == parameter.product) {
            let escrow = host.state_mut().escrows.remove(escrow_index);

            // Transfer the funds from the escrow to the farmer's account
            host.invoke_transfer(&product.farmer, escrow.funds)?;

            Ok(())
        } else {
            // Escrow not found
            Err(MarketplaceError::EscrowNotFound)
        }
    } else {
        // Product not found
        Err(MarketplaceError::ProductNotFound)
    }
   
}



// View function to get all product listings
#[receive(contract = "gonana_marketplace", name = "view_product_listings", return_value = "Vec<ProductListing>")]
fn view_product_listings(_ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<Vec<ProductListing>> {
    Ok(host.state().product_listings.clone())
}

// View function to get all orders
#[receive(contract = "gonana_marketplace", name = "view_orders", return_value = "Vec<Order>")]
fn view_orders(_ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<Vec<Order>> {
    Ok(host.state().orders.clone())
}

// View function to get all escrows
#[receive(contract = "gonana_marketplace", name = "view_escrows", return_value = "Vec<Escrow>")]
fn view_escrows(_ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<Vec<Escrow>> {
    Ok(host.state().escrows.clone())
}