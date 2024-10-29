## Gonana Smart Contract Wallet

This smart contract implements a **CIS-5 (Concordium Interoperability Specification 5)** smart contract wallet, providing users with the ability to hold and transfer **CCD** and **CIS-2 tokens** on the Concordium blockchain. The contract is designed for secure self-custody of assets, meaning the holder of the private key associated with a public key can fully control token and CCD transactions.

- **CIS-5 Specification**: The CIS-5 specification is available [here](https://proposals.concordium.software/CIS/cis-5.html).

---

### Core Features

1. **Self-Custodial Wallet**:
   - Users control the assets tied to their assigned public key without relying on third-party custody.
   - Only the private key holder for a given public key can authorize transfers, providing self-custody security.

2. **Transfer Mechanism**:
   - Token and CCD balances can be transferred via this wallet through a signature-based approach. On-chain submission of these transactions can be done by any third party.
   - Users can offer a **service fee** within their signed message to incentivize third-party involvement in submitting their transactions on-chain.

3. **Deposit and Withdraw Functions**:
   - **Deposit**: Accepts CCD or CIS-2 tokens and associates them with a user-designated public key.
   - **Withdraw**: Allows users to withdraw CCD or CIS-2 tokens from the wallet to any native account or smart contract.

4. **Nonce System**:
   - The nonce registry tracks nonces associated with each public key, preventing replay attacks by ensuring each transaction signature is unique and incrementally updated.

5. **KYC-Free Initial Onboarding**:
   - Users can begin transacting in CCD and CIS-2 tokens without requiring a native account or KYC process. KYC is only necessary when creating a native account.

---

### State Management

#### State Fields

1. **token_balances**: 
   - Stores token balances as a mapping of `(ContractAddress, TokenId, PublicKey)` to token amounts.
2. **ccd_balances**:
   - Maps each public key to their respective CCD balance.
3. **implementors**:
   - Tracks addresses of contract implementations for additional standards.
4. **nonces_registry**:
   - Maintains nonces for each public key, allowing for sequential validation of messages and preventing replay attacks.

#### State Methods

- **`empty()`**:
  - Initializes an empty state for balances, implementors, and nonces.

---

### Core Functions

#### Initialization

**`contract_init`**:
- Initializes the contract with an empty state. No balances are set at this stage.

#### Deposit Functions

1. **`depositCcd`**:
   - **Parameters**: Accepts a public key and the amount of CCD being deposited.
   - **Behavior**: Increases the CCD balance assigned to the specified public key and logs a `DepositCcd` event.

2. **`depositCis2Tokens`**:
   - **Parameters**: Accepts token transfer parameters and sender’s contract address.
   - **Behavior**: Increases the balance of CIS-2 tokens for a given public key, contract, and token ID, logging a `DepositCis2Tokens` event.

#### Withdrawal Functions

1. **`Withdraw<T>` Structure**:
   - Defines a withdrawal operation specifying the receiver, amount, and optional data.
   
2. **`WithdrawMessage<T>` Structure**:
   - Encapsulates the signed message for withdrawal, including a nonce, expiration, service fee details, and a list of withdrawal actions.

3. **`WithdrawBatch<T>` Structure**:
   - Holds the signed message, signer's public key, and signature. Used as input for `withdrawCcd` and `withdrawCis2Tokens` functions.

#### Transfer Functions

1. **`transfer_ccd`**:
   - **Parameters**: Accepts `from` and `to` public keys, transfer amount, and a logger.
   - **Behavior**: Transfers CCD between two public keys and logs a `TransferCcd` event. Fails if insufficient funds or overflow occurs.

2. **`transfer_cis2_tokens`**:
   - **Parameters**: Similar to `transfer_ccd`, but specifies token ID and contract address.
   - **Behavior**: Transfers a CIS-2 token balance between two public keys, logging a `TransferCis2Tokens` event. Handles overflow and balance checks.

---

### Message Validation

**`validate_signature_and_increase_nonce`**:
- **Parameters**: Takes a signed message, signer’s public key, and additional security checks.
- **Functionality**: Validates the signature, nonce, and message expiration to ensure only legitimate transactions are processed. If valid, the nonce is incremented to prevent replay.

---

### Utility Functions

**`calculate_message_hash_from_bytes`**:
- Computes a message hash with a contextual string to ensure consistency and avoid message manipulation. Uses the contract's address and genesis hash as identifiers.

---

### Error Handling

The contract uses a custom error enumeration, `CustomContractError`, to handle various errors:
- **`ParseParams`**: Invalid parameter parsing.
- **`InsufficientFunds`**: Balance errors.
- **`NonceMismatch`**: Nonce issues for replay prevention.
- **`Expired`**: Expired signatures.

