// will contain our new enum pizza error here
use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};

use derive_more::Display;
#[derive(Debug, Display)]
pub enum MarketplaceError {
    ProductNotFound,
    OrderNotFound,
    OrderAlreadyExists,
    InvalidProductState,
    InsufficientFunds,
    InvalidPrice,
    ParseParams,
    NonceAlreadyUsed,
    WrongContract,
    TransactionSimulationError
}

impl ResponseError for MarketplaceError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            MarketplaceError::InsufficientFunds => StatusCode::FAILED_DEPENDENCY,
            MarketplaceError::InvalidPrice => StatusCode::NOT_ACCEPTABLE,
            MarketplaceError::InvalidProductState => StatusCode::FORBIDDEN,
            MarketplaceError::NonceAlreadyUsed => StatusCode::FORBIDDEN,
            MarketplaceError::OrderAlreadyExists => StatusCode::CONFLICT,
            MarketplaceError::OrderNotFound => StatusCode::NOT_FOUND,
            MarketplaceError::ParseParams => StatusCode::UNPROCESSABLE_ENTITY,
            MarketplaceError::ProductNotFound => StatusCode::NOT_FOUND,
            MarketplaceError::TransactionSimulationError => StatusCode::BAD_REQUEST,
            MarketplaceError::WrongContract => StatusCode::BAD_GATEWAY
        }
    }
}
