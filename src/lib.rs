#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract
use concordium_std::*;
use core::fmt::Debug;




// Struct to represent a product listing.
#[derive(Serialize, SchemaType)]
pub struct Listing {
    product_name: String,
    price: Amount,
    seller: AccountAddress,
}

// Struct to represent an order.
#[derive(Debug, Serialize, SchemaType, PartialEq, Eq, Clone)]
pub struct Order {
    product_name: String,
    price: Amount,
    seller: AccountAddress,
    buyer: AccountAddress
}

#[derive(Debug, Serialize, SchemaType, PartialEq, Eq, Clone)]
pub struct Escrow {
    order_index: u64,
    funds: Amount,
}

// Enum to represent order status.
#[derive(Debug, PartialEq, Eq, Serialize, SchemaType)]
pub enum OrderStatus {
    Placed,
    Shipped,
    Completed,
    Cancelled,
}

#[derive(Serialize, SchemaType)]
pub struct InitParameter;

/// 'add_listing' function errors
#[derive(Debug, PartialEq, Eq, Clone, Reject, Serialize, SchemaType)]
pub enum AddListingError {
    /// Raised when an account other than the seller tries to add a listing
    NotSeller,
}


/// 'place_order' function errors
#[derive(Debug, PartialEq, Eq, Clone, Reject, Serialize, SchemaType)]
pub enum PlaceOrderError {
    /// Raised when placing an order for a non-existent listing
    ListingNotFound,
    /// Raised when a buyer tries to place an order for their own listing
    CannotBuyOwnListing,
    /// Raised when placing an order with insufficient funds
    InsufficientFunds,
}

/// 'view_listings' function errors
#[derive(Debug, PartialEq, Eq, Clone, Reject, Serialize, SchemaType)]
pub enum ViewOrdersError {
    /// An error occurred while accessing the orders.

}

/// Your smart contract errors.
#[derive(Debug, PartialEq, Eq, Reject, Serialize, SchemaType)]
pub enum Error {
    /// Failed parsing the parameter.
    #[from(ParseError)]
    ParseParams,
    /// Your error
    ProductNotFound,
    OrderNotFound,
    InvalidOrderStatus,
    InsufficientFunds,
}

/// Initialization function for the marketplace smart contract
#[init(contract = "gonana_concordium_smart_contract")]
fn init(_ctx: &InitContext, _state_builder: &mut StateBuilder) -> InitResult<State> {
    // Your code

    Ok(State {
        products: Vec::new(),
        orders: Vec::new(),
    })
}

pub type MyInputType = bool;

/// Receive function. The input parameter is the boolean variable `throw_error`.
///  If `throw_error == true`, the receive function will throw a custom error.
///  If `throw_error == false`, the receive function executes successfully.
#[receive(
    contract = "gonana_concordium_smart_contract",
    name = "receive",
    parameter = "MyInputType",
    error = "Error",
    mutable
)]
fn receive(ctx: &ReceiveContext, _host: &mut Host<State>) -> Result<(), Error> {
    // Your code

    let throw_error = ctx.parameter_cursor().get()?; // Returns Error::ParseError on failure
    if throw_error {
        Err(Error::YourError)
    } else {
        Ok(())
    }
}

/// View function that returns the content of the state.
#[receive(contract = "gonana_concordium_smart_contract", name = "view", return_value = "State")]
fn view<'b>(_ctx: &ReceiveContext, host: &'b Host<State>) -> ReceiveResult<&'b State> {
    Ok(host.state())
}
