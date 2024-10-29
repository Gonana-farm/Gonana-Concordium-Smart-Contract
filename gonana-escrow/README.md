# Gonana Escrow Smart Contract

This Concordium V1 smart contract implements an escrow service. It enables secure payments between a `payer` and a `receiver`, who are identified by their public keys. The smart contract provides an escrow-like mechanism, where funds are held temporarily and only released when certain conditions are met. This functionality is facilitated by tracking the state of payments with an `Invoice` structure.

## Overview of Key Structures

### `Invoice`
The `Invoice` structure represents the details of each payment transaction in the escrow.

- **Fields**:
  - `id` (`String`): Offchain identifier for the transaction.
  - `amount` (`Amount`): The monetary amount associated with the transaction.
  - `payer` (`PublicKeyEd25519`): Public key of the user making the payment.
  - `receiver` (`PublicKeyEd25519`): Public key of the payment recipient.
  - `timestamp` (`Timestamp`): The time at which the transaction was created.
  - `fufilled` (`bool`): Indicates if the transaction has been fulfilled by the smart contract.

- **Functions**:
  - `new`: A constructor for `Invoice` that takes in the transaction details and sets the `timestamp` to the current time and `fufilled` status to `false`.

### `PayParam`
The `PayParam` structure encapsulates parameters for the `pay` function:

- **Fields**:
  - `id` (`String`): Offchain identifier for the transaction.
  - `payer` (`PublicKeyEd25519`): Public key of the payer.
  - `receiver` (`PublicKeyEd25519`): Public key of the receiver.

### `State`
This is the main structure holding the contract’s state.

- **Fields**:
  - `payments` (`StateMap<String, Invoice>`): A mapping from payment `id`s to `Invoice` objects, representing the payments held in escrow.
  - `admin` (`Address`): The administrator's address, which can fulfill the contract.
  - `smart_wallet` (`ContractAddress`): The smart wallet contract address where funds will be transferred upon fulfillment.

- **Functions**:
  - `fufil`: Marks a payment as fulfilled, preventing further actions on that payment.

## Error Types

The `Error` enum defines potential errors in this smart contract:

- `ParseParams`: Occurs when there is an error parsing parameters.
- `CallerNotAdminError`: The caller does not have administrator privileges.
- `InvoiceNotFoundError`: The specified invoice does not exist.
- `InvoiceAlreadyPaid`: The specified invoice has already been fulfilled.
- `InvokeContractError`: An error occurred while invoking another contract.

## Contract Functions

### `init`
Initializes the escrow smart contract, setting the smart wallet address and registering the contract’s administrator.

- **Parameters**:
  - `smart_wallet` (`ContractAddress`): The address of the smart wallet.
  
- **Functionality**:
  - Sets the `payments` map, `admin` address, and `smart_wallet` address.

### `pay`
Accepts a payment to be held in escrow. This function is used by a user to simulate a payment to the escrow service.

- **Parameters**:
  - `PayParam`: Includes the `id`, `payer`, and `receiver` details.
  
- **Functionality**:
  - Creates a new `Invoice` with the specified `id` and payment details.
  - Inserts the `Invoice` into the `payments` map if it doesn't already exist.

### `withdraw`
Allows the administrator to release funds from escrow to the specified `receiver`.

- **Parameters**:
  - `id` (`String`): The transaction ID of the invoice to fulfill.
  
- **Functionality**:
  - Ensures the caller is the administrator.
  - Validates that the invoice exists and has not been previously fulfilled.
  - Marks the invoice as fulfilled.
  - Invokes the `depositCcd` entry point on the specified `smart_wallet` contract to transfer funds to the receiver.

---