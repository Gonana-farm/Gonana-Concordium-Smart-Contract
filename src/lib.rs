#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract

use concordium_std::*;
use core::fmt::Debug;
use concordium_std::Amount;


/// Enum representing the possible states of a product
#[derive(Debug, Serialize, SchemaType, PartialEq, Eq, Clone)]
pub enum ProductState {
    Listed,
    Released,
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

#[derive(Serialize, SchemaType, Eq, PartialEq, PartialOrd, Clone)]
pub struct Escrow {
    pub funds: Amount,
    pub product: String,
    pub buyer: AccountAddress,
}


/// Smart contract state
#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S = StateApi>  {
    pub product_listings: StateMap<String,ProductListing, S>,
    pub orders: StateMap<String,Order,S>,
    pub escrows: StateMap<String,Escrow,S>,
}

impl<S: HasStateApi> State<S> {
    pub fn get_product_by_name(&self, product_name: &String) -> Result<ProductListing, MarketplaceError> {
        self.product_listings
            .get(product_name)
            .ok_or(MarketplaceError::ProductNotFound)
            .map(|product| product.clone())
    }
}

/// Error types
#[derive(Debug, PartialEq, Eq, Clone, Reject, Serialize, SchemaType)]
pub enum MarketplaceError {
    ProductNotFound,
    OrderNotFound,
    OrderAlreadyExists,
    EscrowNotFound,
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
    pub price: Amount
}

// Init function to initialize the marketplace state
#[init(contract = "gonana_marketplace")]
fn init(_ctx: &InitContext, state_builder: &mut StateBuilder) -> InitResult<State>{
    Ok(State { 
         product_listings: state_builder.new_map(),
        orders: state_builder.new_map(),
        escrows: state_builder.new_map(),
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
            ProductState::Listed | ProductState::Released => {
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

      // Find the product by name using the new method
    let mut product = host.state().get_product_by_name(&parameter.product_name)?;
       

        // Ensure that the product is in a valid state for placing an order
    ensure!(product.state == ProductState::Listed, MarketplaceError::InvalidProductState);
    
      // Ensure that the order price is greater than zero
      ensure!(parameter.price > Amount::zero(), MarketplaceError::InvalidPrice);

      // Ensure that the buyer has enough funds
     // Ensure that the buyer has enough funds
     ensure!(parameter.price <= amount, MarketplaceError::InsufficientFunds);


     // Create an order
     let order = Order {
        buyer: parameter.buyer,
        product: product.product.clone(),
        price: parameter.price,
    };

   // Deduct the order amount from the buyer's account
   host.invoke_transfer(&product.farmer, parameter.price)?;

   // Insert the order and update the product state to Released in one statement
   if host.state_mut().orders.insert(parameter.product_name.clone(), order).is_none() {
    // If the insert is successful, update the product state
    product.state = ProductState::Released;
    Ok(())
} else {
    // Handle the case when the order already exists
    Err(MarketplaceError::OrderAlreadyExists)
}

}




/// Function to confirm an escrow in the marketplace
#[receive(contract = "gonana_marketplace", name = "confirm_escrow", parameter = "CancelProductParameter", mutable)]
fn confirm_escrow(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), MarketplaceError> {
    let parameter: CancelProductParameter = ctx.parameter_cursor().get()?;
    let product_name = parameter.product_name.clone();

     // Find the product by name
     let mut product = {
        let state = host.state_mut();
        state.product_listings.remove_and_get(&product_name).ok_or(MarketplaceError::ProductNotFound)?
    };

     // Ensure that the product is in a valid state for confirming the escrow
     ensure!(product.state == ProductState::Released, MarketplaceError::InvalidProductState);

   // Remove the escrow and get its value
   if let Some(escrow) = host.state_mut().escrows.remove_and_get(&product_name) {
    // Transfer funds
    host.invoke_transfer(&product.farmer, escrow.funds)?;

    // It's important to call `Deletable::delete` on the value
    escrow.delete();
    // If the removal is successful, update the product state
    product.state = ProductState::Confirmed;

    Ok(())
} else {
    // Escrow not found
    Err(MarketplaceError::EscrowNotFound)
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

// // View function to get all escrows
#[receive(contract = "gonana_marketplace", name = "view_escrows", return_value = "Vec<Escrow>")]
fn view_escrows(_ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<Vec<Escrow>> {
    let state = host.state();
    let escrows: Vec<Escrow> = state.escrows.iter().map(|(_, escrow)| escrow.clone()).collect();
    Ok(escrows)
}