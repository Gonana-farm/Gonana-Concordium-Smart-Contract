//! Upgradable smart contract (Concordium's canonical Gona
//! implementation following the CIS-2 standard)
//!
//! # Description
//! The token in this contract aims to the mirror a reflective token algorithim . This smart contract can be
//! paused/unpaused and upgraded by an admin address.
//! Note: The word 'address' refers to either an account address or a
//! contract address.
//!
//! As follows from the CIS-2 specification, the contract has a `transfer`
//! function for transferring an amount of a specific token type from one
//! address to another address. An address can enable and disable one or more
//! addresses as operators with the `updateOperator` function. An operator of
//! some token owner address is allowed to transfer or unwrap any tokens of the
//! owner.
//!

//!
//! The admin address can pause/unpause the protocol, set implementors, transfer
//! the admin address to a new address, and update the metadata URL.
#![cfg_attr(not(feature = "std"), no_std)]

use core::cmp::Ordering;

use concordium_cis2::{Cis2Event, *};
use concordium_std::*;
use primitive_types::*;




/// The id of the Gona token in this contract.
pub const TOKEN_ID_Gona: ContractTokenId = TokenIdUnit();

/// Tag for the NewAdmin event.
pub const NEW_ADMIN_EVENT_TAG: u8 = 0;

/// List of supported standards by this contract address.
pub const SUPPORTS_STANDARDS: [StandardIdentifier<'static>; 2] =
    [CIS0_STANDARD_IDENTIFIER, CIS2_STANDARD_IDENTIFIER];

/// Sha256 digest
pub type Sha256 = [u8; 32];

// Types

/// Contract token ID type.
/// Since this contract will only ever contain this one token type, we use the
/// smallest possible token ID.
pub type ContractTokenId = TokenIdUnit;

/// Contract token amount type.
/// Since this contract is wrapping the CCD and the CCD can be represented as a
/// u64, we can specialize the token amount to u64 reducing module size and cost
/// of arithmetics.
pub type ContractTokenAmount = TokenAmountU256;



/// Maximum number of tokens that can be minted
pub const MAX: U256 = U256::max_value();

/// decimals used on the token: 1e9 
pub const DECIMALS:u64 =  10_u64.pow(9);

/// Total supply of the token: 50000000 * 1e9
pub const TOTAL_SUPPLY:u64 = 50000000 * DECIMALS; //tTotal


pub const TAX_FEE:u64 = 5;

pub const PREV_TAX_FEE:u64 = TAX_FEE;

pub const BURN_FEE :u64 = 5;

pub const PREV_BURN_FEE:u64 = BURN_FEE;

pub const MAX_TOKENS_TO_TRANSFER:u64 = 5000 * DECIMALS;



/// The state tracked for each address.
#[derive(Serial, DeserialWithState, Deletable)]
#[concordium(state_parameter = "S")]
pub struct AddressState<S = StateApi> {
    /// The number of tokens owned by this address.
    pub balance:   ContractTokenAmount,
    /// The address which are currently enabled as operators for this token and
    /// this address.
    pub operators: StateSet<Address, S>,
}

/// The contract state,
#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
struct State<S: HasStateApi = StateApi> {
    /// The admin address can upgrade the contract, pause and unpause the
    /// contract, transfer the admin address to a new address, set
    /// implementors, and update the metadata URL in the contract.
    admin:        Address,
    /// Contract is paused if `paused = true` and unpaused if `paused = false`.
    paused:       bool,
    /// Map specifying the `AddressState` (balance and operators) for every
    /// address.
    t_token:        StateMap<Address, AddressState<S>, S>,

    /// Amount of Reflective tokens owned
    r_token:        StateMap<Address, AddressState<S>, S>,

    /// Addresses that are excluded from paying the tax.
    is_excluded_from_tax: StateMap<Address, bool, S>,

    /// Addresses that are excluded from receiving the tax rewards.
    is_excluded_from_reward: StateMap<Address, bool, S>,

    /// array of addresses that are excluded.
    excluded: StateSet<Address, S>,

    black_list: StateMap<Address, bool, S>,
    
    roles:    StateMap<Address, bool, S>,


    /// Map with contract addresses providing implementations of additional
    /// standards.
    implementors: StateMap<StandardIdentifierOwned, Vec<ContractAddress>, S>,
    /// The MetadataUrl of the token.
    /// `StateBox` allows for lazy loading data. This is helpful
    /// in the situations when one wants to do a partial update not touching
    /// this field, which can be large.
    metadata_url: StateBox<MetadataUrl, S>,
    /// The total supply of the token.
    total_supply: ContractTokenAmount,

    total_fee: ContractTokenAmount,
    /// The number of reflective tokens that will ever be minted.
    total_reflective_tokens: ContractTokenAmount,

    minted: bool,

    tax_fee: u8,

    previous_tax_fee: u8,

    previous_burn_fee: u8,

    burn_fee: u8,
}

/// The parameter type for the contract function `unwrap`.
/// Takes an amount of tokens and unwraps the CCD and sends it to a receiver.
#[derive(Serialize, SchemaType)]
pub struct UnwrapParams {
    /// The amount of tokens to unwrap.
    pub amount:   ContractTokenAmount,
    /// The owner of the tokens.
    pub owner:    Address,
    /// The address to receive these unwrapped CCD.
    pub receiver: Receiver,
    /// If the `Receiver` is a contract the unwrapped CCD together with these
    /// additional data bytes are sent to the function entrypoint specified in
    /// the `Receiver`.
    pub data:     AdditionalData,
}
#[derive(Serialize, SchemaType)]
pub struct BurnParams {
    pub token_id : ContractTokenId,
    pub amount: ContractTokenAmount, 
}
/// The parameter type for the contract function `wrap`.
/// It includes a receiver for receiving the wrapped CCD tokens.
#[derive(Serialize, SchemaType, Debug)]
pub struct WrapParams {
    /// The address to receive these tokens.
    /// If the receiver is the sender of the message wrapping the tokens, it
    /// will not log a transfer event.
    pub to:   Receiver,
    /// Some additional data bytes are used in the `OnReceivingCis2` hook. Only
    /// if the `Receiver` is a contract and the `Receiver` is not
    /// the invoker of the wrap function the receive hook function is
    /// executed. The `OnReceivingCis2` hook invokes the function entrypoint
    /// specified in the `Receiver` with these additional data bytes as
    /// part of the input parameters. This action allows the receiving smart
    /// contract to react to the credited Gona amount.
    pub data: AdditionalData,
}



#[derive(Serialize, SchemaType, Debug)]
pub struct MintParams {
    pub to:   Address,
    //pub amount: ContractTokenAmount,
}


/// The parameter type for the contract function `setImplementors`.
/// Takes a standard identifier and list of contract addresses providing
/// implementations of this standard.
#[derive(Debug, Serialize, SchemaType, PartialEq, Eq)]
pub struct SetImplementorsParams {
    /// The identifier for the standard.
    pub id:           StandardIdentifierOwned,
    /// The addresses of the implementors of the standard.
    pub implementors: Vec<ContractAddress>,
}

/// The parameter type for the contract function `upgrade`.
/// Takes the new module and optionally an entrypoint to call in the new module
/// after triggering the upgrade. The upgrade is reverted if the entrypoint
/// fails. This is useful for doing migration in the same transaction triggering
/// the upgrade.
#[derive(Debug, Serialize, SchemaType, PartialEq, Eq)]
pub struct UpgradeParams {
    /// The new module reference.
    pub module:  ModuleReference,
    /// Optional entrypoint to call in the new module after upgrade.
    pub migrate: Option<(OwnedEntrypointName, OwnedParameter)>,
}

/// The return type for the contract function `view`.
#[derive(Serialize, SchemaType, PartialEq, Eq, Debug)]
pub struct ReturnBasicState {
    /// The admin address can upgrade the contract, pause and unpause the
    /// contract, transfer the admin address to a new address, set
    /// implementors, and update the metadata URL in the contract.
    pub admin:        Address,
    /// Contract is paused if `paused = true` and unpaused if `paused = false`.
    pub paused:       bool,
    /// The metadata URL of the token.
    pub metadata_url: MetadataUrl,
}

/// The parameter type for the contract function `setMetadataUrl`.
#[derive(Serialize, SchemaType, Clone, PartialEq, Eq, Debug)]
pub struct SetMetadataUrlParams {
    /// The URL following the specification RFC1738.
    pub url:  String,
    /// The hash of the document stored at the above URL.
    pub hash: Option<Sha256>,
}

/// The parameter type for the contract function `setPaused`.
#[derive(Serialize, SchemaType, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct SetPausedParams {
    /// Contract is paused if `paused = true` and unpaused if `paused = false`.
    pub paused: bool,
}

/// A NewAdminEvent introduced by this smart contract.
#[derive(Serialize, SchemaType, PartialEq, Eq, Debug)]
#[repr(transparent)]
#[concordium(transparent)]
pub struct NewAdminEvent {
    /// New admin address.
    pub new_admin: Address,
}

/// Tagged events to be serialized for the event log.
#[derive(SchemaType, Serialize, PartialEq, Eq, Debug)]
#[concordium(repr(u8))]
pub enum GonaEvent {
    NewAdmin {
        new_admin: NewAdminEvent,
    },
  
    #[concordium(forward = cis2_events)]
    Cis2Event(Cis2Event<ContractTokenId, ContractTokenAmount>),
}

/// The different errors the contract can produce.
#[derive(Serialize, Debug, PartialEq, Eq, Reject, SchemaType)]
pub enum CustomContractError {
    /// Failed parsing the parameter.
    #[from(ParseError)]
    ParseParams,
    /// Failed logging: Log is full.
    LogFull,
    /// Failed logging: Log is malformed.
    LogMalformed,
    /// Contract is paused.
    ContractPaused,
    /// Failed to invoke a contract.
    InvokeContractError,
    /// Failed to invoke a transfer.
    InvokeTransferError,
    /// Upgrade failed because the new module does not exist.
    FailedUpgradeMissingModule,
    /// Upgrade failed because the new module does not contain a contract with a
    /// matching name.
    FailedUpgradeMissingContract,
    /// Upgrade failed because the smart contract version of the module is not
    /// supported.
    FailedUpgradeUnsupportedModuleVersion,
    CannotSendToSelf,
    CannotSendToAddressZero,
    CannotTransferMoreThanMax,
    TotalSupplyHasAlreadyBeenMinted,
    UserIsAlreadyExcluded,

}

pub type ContractError = Cis2Error<CustomContractError>;

pub type ContractResult<A> = Result<A, ContractError>;

/// Mapping the logging errors to ContractError.
impl From<LogError> for CustomContractError {
    fn from(le: LogError) -> Self {
        match le {
            LogError::Full => Self::LogFull,
            LogError::Malformed => Self::LogMalformed,
        }
    }
}

/// Mapping errors related to contract invocations to CustomContractError.
impl<T> From<CallContractError<T>> for CustomContractError {
    fn from(_cce: CallContractError<T>) -> Self { Self::InvokeContractError }
}

/// Mapping errors related to contract invocations to CustomContractError.
impl From<TransferError> for CustomContractError {
    fn from(_te: TransferError) -> Self { Self::InvokeTransferError }
}

/// Mapping errors related to contract upgrades to CustomContractError.
impl From<UpgradeError> for CustomContractError {
    #[inline(always)]
    fn from(ue: UpgradeError) -> Self {
        match ue {
            UpgradeError::MissingModule => Self::FailedUpgradeMissingModule,
            UpgradeError::MissingContract => Self::FailedUpgradeMissingContract,
            UpgradeError::UnsupportedModuleVersion => Self::FailedUpgradeUnsupportedModuleVersion,
        }
    }
}

/// Mapping CustomContractError to ContractError
impl From<CustomContractError> for ContractError {
    fn from(c: CustomContractError) -> Self { Cis2Error::Custom(c) }
}

impl State {
    /// Creates a new state with no one owning any tokens by default.
    fn new(state_builder: &mut StateBuilder, admin: Address, metadata_url: MetadataUrl) -> Self {

        let other = MAX.div_mod(U256::from(TOTAL_SUPPLY));
        let r_total = MAX.checked_sub(other.1).unwrap();
        
        State {
            admin,
            paused: false,
            t_token: state_builder.new_map(),
            r_token: state_builder.new_map(),
            is_excluded_from_reward: state_builder.new_map(),
            is_excluded_from_tax: state_builder.new_map(),
            implementors: state_builder.new_map(),
            metadata_url: state_builder.new_box(metadata_url),
            excluded: state_builder.new_set(),
            black_list : state_builder.new_map(),
            roles : state_builder.new_map(),
            total_supply: TokenAmountU256(U256::from(TOTAL_SUPPLY)),
            total_fee: TokenAmountU256(U256::zero()),
            total_reflective_tokens: TokenAmountU256(U256::from(r_total)),
            minted: false,
            tax_fee: 5,
            previous_tax_fee: 5,
            burn_fee: 5,
            previous_burn_fee: 5,


        }
    }

    /// Get the current balance of a given token id for a given address.
    /// Results in an error if the token id does not exist in the state.
    fn balance(
        &self,
        token_id: &ContractTokenId,
        address: &Address,
    ) -> ContractResult<ContractTokenAmount> {
        ensure_eq!(token_id, &TOKEN_ID_Gona, ContractError::InvalidTokenId);
        // we assume that nobody is in the is_excluded mapping at contract initiatlisation,
        // entering into the is_excluded_from_reward mapping means you opted to be excluded from rewards
        // which means that `is_excluded_from_tax.is_some()` will always be true while being false for those that didnt opt in.
        let is_exclued = &self.is_excluded_from_reward.get(address).is_some();
        if *is_exclued {
            Ok(self.t_token.get(address)
            .map(|s| s.balance)
            .unwrap_or_else(|| TokenAmountU256(U256::zero())))
        } else {
            let token = self.r_token.get(address)
                .map(|s| s.balance)
                .unwrap_or_else(|| TokenAmountU256(U256::zero()));
            let r_amount = token.0;
            let reflection = self.token_from_reflection(r_amount)?;
            Ok(TokenAmountU256::from(reflection))
        }      
       
    }

    /// Check if an address is an operator of a specific owner address.
    fn is_operator(&self, address: &Address, owner: &Address) -> bool {
        self.t_token
            .get(owner)
            .map(|address_state| address_state.operators.contains(address))
            .unwrap_or(false)
    }

    /// Update the state with a transfer.
    /// Results in an error if the token id does not exist in the state or if
    /// the from address has insufficient tokens to do the transfer.
    fn transfer(
        &mut self,
        token_id: &ContractTokenId,
        amount: ContractTokenAmount,
        from: &Address,
        to: &Address,
        state_builder: &mut StateBuilder,
    ) -> ContractResult<U256> {
        ensure_eq!(token_id, &TOKEN_ID_Gona, ContractError::InvalidTokenId);
        
        // cannot transfer token to self
        ensure!(to != from, ContractError::Custom(CustomContractError::CannotSendToSelf));
        
        // ensure transfer is not the the zero address
        ensure!(to.cmp(&Address::from(AccountAddress([0; 32]))) != Ordering::Equal, 
            ContractError::Custom(CustomContractError::CannotSendToAddressZero));
       
        // if amount is zero just return okay
        if amount == TokenAmountU256(U256::zero()) {
            return Ok(U256::zero());
        }

        // cannot transfer more than max tokens
        // if from.cmp(&self.admin) != Ordering::Equal && from.cmp(&self.admin) != Ordering::Equal {
        //     ensure!(amount.0.lt(&U256::from(MAX_TOKENS_TO_TRANSFER)),
        //         ContractError::Custom(CustomContractError::CannotTransferMoreThanMax));
        // }

         // cannot transfer more than max tokens
         if self.roles.get(&from).is_none() && self.roles.get(&to).is_none() {
            ensure!(amount.0.lt(&U256::from(MAX_TOKENS_TO_TRANSFER)),
                ContractError::Custom(CustomContractError::CannotTransferMoreThanMax));
        }
        
         //indicates if fee should be deducted from transfer
        let mut take_fee = true;

        if self.is_excluded_from_tax.get(&to).is_some() ||
            self.is_excluded_from_tax.get(&from).is_some() {
                take_fee = false;
            }

        // transfer reflective token    
        let transfer_amount = self.token_transfer(amount, from, to,take_fee, state_builder)?;
            
        // {
        //     let mut from_state = 
        //         self.t_token.get_mut(from).ok_or(ContractError::InsufficientFunds)?;
        //     ensure!(from_state.balance >= amount, ContractError::InsufficientFunds);
        //     from_state.balance -= amount;
        // }
        // let mut to_state = 
        //     self.t_token.entry(*to).or_insert_with(|| AddressState {
        //     balance:   TokenAmountU256(U256::zero()),
        //     operators: state_builder.new_set(),
        // });
        // to_state.balance += amount;

        Ok(transfer_amount)
    }

    fn token_transfer(
        &mut self,
        amount: ContractTokenAmount,
        from: &Address,
        to: &Address,
        take_fee: bool,
        state_builder: &mut StateBuilder,
    ) -> ContractResult<U256>
    {
        if !take_fee {self.remove_all_fee();}

        let transfer_amount;

        // if sender is excluded from reward and receiver is not excluded from reward
        if self.is_excluded_from_reward.get(from).is_some() &&
            !self.is_excluded_from_reward.get(to).is_some() {
                transfer_amount = self.transfer_from_excluded(amount, from, to, state_builder)?;
        
        // if sender is not excluded from reward and receiver is excluded from reward
        } else if !self.is_excluded_from_reward.get(from).is_some() &&
            self.is_excluded_from_reward.get(to).is_some(){
                transfer_amount = self.transfer_to_excluded(amount, from, to, state_builder)?;
       
        // if both are excluded from the rewards
        } else if self.is_excluded_from_reward.get(from).is_some() &&
            self.is_excluded_from_reward.get(to).is_some(){
                transfer_amount = self.transfer_both_excluded(amount, from, to, state_builder)?;
        
        // both are not excluded from rewards
        } else {
            transfer_amount = self.transfer_standard(amount, from, to, state_builder)?;
        }

        if !take_fee {self.restore_all_fee();}

        Ok(transfer_amount)

    }



    /// if sender is excluded from reward and receiver is not excluded from reward
    fn transfer_from_excluded(
        &mut self,
        amount: ContractTokenAmount,
        from: &Address,
        to: &Address,
        state_builder: &mut StateBuilder,
    ) -> ContractResult<U256>
    {
        let(r_amount, r_transfer_amount, r_fee,
            t_transfer_amount, t_fee, t_burn) = self.get_values(amount);

         // burn the other token       
         self.burn_liquidity(t_burn, state_builder);
        
         // reflect the fee
         self.reflect_fee(r_fee, &t_fee);

        {
            let mut from_state = 
                self.t_token.get_mut(from).ok_or(ContractError::InsufficientFunds)?;
            ensure!(from_state.balance >= TokenAmountU256::from(amount), ContractError::InsufficientFunds);
            from_state.balance -= TokenAmountU256::from(amount);
        }

        {
            let mut from_state = 
                self.r_token.get_mut(from).ok_or(ContractError::InsufficientFunds)?;
            ensure!(from_state.balance >= TokenAmountU256::from(r_amount), ContractError::InsufficientFunds);
            from_state.balance -= TokenAmountU256::from(r_amount);
        }

        let mut to_state = 
            self.r_token.entry(*to).or_insert_with(|| AddressState {
            balance:   TokenAmountU256(U256::zero()),
            operators: state_builder.new_set(),
        });
        to_state.balance += TokenAmountU256::from(r_transfer_amount);
       
        Ok(t_transfer_amount)

    }

    /// if sender is not excluded from reward and receiver is excluded from reward
    fn transfer_to_excluded(
        &mut self,
        amount: ContractTokenAmount,
        from: &Address,
        to: &Address,
        state_builder: &mut StateBuilder,
    ) -> ContractResult<U256>
    {
        let(r_amount, r_transfer_amount, r_fee,
            t_transfer_amount, t_fee, t_burn) = self.get_values(amount);

        // burn the other token       
        self.burn_liquidity(t_burn, state_builder);
        
         // reflect the fee
        self.reflect_fee(r_fee, &t_fee);

        {
            let mut from_state = 
                self.r_token.get_mut(from).ok_or(ContractError::InsufficientFunds)?;
            ensure!(from_state.balance >= TokenAmountU256::from(r_amount), ContractError::InsufficientFunds);
            from_state.balance -= TokenAmountU256::from(r_amount);
        }

        let mut to_state = 
            self.t_token.entry(*to).or_insert_with(|| AddressState {
            balance:   TokenAmountU256(U256::zero()),
            operators: state_builder.new_set(),
        });
        to_state.balance += TokenAmountU256::from(t_transfer_amount);

        let mut to_state = 
            self.r_token.entry(*to).or_insert_with(|| AddressState {
            balance:   TokenAmountU256(U256::zero()),
            operators: state_builder.new_set(),
        });
        to_state.balance += TokenAmountU256::from(r_transfer_amount);

        Ok(t_transfer_amount)

    }



    fn transfer_both_excluded(
        &mut self,
        amount: ContractTokenAmount,
        from: &Address,
        to: &Address,
        state_builder: &mut StateBuilder,
    ) -> ContractResult<U256>
    {
        let(r_amount, r_transfer_amount, r_fee,
            t_transfer_amount, t_fee, t_burn) = self.get_values(amount);
        
       
        // burn the other token       
        self.burn_liquidity(t_burn, state_builder);
        
        // reflect the fee
        self.reflect_fee(r_fee, &t_fee);

       
        {
            let mut from_state = 
                self.t_token.get_mut(from).ok_or(ContractError::InsufficientFunds)?;
            ensure!(from_state.balance >= TokenAmountU256::from(amount), ContractError::InsufficientFunds);
            from_state.balance -= TokenAmountU256::from(amount);
        }

        {
            let mut from_state = 
                self.r_token.get_mut(from).ok_or(ContractError::InsufficientFunds)?;
            ensure!(from_state.balance >= TokenAmountU256::from(r_amount), ContractError::InsufficientFunds);
            from_state.balance -= TokenAmountU256::from(r_amount);
        }

        let mut to_state = 
            self.t_token.entry(*to).or_insert_with(|| AddressState {
            balance:   TokenAmountU256(U256::zero()),
            operators: state_builder.new_set(),
        });
        to_state.balance += TokenAmountU256::from(t_transfer_amount);

        let mut to_state = 
            self.r_token.entry(*to).or_insert_with(|| AddressState {
            balance:   TokenAmountU256(U256::zero()),
            operators: state_builder.new_set(),
        });
        to_state.balance += TokenAmountU256::from(r_transfer_amount);


        Ok(t_transfer_amount)

    }



    fn transfer_standard(
        &mut self,
        amount: ContractTokenAmount,
        from: &Address,
        to: &Address,
        state_builder: &mut StateBuilder,
    ) -> ContractResult<U256>
    {
        let(r_amount, r_transfer_amount, r_fee,
                t_transfer_amount, t_fee, t_burn) = self.get_values(amount);
        
        // burn the other token       
        self.burn_liquidity(t_burn, state_builder);
        
        // reflect the fee
        self.reflect_fee(r_fee, &t_fee);
        
        {
            let mut from_state = 
                self.r_token.get_mut(from).ok_or(ContractError::InsufficientFunds)?;
            ensure!(from_state.balance >= TokenAmountU256::from(r_amount), ContractError::InsufficientFunds);
            from_state.balance -= TokenAmountU256::from(r_amount);
        }
        let mut to_state = 
            self.r_token.entry(*to).or_insert_with(|| AddressState {
            balance:   TokenAmountU256(U256::zero()),
            operators: state_builder.new_set(),
        });
        to_state.balance += TokenAmountU256::from(r_transfer_amount);

        Ok(t_transfer_amount)
    }


    fn get_values(&self, amount: ContractTokenAmount) -> (U256, U256, U256, U256, U256, U256) {
        // calculate transaction fee + liquidity fee then subtract it from the transaction amount 
        let (t_transfer_amount, t_fee, t_burn) = self.get_t_values(amount);
        // this function takes the tAmount, tFee, tLiquidity and the current rate
        let rate =  self.current_rate();
        let (r_amount, r_transfer_amount, r_fee) = self.get_r_values(amount,&t_fee,&t_burn,&rate);
        (r_amount.into(), r_transfer_amount.into(), r_fee.into(), t_transfer_amount.into(), t_fee.into(), t_burn.to_owned())

    }

    fn get_t_values(&self,amount: ContractTokenAmount) -> (U256, U256, U256) {
        let t_fee = &self.calculate_tx_fee(amount);
        let t_burn = &self.calculate_burn_fee(amount);
        let t_transfer_amount = amount.0 - t_fee - t_burn;
        (t_transfer_amount, t_fee.to_owned(), t_burn.to_owned())

    }

    fn get_r_values(&self,amount: ContractTokenAmount,t_fee:&U256, burn_fee:&U256,rate:&U256) -> (U256, U256, U256) {
        let r_amount = amount.0 * rate;
        let r_fee = t_fee *rate;
        let r_burn_fee = burn_fee * rate;
        let r_transfer_amount = r_amount - r_fee - r_burn_fee;
        (r_amount,r_transfer_amount,r_fee)
    }

    fn burn_liquidity(&mut self, t_burn: U256,state_builder: &mut StateBuilder) {
        let rate = self.current_rate();
        let r_burn_fee = t_burn * rate;
        let mut to_state = self.r_token.entry(Address::Account(AccountAddress([9; 32]))).or_insert_with(|| AddressState {
            balance:   TokenAmountU256(U256::zero()),
            operators: state_builder.new_set(),
        });
        to_state.balance += TokenAmountU256::from(r_burn_fee);
    }

    fn current_rate(&self) -> U256 {
        let (r_supply, t_supply) = self.get_current_supply();
        r_supply.div_mod(t_supply).0
    }

    fn calculate_tx_fee(&self, amount: ContractTokenAmount)-> U256 {
        (amount.0 * U256::from(self.tax_fee)) / U256::from(100)
    }

    fn calculate_burn_fee(&self, amount: ContractTokenAmount)-> U256 {
        (amount.0 * U256::from(self.burn_fee)) / U256::from(100)
    }


    fn reflect_fee(&mut self, r_fee: U256, t_fee:&U256){
        let prev_amount = self.total_reflective_tokens.0;
        let new_amount = prev_amount - r_fee;
        self.total_reflective_tokens = TokenAmountU256(new_amount);

        let prev_amount = self.total_fee.0;
        let new_amount = prev_amount + t_fee;
        self.total_fee = TokenAmountU256(new_amount);

    }

    /// remove all the tax fee from the transfer
    fn remove_all_fee(&mut self){
        
        if self.tax_fee == 0 && self.burn_fee == 0 {
            return;
        }
        self.previous_tax_fee = self.tax_fee;
        self.previous_burn_fee = self.burn_fee;
        
        self.tax_fee = 0;
        self.burn_fee = 0;
    }

    fn restore_all_fee(&mut self){
        if self.tax_fee == 0 && self.burn_fee == 0 {
            self.tax_fee = self.previous_tax_fee;
            self.burn_fee = self.previous_burn_fee;
        }
    }


    /// Update the state adding a new operator for a given token id and address.
    /// Succeeds even if the `operator` is already an operator for this
    /// `token_id` and `address`.
    fn add_operator(
        &mut self,
        owner: &Address,
        operator: &Address,
        state_builder: &mut StateBuilder,
    ) {
        let mut owner_state = self.t_token.entry(*owner).or_insert_with(|| AddressState {
            balance:   TokenAmountU256(U256::zero()),
            operators: state_builder.new_set(),
        });
        owner_state.operators.insert(*operator);
    }

    /// Update the state removing an operator for a given token id and address.
    /// Succeeds even if the `operator` is not an operator for this `token_id`
    /// and `address`.
    fn remove_operator(&mut self, owner: &Address, operator: &Address) {
        self.t_token.entry(*owner).and_modify(|address_state| {
            address_state.operators.remove(operator);
        });
    }

    /// Mint an amount of Gona tokens.
    /// Results in an error if the token id does not exist in the state.
    fn mint(
        &mut self,
        token_id: &ContractTokenId,
        amount: ContractTokenAmount,
        owner: &Address,
        state_builder: &mut StateBuilder,
    ) -> ContractResult<()> {
        ensure_eq!(token_id, &TOKEN_ID_Gona, ContractError::InvalidTokenId);
        ensure!(self.minted == false, ContractError::Unauthorized);
        self.minted = true;
        let mut owner_state = self.r_token.entry(*owner).or_insert_with(|| AddressState {
            balance:   TokenAmountU256(U256::zero()),
            operators: state_builder.new_set(),
        });

        owner_state.balance += amount;
        Ok(())
    }



    fn is_excluded_from_fee(self, sender:&Address) -> ContractResult<bool>{
        let result = self.is_excluded_from_tax.get(sender).is_some();
        Ok(result)
    }

    fn excluded_from_fee(&mut self, sender:Address) -> ContractResult<()> {
        let result = self.is_excluded_from_tax.insert(sender, true);    
        if let Some(_) = result{
            Ok(())
        }else{
            Err(ContractError::Custom(CustomContractError::InvokeContractError))
        }
    }

    fn include_in_fee(&mut self, sender:&Address) -> ContractResult<()> {
        let result = self.is_excluded_from_tax.remove_and_get(&sender);  
        if let Some(_) = result{
            Ok(())
        }else{
            Err(ContractError::Custom(CustomContractError::InvokeContractError))
        }  
    }

    fn set_tax_fee(&mut self, fee:u8) -> ContractResult<()> {
        self.tax_fee = fee;
        Ok(())
    }

    fn set_burn_fee(&mut self, fee:u8) -> ContractResult<()> {
        self.burn_fee = fee;
        Ok(())
    }

    fn reflection_from_token
        (&mut self, amount:TokenAmountU256, tax_fee:bool) 
        -> ContractResult<U256> {
            ensure!(amount <= self.total_supply, ContractError::Custom(CustomContractError::CannotTransferMoreThanMax));
            match tax_fee {
                true => {
                    let(r_amount, _, _, _, _, _) = self.get_values(amount);
                    Ok(r_amount)
                },
                false => {
                    let( _, r_transfer_amount, _, _, _, _) = self.get_values(amount);
                    Ok(r_transfer_amount)
                }
            }
        }


        //todo() ❌ 
        fn exclude_from_reward(
            &mut self, 
            sender:Address,
            state_builder: &mut StateBuilder,
        ) -> ContractResult<()> {
            
            let excluded = self.is_excluded_from_reward.get(&sender).is_some();
            if excluded {
                return Err(ContractError::Custom(CustomContractError::UserIsAlreadyExcluded))
            }
            let address = sender.clone();
            let mut sender_state = self.r_token.entry(address).or_insert_with(|| AddressState {
                balance:   TokenAmountU256(U256::zero()),
                operators: state_builder.new_set(),
            });

            if sender_state.balance.0 > U256::from(0) {
            }
            
            Ok(())

        }
    



















    fn token_from_reflection(&self,r_amount:U256) -> ContractResult<U256> {
        let r_supply = self.total_reflective_tokens.0;
        ensure!(r_amount <= r_supply, ContractError::Unauthorized);
        let rate = self.get_rate();
        let balance = r_amount.div_mod(rate).0;
        Ok(balance)
    }



    /// Get the rate of of the Refection
    fn get_rate(&self) -> U256 {
        let (r_supply, t_supply) = self.get_current_supply();  
        let rate = r_supply.div_mod(t_supply).0;
        rate    
    }



    /// Get the current Supply in the contract
    fn get_current_supply(&self) -> (U256,U256) {
        //let reflective_supply = U256::from(TokenAmountU256::from(U256::zero()));
        let mut r_supply = self.total_reflective_tokens.0;
        let r_supply_clone = r_supply.clone();
        let mut t_supply = self.total_supply.0;
        let t_supply_clone = t_supply.clone();
        let excluded = &self.excluded;

        for address in excluded.iter() {
            if &self.r_token.get(&address.clone()).unwrap().balance.0 > &r_supply 
            || 
            &self.t_token.get(&address.clone()).unwrap().balance.0 > &t_supply {
               return(r_supply_clone, t_supply_clone);
            }
            r_supply -= self.r_token.get(&address.clone()).unwrap().balance.0;
            t_supply -= self.t_token.get(&address.clone()).unwrap().balance.0;
        }
        if r_supply < r_supply_clone.div_mod(t_supply_clone).0 {
            return(r_supply_clone, t_supply_clone);
        }
        (r_supply, t_supply)

    }























    /// Burn an amount of Gona tokens.
    /// Results in an error if the token id does not exist in the state or if
    /// the owner address has insufficient tokens to do the burn.
    fn burn(
        &mut self,
        token_id: &ContractTokenId,
        amount: ContractTokenAmount,
        owner: &Address,
        state_builder: &mut StateBuilder,
    ) -> ContractResult<()> {
        ensure_eq!(token_id, &TOKEN_ID_Gona, ContractError::InvalidTokenId);
        if amount == TokenAmountU256(U256::zero()) {
            return Ok(());
        }
        let from_state = self.t_token.get(owner).ok_or(ContractError::InsufficientFunds)?;
        ensure!(from_state.balance >= amount, ContractError::InsufficientFunds);
        
        let burn_address = Address::from(AccountAddress([9; 32]));
        self.transfer(token_id, amount, owner, &burn_address, state_builder)?;

        Ok(())
    }


    /// Check if state contains any implementors for a given standard.
    fn have_implementors(&self, std_id: &StandardIdentifierOwned) -> SupportResult {
        if let Some(addresses) = self.implementors.get(std_id) {
            SupportResult::SupportBy(addresses.to_vec())
        } else {
            SupportResult::NoSupport
        }
    }

    /// Set implementors for a given standard.
    fn set_implementors(
        &mut self,
        std_id: StandardIdentifierOwned,
        implementors: Vec<ContractAddress>,
    ) {
        self.implementors.insert(std_id, implementors);
    }
}

// Contract functions

/// Initialize contract instance with no initial tokens.
/// Logs a `Mint` event for the single token id with no amounts.
/// Logs a `TokenMetadata` event with the metadata url and hash.
/// Logs a `NewAdmin` event with the invoker as admin.
#[init(
    contract = "gona_reflective",
    enable_logger,
    parameter = "SetMetadataUrlParams",
    event = "GonaEvent"
)]
fn contract_init(
    ctx: &InitContext,
    state_builder: &mut StateBuilder,
    logger: &mut impl HasLogger,
) -> InitResult<State> {
    // Parse the parameter.
    let params: SetMetadataUrlParams = ctx.parameter_cursor().get()?;
    // Get the instantiator of this contract instance to be used as the initial
    // admin.
    let invoker = Address::Account(ctx.init_origin());

    // Create the metadata_url
    let metadata_url = MetadataUrl {
        url:  params.url.clone(),
        hash: params.hash,
    };

    // Construct the initial contract state.
    let state = State::new(state_builder, invoker, metadata_url.clone());
    // Log event for the newly minted token.
    logger.log(&GonaEvent::Cis2Event(Cis2Event::Mint(MintEvent {
        token_id: TOKEN_ID_Gona,
        amount:   ContractTokenAmount::from(TokenAmountU256(U256::zero())),
        owner:    invoker,
    })))?;

    // Log event for where to find metadata for the token
    logger.log(&GonaEvent::Cis2Event(Cis2Event::TokenMetadata::<_, ContractTokenAmount>(
        TokenMetadataEvent {
            token_id: TOKEN_ID_Gona,
            metadata_url,
        },
    )))?;

    // Log event for the new admin.
    logger.log(&GonaEvent::NewAdmin {
        new_admin: NewAdminEvent {
            new_admin: invoker,
        },
    })?;

    Ok(state)
}


/// Transfer the admin address to a new admin address.
///
/// It rejects if:
/// - Sender is not the current admin of the contract instance.
/// - It fails to parse the parameter.
#[receive(
    contract = "gona_reflective",
    name = "updateAdmin",
    parameter = "Address",
    error = "ContractError",
    enable_logger,
    mutable
)]
fn contract_update_admin(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut impl HasLogger,
) -> ContractResult<()> {
    // Check that only the current admin is authorized to update the admin address.
    ensure_eq!(ctx.sender(), host.state().admin, ContractError::Unauthorized);

    // Parse the parameter.
    let new_admin = ctx.parameter_cursor().get()?;

    // Update the admin variable.
    host.state_mut().admin = new_admin;

    // Log a new admin event.
    logger.log(&GonaEvent::NewAdmin {
        new_admin: NewAdminEvent {
            new_admin,
        },
    })?;

    Ok(())
}

/// Pause/Unpause this smart contract instance by the admin. All non-admin
/// state-mutative functions (wrap, unwrap, transfer, updateOperator) cannot be
/// executed when the contract is paused.
///
/// It rejects if:
/// - Sender is not the admin of the contract instance.
/// - It fails to parse the parameter.
#[receive(
    contract = "gona_reflective",
    name = "setPaused",
    parameter = "SetPausedParams",
    error = "ContractError",
    mutable
)]
fn contract_set_paused(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
    // Check that only the admin is authorized to pause/unpause the contract.
    ensure_eq!(ctx.sender(), host.state().admin, ContractError::Unauthorized);

    // Parse the parameter.
    let params: SetPausedParams = ctx.parameter_cursor().get()?;

    // Update the paused variable.
    host.state_mut().paused = params.paused;

    Ok(())
}

/// Update the metadata URL in this smart contract instance.
///
/// It rejects if:
/// - Sender is not the admin of the contract instance.
/// - It fails to parse the parameter.
#[receive(
    contract = "gona_reflective",
    name = "setMetadataUrl",
    parameter = "SetMetadataUrlParams",
    error = "ContractError",
    enable_logger,
    mutable
)]
fn contract_state_set_metadata_url(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut impl HasLogger,
) -> ContractResult<()> {
    // Check that only the admin is authorized to update the URL.
    ensure_eq!(ctx.sender(), host.state().admin, ContractError::Unauthorized);

    // Parse the parameter.
    let params: SetMetadataUrlParams = ctx.parameter_cursor().get()?;

    // Create the metadata_url
    let metadata_url = MetadataUrl {
        url:  params.url.clone(),
        hash: params.hash,
    };

    // Update the hash variable.
    *host.state_mut().metadata_url = metadata_url.clone();

    // Log an event including the new metadata for this token.
    logger.log(&GonaEvent::Cis2Event(Cis2Event::TokenMetadata::<_, ContractTokenAmount>(
        TokenMetadataEvent {
            token_id: TOKEN_ID_Gona,
            metadata_url,
        },
    )))?;

    Ok(())
}

// Contract functions required by the CIS-2 standard

type TransferParameter = TransferParams<ContractTokenId, ContractTokenAmount>;

/// Execute a list of token transfers, in the order of the list.
///
/// Logs a `Transfer` event and invokes a receive hook function for every
/// transfer in the list.
///
/// It rejects if:
/// - It fails to parse the parameter.
/// - Any of the transfers fail to be executed, which could be if:
///     - The `token_id` does not exist.
///     - The sender is not the owner of the token, or an operator for this
///       specific `token_id` and `from` address.
///     - The token is not owned by the `from`.
/// - Fails to log event.
/// - Any of the receive hook function calls rejects.
#[receive(
    contract = "gona_reflective",
    name = "transfer",
    parameter = "TransferParameter",
    error = "ContractError",
    enable_logger,
    mutable
)]
fn contract_transfer(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut impl HasLogger,
) -> ContractResult<()> {
    // Check that contract is not paused.
    ensure!(!host.state().paused, ContractError::Custom(CustomContractError::ContractPaused));

    // Parse the parameter.
    let TransferParams(transfers): TransferParameter = ctx.parameter_cursor().get()?;
    // Get the sender who invoked this contract function.
    let sender = ctx.sender();

    for Transfer {
        token_id,
        amount,
        from,
        to,
        data,
    } in transfers
    {
        let (state, builder) = host.state_and_builder();
        // Authenticate the sender for this transfer
        ensure!(from == sender || state.is_operator(&sender, &from), ContractError::Unauthorized);
        let to_address = to.address();
        // Update the contract state
        let transfer_amount = state.transfer(&token_id, amount, &from, &to_address, builder)?;
        
        let amount = TokenAmountU256::from(transfer_amount);

        // Log transfer event
        logger.log(&GonaEvent::Cis2Event(Cis2Event::Transfer(TransferEvent {
            token_id,
            amount,
            from,
            to: to_address,
        })))?;

        // If the receiver is a contract: invoke the receive hook function.
        if let Receiver::Contract(address, function) = to {
            let parameter = OnReceivingCis2Params {
                token_id,
                amount,
                from,
                data,
            };
            host.invoke_contract(
                &address,
                &parameter,
                function.as_entrypoint_name(),
                Amount::zero(),
            )?;
        }
    }
    Ok(())
}




#[receive(
    contract = "gona_reflective",
    name = "burn",
    parameter = "Address",
    error = "ContractError",
    enable_logger,
    mutable
)]
fn contract_burn(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut impl HasLogger,
) -> ContractResult<()> {
    // Parse the parameter.
    let params: BurnParams = ctx.parameter_cursor().get()?;
    // Get the sender who invoked this contract function.

    let (state, state_builder) = host.state_and_builder();
    
    state.burn(&params.token_id, params.amount, &ctx.sender(), state_builder)?;

        // Log the appropriate event
         // Log transfer event
         logger.log(&GonaEvent::Cis2Event(Cis2Event::Burn(BurnEvent{
            token_id: params.token_id,
            amount: params.amount,
            owner: ctx.sender(),
        })))?;

    Ok(())
}



/// Enable or disable addresses as operators of the sender address.
/// Logs an `UpdateOperator` event.
///
/// It rejects if:
/// - It fails to parse the parameter.
/// - Fails to log event.
#[receive(
    contract = "gona_reflective",
    name = "updateOperator",
    parameter = "UpdateOperatorParams",
    error = "ContractError",
    enable_logger,
    mutable
)]
fn contract_update_operator(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut impl HasLogger,
) -> ContractResult<()> {
    ensure!(!host.state().paused, ContractError::Custom(CustomContractError::ContractPaused));

    // Parse the parameter.
    let UpdateOperatorParams(params) = ctx.parameter_cursor().get()?;
    // Get the sender who invoked this contract function.
    let sender = ctx.sender();

    let (state, state_builder) = host.state_and_builder();
    for param in params {
        // Update the operator in the state.
        match param.update {
            OperatorUpdate::Add => state.add_operator(&sender, &param.operator, state_builder),
            OperatorUpdate::Remove => state.remove_operator(&sender, &param.operator),
        }

        // Log the appropriate event
        logger.log(&GonaEvent::Cis2Event(
            Cis2Event::<ContractTokenId, ContractTokenAmount>::UpdateOperator(
                UpdateOperatorEvent {
                    owner:    sender,
                    operator: param.operator,
                    update:   param.update,
                },
            ),
        ))?;
    }

    Ok(())
}

/// Parameter type for the CIS-2 function `balanceOf` specialized to the subset
/// of TokenIDs used by this contract.
pub type ContractBalanceOfQueryParams = BalanceOfQueryParams<ContractTokenId>;

pub type ContractBalanceOfQueryResponse = BalanceOfQueryResponse<ContractTokenAmount>;

/// Get the balance of given token IDs and addresses.
///
/// It rejects if:
/// - It fails to parse the parameter.
/// - Any of the queried `token_id` does not exist.
#[receive(
    contract = "gona_reflective",
    name = "balanceOf",
    parameter = "ContractBalanceOfQueryParams",
    return_value = "ContractBalanceOfQueryResponse",
    error = "ContractError"
)]
fn contract_balance_of(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<ContractBalanceOfQueryResponse> {
    // Parse the parameter.
    let params: ContractBalanceOfQueryParams = ctx.parameter_cursor().get()?;
    // Build the response.
    let mut response = Vec::with_capacity(params.queries.len());
    for query in params.queries {
        // Query the state for balance.
        let amount = host.state().balance(&query.token_id, &query.address)?;
        response.push(amount);
    }
    let result = ContractBalanceOfQueryResponse::from(response);
    Ok(result)
}




#[receive(
    contract = "gona_reflective",
    name = "mint",
    parameter = "MintParams",
    error = "ContractError",
    enable_logger,
    mutable,
)]
fn contract_mint(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut impl HasLogger,
) -> ContractResult<()> {
    // Check that contract is not paused.
    ensure!(!host.state().paused, ContractError::Custom(CustomContractError::ContractPaused));

    // Ensure that only the admin can call this
    ensure_eq!(ctx.sender(), host.state().admin, ContractError::Unauthorized);

    
    // ensure that tokens has not been minted
    ensure!(host.state().minted == false, ContractError::Custom(CustomContractError::TotalSupplyHasAlreadyBeenMinted));


    // Parse the parameter.
    let params: MintParams = ctx.parameter_cursor().get()?;
    // Get the sender who invoked this contract function.
    let sender = ctx.sender();

    let mint_to = params.to;

    let (state, state_builder) = host.state_and_builder();

    let amount = state.total_reflective_tokens;

    // Update the state.
    state.mint(&TOKEN_ID_Gona, amount, &mint_to, state_builder)?;

    let amount = state.reflection_from_token(amount, false)?;

    // Log the newly minted tokens.
    logger.log(&GonaEvent::Cis2Event(Cis2Event::Mint(MintEvent {
        token_id: TOKEN_ID_Gona,
        amount:   ContractTokenAmount::from(amount),
        owner:    sender,
    })))?;

    state.minted = true;
    // Only logs a transfer event if the receiver is not the sender.
    // Only executes the `OnReceivingCis2` hook if the receiver is not the sender
    // and the receiver is a contract.
    // if sender != receive_address {
    //     logger.log(&GonaEvent::Cis2Event(Cis2Event::Transfer(TransferEvent {
    //         token_id: TOKEN_ID_Gona,
    //         amount:   ContractTokenAmount::from(amount),
    //         from:     sender,
    //         to:       receive_address,
    //     })))?;
    // }
    logger.log(&GonaEvent::Cis2Event(Cis2Event::Transfer(TransferEvent {
        token_id: TOKEN_ID_Gona,
        amount:   ContractTokenAmount::from(amount),
        from:     Address::Account(AccountAddress([0; 32])),
        to:       mint_to,
    })))?;

    Ok(())
}






/// Takes a list of queries. Each query contains an owner address and some
/// address that will be checked if it is an operator to the owner address.
///
/// It rejects if:
/// - It fails to parse the parameter.
#[receive(
    contract = "gona_reflective",
    name = "operatorOf",
    parameter = "OperatorOfQueryParams",
    return_value = "OperatorOfQueryResponse",
    error = "ContractError"
)]
fn contract_operator_of(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<OperatorOfQueryResponse> {
    // Parse the parameter.
    let params: OperatorOfQueryParams = ctx.parameter_cursor().get()?;
    // Build the response.
    let mut response = Vec::with_capacity(params.queries.len());
    for query in params.queries {
        // Query the state if the `address` being an `operator` of `owner`.
        let is_operator = host.state().is_operator(&query.address, &query.owner);
        response.push(is_operator);
    }
    let result = OperatorOfQueryResponse::from(response);
    Ok(result)
}

/// Parameter type for the CIS-2 function `tokenMetadata` specialized to the
/// subset of TokenIDs used by this contract.
// This type is pub to silence the dead_code warning, as this type is only used
// for when generating the schema.
pub type ContractTokenMetadataQueryParams = TokenMetadataQueryParams<ContractTokenId>;

/// Get the token metadata URLs and checksums given a list of token IDs.
///
/// It rejects if:
/// - It fails to parse the parameter.
/// - Any of the queried `token_id` does not exist.
#[receive(
    contract = "gona_reflective",
    name = "tokenMetadata",
    parameter = "ContractTokenMetadataQueryParams",
    return_value = "TokenMetadataQueryResponse",
    error = "ContractError"
)]
fn contract_token_metadata(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<TokenMetadataQueryResponse> {
    // Parse the parameter.
    let params: ContractTokenMetadataQueryParams = ctx.parameter_cursor().get()?;

    // Build the response.
    let mut response = Vec::with_capacity(params.queries.len());
    for token_id in params.queries {
        // Check the token exists.
        ensure_eq!(token_id, TOKEN_ID_Gona, ContractError::InvalidTokenId);

        response.push(host.state().metadata_url.clone());
    }
    let result = TokenMetadataQueryResponse::from(response);
    Ok(result)
}

/// Function to view the basic state of the contract.
#[receive(
    contract = "gona_reflective",
    name = "view",
    return_value = "ReturnBasicState",
    error = "ContractError"
)]
fn contract_view(_ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<ReturnBasicState> {
    let state = ReturnBasicState {
        admin:        host.state().admin,
        paused:       host.state().paused,
        metadata_url: host.state().metadata_url.clone(),
    };
    Ok(state)
}

/// Get the supported standards or addresses for a implementation given list of
/// standard identifiers.
///
/// It rejects if:
/// - It fails to parse the parameter.
#[receive(
    contract = "gona_reflective",
    name = "supports",
    parameter = "SupportsQueryParams",
    return_value = "SupportsQueryResponse",
    error = "ContractError"
)]
fn contract_supports(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<SupportsQueryResponse> {
    // Parse the parameter.
    let params: SupportsQueryParams = ctx.parameter_cursor().get()?;

    // Build the response.
    let mut response = Vec::with_capacity(params.queries.len());
    for std_id in params.queries {
        if SUPPORTS_STANDARDS.contains(&std_id.as_standard_identifier()) {
            response.push(SupportResult::Support);
        } else {
            response.push(host.state().have_implementors(&std_id));
        }
    }
    let result = SupportsQueryResponse::from(response);
    Ok(result)
}

/// Set the addresses for an implementation given a standard identifier and a
/// list of contract addresses.
///
/// It rejects if:
/// - Sender is not the admin of the contract instance.
/// - It fails to parse the parameter.
#[receive(
    contract = "gona_reflective",
    name = "setImplementors",
    parameter = "SetImplementorsParams",
    error = "ContractError",
    mutable
)]
fn contract_set_implementor(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
    // Check that only the admin is authorized to set implementors.
    ensure_eq!(ctx.sender(), host.state().admin, ContractError::Unauthorized);
    // Parse the parameter.
    let params: SetImplementorsParams = ctx.parameter_cursor().get()?;
    // Update the implementors in the state
    host.state_mut().set_implementors(params.id, params.implementors);
    Ok(())
}

/// Upgrade this smart contract instance to a new module and call optionally a
/// migration function after the upgrade.
///
/// It rejects if:
/// - Sender is not the admin of the contract instance.
/// - It fails to parse the parameter.
/// - If the ugrade fails.
/// - If the migration invoke fails.
///
/// This function is marked as `low_level`. This is **necessary** since the
/// high-level mutable functions store the state of the contract at the end of
/// execution. This conflicts with migration since the shape of the state
/// **might** be changed by the migration function. If the state is then written
/// by this function it would overwrite the state stored by the migration
/// function.
#[receive(
    contract = "gona_reflective",
    name = "upgrade",
    parameter = "UpgradeParams",
    error = "ContractError",
    low_level
)]
fn contract_upgrade<S: HasStateApi>(
    ctx: &ReceiveContext,
    host: &mut impl HasHost<S>,
) -> ContractResult<()> {
    // Read the top-level contract state.
    let state: State<S> = host.state().read_root()?;

    // Check that only the admin is authorized to upgrade the smart contract.
    ensure_eq!(ctx.sender(), state.admin, ContractError::Unauthorized);
    // Parse the parameter.
    let params: UpgradeParams = ctx.parameter_cursor().get()?;
    // Trigger the upgrade.
    host.upgrade(params.module)?;
    // Call the migration function if provided.
    if let Some((func, parameters)) = params.migrate {
        host.invoke_contract_raw(
            &ctx.self_address(),
            parameters.as_parameter(),
            func.as_entrypoint_name(),
            Amount::zero(),
        )?;
    }
    Ok(())
}