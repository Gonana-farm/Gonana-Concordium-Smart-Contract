// ProductListing struct
#[derive(Debug, Clone)]
pub struct ProductListing {
    pub farmer: String,
    pub product: String,
    pub price: u64,
}



// Order struct
#[derive(Debug, Clone)]
pub struct Order {
    pub buyer: String,
    pub product: String,
    pub price: u64,
}




// ProductState enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductState {
    Listed,
    Escrowed,
    Confirmed,
    Cancelled,
}





// MarketplaceError enum
#[derive(Debug, Clone)]
pub enum MarketplaceError {
    ProductNotFound,
    OrderNotFound,
    OrderAlreadyExists,
    InvalidProductState,
    InsufficientFunds,
    InvalidPrice,
}





//  State struct to represent the smart contract state
#[derive(Debug, Clone)]
pub struct State {
    pub product_listings: Vec<ProductListing>,
    pub orders: Vec<Order>,
}









// Init function to initialize the marketplace state
#[allow(dead_code)]
pub fn init() -> State {
    State {
        product_listings: Vec::new(),
        orders: Vec::new(),
    }
}








// Function to list a product in the marketplace
#[allow(dead_code)]
pub fn list_product(state: &mut State, farmer: String, product: String, price: u64) {
    let listing = ProductListing {
        farmer,
        product: product.clone(),
        price,
    };

    state.product_listings.push(listing);
}











// Function to cancel or unlist a product
#[allow(dead_code)]
pub fn cancel_product(state: &mut State, product_name: String) {
    if let Some(listing) = state
        .product_listings
        .iter_mut()
        .find(|listing| listing.product == product_name)
    {
        // Check if the product is in a cancellable state
        match listing.state {
            ProductState::Listed | ProductState::Escrowed => {
                listing.state = ProductState::Cancelled;
            }
            _ => {
                // Handle invalid state
                println!("Invalid state for product: {}", product_name);
            }
        }
    } else {
        // Handle product not found
        println!("Product not found: {}", product_name);
    }
}














// Function to view all product listings in the marketplace
#[allow(dead_code)]
pub fn view_product_listings(state: &State) -> Vec<ProductListing> {
    state.product_listings.clone()
}

// Function to view all orders in the marketplace
#[allow(dead_code)]
pub fn view_orders(state: &State) -> Vec<Order> {
    state.orders.clone()
}


