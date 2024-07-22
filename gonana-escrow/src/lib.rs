#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract
use concordium_std::*;
use core::fmt::Debug;

#[derive(Debug, Serialize, SchemaType)]
pub struct Invoice {
    /// id of the transaction offchain 
    id: String,
    /// amount that was paid for the transaction
    amount: Amount,
    /// payer of the transaction
    payer: PublicKeyEd25519,
    /// receiver of the transaction
    receiver: PublicKeyEd25519,
    /// The time that the transaction was made
    timestamp: Timestamp,
    /// Has the transaction been fufilled by the smart contract
    fufilled: bool
}
impl Invoice {
    pub fn new(
        id:String, 
        amount:Amount, 
        payer:PublicKeyEd25519, 
        receiver:PublicKeyEd25519,
        ) -> Self {
        Self { id, amount, payer, receiver, timestamp: Timestamp::now(), fufilled: false }
    }
}

#[derive(Serial,Deserial,SchemaType)]
pub struct PayParam {
   /// id of the transaction offchain 
   pub id: String,
   /// payer of the transaction
   pub payer: PublicKeyEd25519,
   /// receiver of the transaction
   pub receiver: PublicKeyEd25519, 
}
pub type ContractResult<A> = Result<A, Error>;


#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
struct State<S = StateApi> {
    payments: StateMap<String, Invoice, S>,
    admin: Address,
    smart_wallet: ContractAddress
}

impl State {
    fn fufil(&mut self, id:String)-> ContractResult<()> {
       let mut invoice = self.payments
            .entry(id)
            .occupied_or(Error::InvoiceNotFoundError)?;
        invoice.fufilled = true;
        Ok(())
    }
}

/// Your smart contract errors.
#[derive(Debug, PartialEq, Eq, Reject, Serialize, SchemaType)]
pub enum Error {
    /// Failed parsing the parameter.
    #[from(ParseError)]
    ParseParams,
    /// Caller is not the contract admin
    CallerNotAdminError,
    InvoiceNotFoundError,
    InvoiceAlreadyPaid,
    InvokeContractError
}

/// Mapping errors related to contract invocations to CustomContractError.
impl<T> From<CallContractError<T>> for Error {
    fn from(_cce: CallContractError<T>) -> Self { Self::InvokeContractError }
}
/// Init function that creates a new smart contract.
#[init(contract = "gonana_escrow",parameter = "ContractAddress",)]
fn init(ctx: &InitContext, state_builder: &mut StateBuilder) -> InitResult<State> {
    let smart_wallet: ContractAddress = ctx.parameter_cursor().get()?;
    Ok(State {
        payments: state_builder.new_map(),
        admin: Address::Account(ctx.init_origin()),
        smart_wallet 
    })
}


/// This function simulates a payment to be held in escrow, done by a user.
#[receive(
    contract = "gonana_escrow",
    name = "pay",
    parameter = "PayParam",
    error = "Error",
    payable,
    mutable
)]
fn pay(ctx: &ReceiveContext, host: &mut Host<State>, amount:Amount) -> Result<(), Error> {
    // Your code
    let param: PayParam = ctx.parameter_cursor().get()?; // Returns Error::ParseError on failure
    let invoice = Invoice::new(
        param.id.clone(),
        amount,
        param.payer,
        param.receiver
    );
    host.state_mut().payments.entry(param.id).or_insert(invoice);
    Ok(())
    
}


#[receive(
    contract = "gonana_escrow",
    name = "withdraw",
    parameter = "String",
    error = "Error",
    mutable
)]
fn withdraw(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), Error> {
    // Your code
    let id: String = ctx.parameter_cursor().get()?; 
    let to = host.state().smart_wallet; 
    ensure!(ctx.sender().cmp(&host.state().admin) == std::cmp::Ordering::Equal, Error::CallerNotAdminError);
    let invoice = host.state()
        .payments.get(&id)
        .ok_or(Error::InvoiceNotFoundError)?;
    ensure!(invoice.fufilled == false, Error::InvoiceAlreadyPaid);
    let receiver = invoice.receiver;
    let amount = invoice.amount;
    let method = OwnedEntrypointName::new_unchecked("depositCcd".into());

    host.state_mut().fufil(id.clone())?;
    host.invoke_contract(
        &to, 
        &receiver, 
        method.as_entrypoint_name(), 
        amount
    )?;
    Ok(())
    
}


