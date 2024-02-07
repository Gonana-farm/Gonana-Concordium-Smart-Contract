# Gonana Marketplace Smart Contract Documentation

![Concordium Logo](concordium-logo.png)

# Overview

The Gonana Marketplace Smart Contract is a decentralized application (dApp) built on the Concordium blockchain. It facilitates the buying and selling of agricultural products between users on the blockchain. This document provides a comprehensive guide on how to interact with the smart contract, including listing products, placing orders, and confirming escrow transactions.

# Table of Contents

    Getting Started
        Contract Address
        Supported Functions
    Listing a Product
        Function: list_product
        Parameters
        Example
    Placing an Order
        Function: place_order
        Parameters
        Example
    Confirming Escrow
        Function: confirm_escrow
        Parameters
        Example
    Error Handling
        Marketplace Error Codes
    Testing
        Unit Tests
        Integration Tests
    Examples
        JavaScript/TypeScript
        Command-Line Interface (CLI)

# Getting Started

# Contract Address

The Gonana Marketplace Smart Contract is deployed at the following address:

<Contract Address 7504>

# Supported Functions

## Contract Structure

The contract is structured into the following main components:

- **Enums:**

  - `ProductState`: Represents the possible states of a product, such as Listed, Escrowed, Confirmed, or Cancelled.

- **Structs:**

  - `ProductListing`: Represents the details of a listed product, including a unique product ID, amount, wallet address of the creator, hash of product parameters, merchant ID, and the product state.

  - `Order`: Represents an order containing information such as the product ID, amount, buyer's address, and buyer's ID.

  - `ListProductParameter`: Represents the parameters required to list a product, including the product ID, amount, wallet address, hash, and merchant ID.

  - `CancelProductParameter`: Represents the parameters required to cancel or unlist a product, including the product ID and merchant ID.

- **Errors:**

  - `MarketplaceError`: Enumerates possible errors that can occur during contract execution, such as ProductNotFound, OrderNotFound, OrderAlreadyExists, InvalidProductState, InsufficientFunds, InvalidPrice, and more.

- **Events:**

  - `Event`: Enumerates events logged by the contract, including the NonceEvent.

- **State:**
  - `State`: Represents the smart contract state, including product listings, orders, and a registry of nonces.

## Entrypoints

### `list_product`

- **Description:** List a product in the marketplace.
- **Parameters:** `ListProductParameter`
- **Mutability:** Mutable
- Sample Parameter:

```json
{
  "product_id": "12345",
  "amount": 500,
  "wallet": "account_address_here",
  "hash": "abcdef1234567890",
  "merchant_id": "merchant_123"
}
```

### `cancel_product`

- **Description:** Cancel or unlist a product.
- **Parameters:** `CancelProductParameter`
- **Mutability:** Mutable
- Sample Parameter:

```json
{
  "product_id": "12345",
  "merchant_id": "merchant_123"
}
```

### `place_order`

- **Description:** Place an order for a product.
- **Parameters:** `PlaceOrderParameter`
- **Mutability:** Mutable, Payable
- Sample Parameter:

```json
{
  "product_id": "12345",
  "buyer_address": "buyer_address_here",
  "buyer_id": "buyer_456",
  "amount": 600
}
```

### `confirm_order`

- **Description:** Confirm an order and release funds from escrow.
- **Parameters:** `PlaceOrderParameter`
- **Mutability:** Mutable
- Sample Parameter:

```json
{
  "product_id": "12345",
  "buyer_address": "buyer_address_here",
  "buyer_id": "buyer_456",
  "amount": 600
}
```

### `view_product_listings`

- **Description:** View all product listings.
- **Return Type:** `Vec<ProductListing>`
- Sample Return Type:

```json
[
  {
    "product_id": "12345",
    "amount": 500,
    "wallet": "account_address_here",
    "hash": "abcdef1234567890",
    "merchant_id": "merchant_123",
    "state": "Listed"
  }
]
```

### `view_orders`

- **Description:** View all placed orders.
- **Return Type:** `Vec<Order>`
- Sample Return Type:

```json
[
  {
    "product_id": "12345",
    "amount": 600,
    "buyer_address": "buyer_address_here",
    "buyer_id": "buyer_456"
  }
]
```

### `permit`

- **Description:** Permit function for transaction authorization.
- **Parameters:** `PermitParam`
- **Mutability:** Mutable
- Sample Parameter:

```json
{
  "signature": {
    "Single": "0101010101010101010101010101010101010101010101010101010101010101"
  },
  "signer": "signer_address_here",
  "message": {
    "contract_address": "contract_address_here",
    "nonce": 1,
    "timestamp": 1234567890,
    "entry_point": "list_product",
    "payload": [0, 1, 2, 3]
  }
}
```

## Initialization

### `init`

- **Description:** Initializes the marketplace smart contract state.
- **Parameters:** None
- **Return Type:** `State`

## Helper Functions

### `publicKeyOf`

- **Description:** Get public keys of accounts.
- **Parameters:** `VecOfAccountAddresses`
- **Return Type:** `PublicKeyOfQueryResponse`
- Sample Parameter:

```json
{
  "queries": ["account_address_1", "account_address_2"]
}
```

### `nonceOf`

- **Description:** Get nonces of accounts.
- **Parameters:** `VecOfAccountAddresses`
- **Return Type:** `NonceOfQueryResponse`
- Sample Parameter:

```json
{
  "queries": ["account_address_1", "account_address_2"]
}
```

### `viewMessageHash`

- **Description:** Calculate the message hash for permit function.
- **Parameters:** `PermitParamPartial`
- **Return Type:** `[u8;32]`
- Sample Parameter:

```json
{
  "signature": {
    "Single": "0101010101010101010101010101010101010101010101010101010101010101"
  },
  "signer": "signer_address_here"
}
```

## Events

- `NonceEvent`: Logged when the `permit` function is invoked, tracking the nonce used by the signer of the `PermitMessage`.

## Notes

- The contract supports transaction authorization using the `permit` function.

- Nonce events are logged for each invocation of the `permit` function.

- The contract provides view functions to retrieve information about product listings and placed orders.

## Usage

To use the Marketplace Smart Contract, follow the specified entrypoints for listing products, placing orders, confirming orders, and viewing information.

Make sure to handle errors appropriately, and use the provided view functions to query the contract state.

For transaction authorization, use the `permit` function with the required parameters.

## GonaToken Smart Contract Documentation

## Overview

The GonaToken smart contract is designed to manage the token functionality within the Gonana Marketplace ecosystem. It provides essential features for token transfers, approvals, and balance management.

## Endpoints

### `Approve Function`

- **Description:** Grants permission to another account or contract to spend tokens on behalf of the owner.
- **Parameters:** `spender: Address of the account or contract to be granted spending approval.
amount: Amount of tokens to approve for spending.
token_id: TokenIdUnit`
- Sample Parameter:

```json
{
  "spender": "AccountAddress or ContractAddresss",
  "amount": 100,
  "token_id": TokenIdUnit,
}
```

### `Transfer Function`

- **Description:** Transfers tokens from the sender's account to the specified recipient.
- **Parameters:** `to: Address of the recipient account.
amount: Amount of tokens to transfer.
token_id: TokenIdUnit`
- Sample Parameter:

```json
{
    "to": "AcccountAddress",
    "amount": 50,
    "token_id": TokenIdUnit,
}

```

### `Transfer Function`

- **Description:** Transfers tokens from the sender's account to the specified recipient.
- **Parameters:** `to: Address of the recipient account.
amount: Amount of tokens to transfer.
token_id: TokenIdUnit`
- Sample Parameter:

```json
{
    "to": "AcccountAddress",
    "amount": 50,
    "token_id": TokenIdUnit,
}
```

## Note

    The GonanaToken smart contract operates within the Gonana Marketplace ecosystem, facilitating secure and transparent token transactions.
    Users can approve spending, transfer tokens, check balances, and execute other token-related operations using the provided endpoints.
    Detailed implementation and internal logic are abstracted in this documentation for simplicity. Users can refer to the contract source code for deeper insights into functionality and implementation details.

## License

This smart contract is provided under [License Name]. Refer to the LICENSE file for details.

For more information about Concordium, visit [Concordium](https://developer.concordium.software/en/mainnet/).
