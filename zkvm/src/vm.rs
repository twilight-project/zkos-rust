// ignore warnings for now
#![allow(warnings)]

use bulletproofs::r1cs;
use core::iter;
use core::iter::FromIterator;
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use crate::constraints::{Commitment, Constraint, Expression, Variable};
use crate::contract::{Anchor, Contract, ContractID, PortableItem};
use crate::encoding::*;
use crate::errors::VMError;
use crate::fees::{fee_flavor, CheckedFee};
use crate::ops::Instruction;
use crate::predicate::{CallProof, Predicate};
use crate::program::ProgramItem;
use crate::scalar_witness::ScalarWitness;
use crate::types::*;
use crate::zkos_types::{IOType, Input, Output, OutputCoin, OutputMemo};
use rangeproof;
use rangeproof::BitRange;
use std::mem;

/// Current tx version determines which extension opcodes are treated as noops (see VM.extension flag).
pub const CURRENT_VERSION: u64 = 1;


///VM for Script execution
#[derive(Debug)]
pub struct VMScript<'d, CS, R>
where
    CS: r1cs::RandomizableConstraintSystem,
    R: VMRun<CS>,
{
    // stack of all items in the VM
    stack: Vec<Item>,

    current_run: R::RunType,
    run_stack: Vec<R::RunType>,
    // collect all cs related operations
    delegate: &'d mut R,
    // tx inputs and outputs
    inputs_tx: &'d [Input],
    outputs_tx: &'d [Output],
    // Flag used for contract initialization. 0 -> no, 1 -> yes
    // in case of contract initialization, input state is not loaded on stack
    // input state is zero in this case and is not used in the program
    // contract_init_flag: u8,
    tx_data: Option<crate::String>,
}

pub trait VMRun<CS: r1cs::RandomizableConstraintSystem> {
    /// Container type for the currently running program.
    type RunType;

    /// Adds a Commitment to the underlying constraint system, producing a high-level variable
    fn commit_variable(
        &mut self,
        com: &Commitment,
    ) -> Result<(CompressedRistretto, r1cs::Variable), VMError>;

    /// Returns the delegate's underlying constraint system
    fn cs(&mut self) -> &mut CS;

    /// Returns the next instruction.
    /// Returns Err() upon decoding/format error.
    /// Returns Ok(Some()) if there is another instruction available.
    /// Returns Ok(None) if there is no more instructions to execute.
    fn next_instruction(&mut self, run: &mut Self::RunType)
        -> Result<Option<Instruction>, VMError>;

    fn new_run(&self, prog: ProgramItem) -> Result<Self::RunType, VMError>;
}


///SCRIPT EXECUTION
///
impl<'d, CS, R> VMScript<'d, CS, R>
where
    CS: r1cs::RandomizableConstraintSystem,
    R: VMRun<CS>,
{
    /// Instantiates a new ScriptVM instance.
    pub fn new(
        run: R::RunType,
        delegate: &'d mut R,
        inputs: &'d [Input],
        outputs: &'d [Output],
        tx_data: Option<String>, // contract_init_flag: u8,
    ) -> Self {
        VMScript {
            delegate,
            stack: Vec::new(),
            current_run: run,
            run_stack: Vec::new(),
            inputs_tx: inputs,
            outputs_tx: outputs,
            tx_data,
            // contract_init_flag: contract_init_flag,
        }
    }
    /// Contract Initalization is treated differently because a contract address/state has to be deployed
    /// can not use a lend program to do this because no address and state exists already  
    /// Initialze the VM Stack with the inputs and outputs of the transaction
    /// trying to deploy a contract
    /// Assuming a single contract is being deployed using a Single input coin acccount for initialization
    pub fn initialize_deploy_contract_stack(&mut self) -> Result<(), VMError> {
        // Contract deploy transaction will have a coin input and corresponding memo output
        // and Zero state inputs and initialized outputs
        // Just pushing memo and output state on stack for efficiency
        // assuming inputs and outputs are in the correct order
        // the proofs for zero balance and zero state are verified in the transaction witness already before firing up the stack
        // Coin -> Memo -> State is the correct order based on the inputs

        // get inputcoin
        let input_coin = match self.inputs_tx[0].as_out_coin(){
            Some(coin) => coin,
            None => return Err(VMError::InvalidInputCoin),
        };
        // get corresponding OutputMemo
        let out_memo: &OutputMemo = match self.outputs_tx[0].as_out_memo(){
            Some(memo) => memo,
            None => return Err(VMError::InvalidOutputMemo),
        };
        //compare the coin and memo owner address
        if input_coin.owner != out_memo.owner {
            return Err(VMError::InvalidCoinMemo);
        }
        //push ther memo commitment and data if present on stack
        //convert CompressedRistretto to String
        let str: String = String::from(out_memo.commitment.clone());
        self.push_item(str);
        // push the memo data on stack
        match out_memo.data.clone() {
            Some(data) => {
                //load the memo data vector on stack
                for d in data.iter() {
                    self.push_item(d.clone());
                }
                //self.push_item(data);
            }
            None => (),
        }
        //push the output state on stack
        let out_state = match self.outputs_tx[1].as_out_state(){
            Some(state) => state,
            None => return Err(VMError::InvalidOutputState),
        };
        let out_value = out_state.unwrap().commitment.clone();
        self.push_item(String::from(out_value));
        //push the state variables if present
        let output_state_variables = out_state.unwrap().state_variables.clone();
        if output_state_variables.is_some() {
            let num_state_variables = output_state_variables.len() as u64;

            let out_state_variables = match out_state.state_variables.clone(){
                Some(vars) => vars,
                None => return Err(VMError::InvalidOutputState),
            };
            // push number of state variables
            // self.push_item(String::U64(num_state_variables));
            //push state variables
            for var in out_state_variables.iter() {
                self.push_item(var.clone());
            }
        }
        Ok(())
    }
    ///Initialize the VM Stack with the inputs and outputs of the regular script transactions
    pub fn initialize_stack(&mut self) -> Result<(), VMError> {
        // Initialize the stack with the inputs and outputs of the transaction
        //assuming inputs and outputs are in the correct order
        // Coin -> Memo -> State is the correct order based on the inputs
        //i.e., coin input -> Memo output
        //      Memo input -> coin output
        //      State input -> State output

        for (i, input) in self.inputs_tx.iter().enumerate() {
            //match inputtype
            match input.in_type {
                IOType::Coin => {
                    let in_coin: &OutputCoin = match input.as_out_coin(){
                        Some(coin) => coin,
                        None => return Err(VMError::InvalidInputCoin),
                    };
                    // get corresponding OutputMemo
                    let out_memo: &OutputMemo = match self.outputs_tx[i].as_out_memo(){
                        Some(memo) => memo,
                        None => return Err(VMError::InvalidOutputMemo),
                    
                    };
                    //compare the coin and memo owner address
                    if in_coin.owner != out_memo.owner {
                        return Err(VMError::InvalidCoinMemo);
                    }
                    //push the Optional Coin data onto stack
                    // check if data is present
                    //  match input.input.as_coin_memo_data() {
                    //    Some(data) => {
                    //      self.push_item(data.clone());
                    //   }
                    //   None => (),
                    // }
                    //push ther memo commitment and data if present on stack
                    //convert CompressedRistretto to String
                    let str: String = String::from(out_memo.commitment.clone());
                    self.push_item(str);

                    //push the memo data if present
                    match out_memo.data.clone() {
                        Some(data) => {
                            //load the memo data vector on stack
                            for d in data.iter() {
                                self.push_item(d.clone());
                            }
                            //self.push_item(data);
                        }
                        None => (),
                    }
                }
                IOType::Memo => {
                    let in_memo: &OutputMemo = match input.as_out_memo(){
                        Some(memo) => memo,
                        None => return Err(VMError::InvalidInputMemo),
                    
                    };
                    // get corresponding OutputCoin
                    let out_coin: &OutputCoin = match self.outputs_tx[i].as_out_coin(){
                        Some(coin) => coin,
                        None => return Err(VMError::InvalidOutputCoin),
                    
                    };
                    //compare the coin and memo owner address
                    if in_memo.owner != out_coin.owner {
                        return Err(VMError::InvalidCoinMemo);
                    }

                    // Push the commitment data stored in the Memo originially
                    // could be needed in some caseds depending on the type script being executed
                    match input.input.as_commitment() {
                        Some(data) => {
                            self.push_item(String::from(data.clone()));
                        }
                        None => return Err(VMError::InvalidInputMemoCommitment),
                    }

                    //push the memo data if present on stack
                    match in_memo.data.clone() {
                        Some(data) => {
                            //load the memo data vector on stack
                            for d in data.iter() {
                                self.push_item(d.clone());
                            }
                            //self.push_item(data);
                        }
                        None => (),
                    }
                    //???STILL HAVE TO DECIDE HOW TO PUSH MEMO COMMITMENT DATA ON STACK
                    // For DEMO purposes PUSHING temporary data Stored in Memo
                    //push the Optional Memo data onto stack
                    // push the coin value on stack if present
                    match input.input.get_coin_value_from_memo() {
                        Some(data) => {
                            self.push_item(String::from(data.clone()));
                        }
                        None => (),
                    }
                }
                IOType::State => {
                    // get input and output state
                    let in_state = input.as_out_state();
                    let out_state = self.outputs_tx[i].as_out_state();
                    //compare the state owner address and script address
                    if in_state.unwrap().owner != out_state.unwrap().owner
                        && in_state.unwrap().script_address != out_state.unwrap().script_address
                    {
                        return Err(VMError::InvalidInputOutputState);
                    }
                    //check output nonce = input nonce + 1
                    if in_state.unwrap().nonce + 1 != out_state.unwrap().nonce {
                        return Err(VMError::InvalidInputOutputState);
                    }
                    //load input / output Value
                    let in_value = in_state.unwrap().commitment.clone();
                    self.push_item(String::from(in_value));

                    let out_value = out_state.unwrap().commitment.clone();
                    self.push_item(String::from(out_value));

                    let input_state_variables = in_state.unwrap().state_variables.clone();
                    //Load State variables if present
                    //match input_state_variables {
                    //None => continue,
                    if input_state_variables.is_some() {
                        let num_state_variables =
                            input_state_variables.clone().unwrap().len() as u64;
                        let out_state = self.outputs_tx[i].as_out_state();
                        let out_state_variables =
                            out_state.unwrap().state_variables.clone().unwrap();
                        //check if len of state variables is same
                        if num_state_variables != out_state_variables.len() as u64 {
                            return Err(VMError::InvalidInputOutputState);
                        }
                        //self.push_item(String::U64(num_state_variables));
                        //push input state variables
                        for (j, var) in input_state_variables.unwrap().iter().enumerate() {
                            self.push_item(var.clone());
                            self.push_item(out_state_variables[j].clone());
                        }
                    }
                    //}

                    //load Optional State script data if available
                    match input.input.as_state_script_data() {
                        None => (),
                        Some(script_data) => {
                            //load the script data vector on stack
                            for data in script_data.iter() {
                                self.push_item(data.clone());
                            }
                        }
                    }
                }
            }
        }
        // load tx data onto stack if present
        match &self.tx_data {
            None => (),
            Some(data) => self.push_item(data.clone()),
        }

        Ok(())
    }
    /// Runs through the entire program and nested programs until completion.
    pub fn run(mut self) -> Result<(), VMError> {
       // println!("stack len : {:?}", self.stack.len());
       // println!("Stack : {:?}", self.stack);
        loop {
            if !self.step()? {
                break;
            }
        }
        if self.stack.len() > 0 {
            return Err(VMError::StackNotClean);
        }

        Ok(())
    }

    fn finish_run(&mut self) -> bool {
        // Do we have more programs to run?
        if let Some(run) = self.run_stack.pop() {
            // Continue with the previously remembered program
            self.current_run = run;
            return true;
        }
        // Finish the execution
        return false;
    }

    /// Returns a flag indicating whether to continue the execution
    fn step(&mut self) -> Result<bool, VMError> {
        if let Some(instr) = self.delegate.next_instruction(&mut self.current_run)? {
            //println!("instr : {:?}", instr);
            // Attempt to read the next instruction and advance the program state
            match instr {
                Instruction::Push(data) => self.pushdata(data),
                Instruction::Program(prog) => self.pushprogram(prog),
                Instruction::Drop => self.drop()?,
                Instruction::Dup(i) => self.dup(i)?,
                Instruction::Roll(i) => self.roll(i)?,
                Instruction::Scalar => self.scalar()?,
                Instruction::Commit => self.commit()?,
                Instruction::Alloc(sw) => self.alloc(sw)?,
                Instruction::Expr => self.expr()?,
                Instruction::Neg => self.neg()?,
                Instruction::Add => self.add()?,
                Instruction::Mul => self.mul()?,
                Instruction::Eq => self.eq()?,
                Instruction::Range => self.range()?,
                Instruction::And => self.and()?,
                Instruction::Or => self.or()?,
                Instruction::Not => self.not()?,
                Instruction::Verify => self.verify()?,
                Instruction::Unblind => (), //self.unblind()?,
                Instruction::Issue => (),   //self.issue()?,
                Instruction::Borrow => self.borrow()?,
                Instruction::Retire => self.retire()?,
                Instruction::Fee => self.fee()?,
                Instruction::Input => self.input()?,
                Instruction::Output(k) => (),   //self.output(k)?,
                Instruction::Contract(k) => (), //self.contract(k)?,
                Instruction::Log => self.log()?,
                // Instruction::Call => (),    //self.call()?,
                // Instruction::Signtx => (),  //self.signtx()?,
                // Instruction::Signid => (),  //self.signid()?,
                // Instruction::Signtag => (), //self.signtag()?,
                Instruction::InputCoin(k) => self.inputcoin(k)?,
                Instruction::OutputCoin(k) => self.outputcoin(k)?,
                Instruction::Ext(opcode) => (), //self.ext(opcode)?,
            }
            return Ok(true);
        } else {
            // Reached the end of the current program
            return Ok(self.finish_run());
        }
    }

    fn pushdata(&mut self, str: String) {
        self.push_item(str);
    }

    fn pushprogram(&mut self, prog: ProgramItem) {
        self.push_item(prog);
    }

    fn drop(&mut self) -> Result<(), VMError> {
        let _dropped = self.pop_item()?.to_droppable()?;
        Ok(())
    }

    fn dup(&mut self, i: usize) -> Result<(), VMError> {
        if i >= self.stack.len() {
            return Err(VMError::StackUnderflow);
        }
        let item_idx = self.stack.len() - i - 1;
        let copied = self.stack[item_idx].dup_copyable()?;
        self.push_item(copied);
        //println!("stack len : {:?}", self.stack.len());
        //println!("Stack : {:?}", self.stack);
        Ok(())
    }

    fn roll(&mut self, i: usize) -> Result<(), VMError> {
        if i >= self.stack.len() {
            return Err(VMError::StackUnderflow);
        }
        let item = self.stack.remove(self.stack.len() - i - 1);
        self.push_item(item);
        Ok(())
    }

    fn expr(&mut self) -> Result<(), VMError> {
        let var = self.pop_item()?.to_variable()?;
        let expr = self.variable_to_expression(var)?;
        self.push_item(expr);
        //  println!("Stack : {:?}", self.stack);
        Ok(())
    }

    fn neg(&mut self) -> Result<(), VMError> {
        let expr = self.pop_item()?.to_expression()?;
        self.push_item(-expr);
        Ok(())
    }

    fn add(&mut self) -> Result<(), VMError> {
        let expr2 = self.pop_item()?.to_expression()?;
        let expr1 = self.pop_item()?.to_expression()?;
        let expr3 = expr1 + expr2;
        self.push_item(expr3);
        Ok(())
    }

    fn mul(&mut self) -> Result<(), VMError> {
        let expr2 = self.pop_item()?.to_expression()?;
        let expr1 = self.pop_item()?.to_expression()?;
        let expr3 = expr1.multiply(expr2, self.delegate.cs());
        self.push_item(expr3);
        Ok(())
    }

    fn eq(&mut self) -> Result<(), VMError> {
        let expr2 = self.pop_item()?.to_expression()?;
        let expr1 = self.pop_item()?.to_expression()?;
        let constraint = Constraint::eq(expr1, expr2);
        self.push_item(constraint);
        Ok(())
    }

    fn range(&mut self) -> Result<(), VMError> {
        let expr = self.pop_item()?.to_expression()?;
        //  println!("Expression: {:?}", expr);
        let res = self.add_range_proof(expr.clone())?;
        //  println!("add range: {:?}", res);
        self.push_item(expr);
        Ok(())
    }

    fn and(&mut self) -> Result<(), VMError> {
       // println!("Stack : {:?}", self.stack);
        let c2 = self.pop_item()?.to_constraint()?;
        let c1 = self.pop_item()?.to_constraint()?;
        let c3 = Constraint::and(c1, c2);
        self.push_item(c3);
        Ok(())
    }

    fn or(&mut self) -> Result<(), VMError> {
        let c2 = self.pop_item()?.to_constraint()?;
        let c1 = self.pop_item()?.to_constraint()?;
        let c3 = Constraint::or(c1, c2);
        self.push_item(c3);
        Ok(())
    }

    fn not(&mut self) -> Result<(), VMError> {
        let c1 = self.pop_item()?.to_constraint()?;
        let c2 = Constraint::not(c1);
        self.push_item(c2);
        Ok(())
    }

    fn verify(&mut self) -> Result<(), VMError> {
        let constraint = self.pop_item()?.to_constraint()?;
        constraint.verify(self.delegate.cs())?;
       // println!("Stack : {:?}", self.stack);
        Ok(())
    }

    fn unblind(&mut self) -> Result<(), VMError> {
        // Pop scalar `v` and commitment `V`
        let v_scalar = self.pop_item()?.to_string()?.to_scalar()?.to_scalar();
        let v_point = self.pop_item()?.to_string()?.to_commitment()?.to_point();

        self.delegate.batch_verifier().append(
            -v_scalar,
            iter::once(Scalar::one()),
            iter::once(v_point.decompress()),
        );

        // Push commitment item
        self.push_item(String::Opaque(v_point.as_bytes().to_vec()));
        Ok(())
    }

    fn scalar(&mut self) -> Result<(), VMError> {
        let scalar_witness = self.pop_item()?.to_string()?.to_scalar()?;
        self.push_item(Expression::constant(scalar_witness));
        Ok(())
    }

    fn commit(&mut self) -> Result<(), VMError> {
        let commitment = self.pop_item()?.to_string()?.to_commitment()?;
        let v = Variable { commitment };
        self.push_item(v);
        Ok(())
    }

    fn alloc(&mut self, sw: Option<ScalarWitness>) -> Result<(), VMError> {
        let var = self
            .delegate
            .cs()
            .allocate(sw.map(|s| s.to_scalar()))
            .map_err(|e| VMError::R1CSError(e))?;
        let expr = Expression::LinearCombination(vec![(var, Scalar::one())], sw);
        self.push_item(expr);
        Ok(())
    }

    fn log(&mut self) -> Result<(), VMError> {
        let _data = self.pop_item()?.to_string()?;
        // self.txlog.push(TxEntry::Data(data.to_bytes()));
        Ok(())
    }

    /// _qty flv data pred_ **issue** → _contract_
    fn issue(&mut self) -> Result<(), VMError> {
        let predicate = self.pop_item()?.to_string()?.to_predicate()?;
        let metadata = self.pop_item()?.to_string()?;
        let flv = self.pop_item()?.to_variable()?;
        let qty = self.pop_item()?.to_variable()?;

        let (flv_point, _) = self.delegate.commit_variable(&flv.commitment)?;
        let (qty_point, _) = self.delegate.commit_variable(&qty.commitment)?;

        let flv_scalar = Value::issue_flavor(&predicate, metadata);
        // flv_point == flavor·B    ->   0 == -flv_point + flv_scalar·B
        self.delegate.batch_verifier().append(
            flv_scalar,
            iter::once(-Scalar::one()),
            iter::once(flv_point.decompress()),
        );

        let value = Value {
            qty: qty.commitment.clone(),
            flv: flv.commitment,
        };

        let qty_expr = self.variable_to_expression(qty)?;
        self.add_range_proof(qty_expr)?;

        // self.txlog.push(TxEntry::Issue(qty_point, flv_point));

        let payload = vec![PortableItem::Value(value)];
        let contract = self.make_contract(predicate, payload)?;

        self.push_item(contract);
        Ok(())
    }

    fn borrow(&mut self) -> Result<(), VMError> {
        let flv = self.pop_item()?.to_variable()?;
        let qty = self.pop_item()?.to_variable()?;

        let (_, flv_var) = self.delegate.commit_variable(&flv.commitment)?;
        let (_, qty_var) = self.delegate.commit_variable(&qty.commitment)?;
        let flv_assignment = flv.commitment.assignment().map(|sw| sw.to_scalar());
        let qty_assignment = ScalarWitness::option_to_integer(qty.commitment.assignment())?;

        rangeproof::range_proof(
            self.delegate.cs(),
            qty_var.into(),
            qty_assignment,
            BitRange::max(),
        )
        .map_err(|_| VMError::R1CSInconsistency)?;

        let neg_qty_assignment = qty_assignment.map(|q| -q);

        let neg_qty_var = self
            .delegate
            .cs()
            .allocate(neg_qty_assignment.map(|q| q.to_scalar()))
            .map_err(|e| VMError::R1CSError(e))?;

        self.delegate.cs().constrain(qty_var + neg_qty_var);

        let value = Value {
            qty: qty.commitment.clone(),
            flv: flv.commitment,
        };
        let wide_value = WideValue(rangeproof::AllocatedValue {
            q: neg_qty_var,
            f: flv_var,
            assignment: match (neg_qty_assignment, flv_assignment) {
                (Some(q), Some(f)) => Some(rangeproof::Value { q, f }),
                _ => None,
            },
        });
        self.push_item(wide_value);
        self.push_item(value);
        Ok(())
    }

    fn retire(&mut self) -> Result<(), VMError> {
        let _value = self.pop_item()?.to_value()?;
        // self.txlog
        //   .push(TxEntry::Retire(value.qty.into(), value.flv.into()));
        Ok(())
    }


    // _value qty_ **fee** → ø
    fn fee(&mut self) -> Result<(), VMError> {
        let fee = self.pop_item()?.to_string()?.to_u32()? as u64;
        let fee_scalar = Scalar::from(fee);
        // self.total_fee = self.total_fee.add(fee).ok_or(VMError::FeeTooHigh)?;

        let v = rangeproof::Value {
            q: -rangeproof::SignedInteger::from(fee),
            f: fee_flavor(),
        };

        let av = v
            .allocate(self.delegate.cs())
            .map_err(|e| VMError::R1CSError(e))?;

        self.delegate.cs().constrain(av.q + fee_scalar);
        self.delegate.cs().constrain(av.f - fee_flavor());

        self.push_item(WideValue(av));

        //self.txlog.push(TxEntry::Fee(fee));
        Ok(())
    }

   
    fn inputcoin(&mut self, i: usize) -> Result<(), VMError> {
        //retrieve input coin from index i in inputs
        let input: &Input = &self.inputs_tx[i];
        //check if input is input_coin
        let check = input.in_type.is_coin();
        if check {
            let encrypt = input
                .input
                .as_encryption()
                .ok_or(VMError::InvalidInputCoin)?;
            let owner = input.input.owner().unwrap();

            let c = OutputCoin {
                encrypt: encrypt.clone(),
                owner: owner.clone(),
            };
            self.push_item(c);
            Ok(())
        } else {
            Err(VMError::InvalidInputCoin)
        }
    }
    fn outputcoin(&mut self, i: usize) -> Result<(), VMError> {
        //retrieve input coin from index i in inputs
        let out: &Output = &self.outputs_tx[i];
        //check if input is input_coin
        let check = out.out_type.is_coin();
        if check == true {
            let encrypt = out
                .output
                .get_encryption()
                .ok_or(VMError::InvalidOutputCoin)?;
            let owner = out.output.get_owner_address().unwrap();

            let c = OutputCoin {
                encrypt: encrypt.clone(),
                owner: owner.clone(),
            };
            self.push_item(c);
            Ok(())
        } else {
            Err(VMError::InvalidOutputCoin)
        }
    }

}

// Utility methods
impl<'d, CS, R> VMScript<'d, CS, R>
where
    CS: r1cs::RandomizableConstraintSystem,
    R: VMRun<CS>,
{
    fn pop_item(&mut self) -> Result<Item, VMError> {
        self.stack.pop().ok_or(VMError::StackUnderflow)
    }

    fn push_item<T>(&mut self, item: T)
    where
        T: Into<Item>,
    {
        self.stack.push(item.into())
    }

    fn variable_to_expression(&mut self, var: Variable) -> Result<Expression, VMError> {
        let (_, r1cs_var) = self.delegate.commit_variable(&var.commitment)?;

        Ok(Expression::LinearCombination(
            vec![(r1cs_var, Scalar::one())],
            var.commitment.assignment(),
        ))
    }

    fn continue_with_program(&mut self, prog: ProgramItem) -> Result<(), VMError> {
        let new_run = self.delegate.new_run(prog)?;
        let paused_run = mem::replace(&mut self.current_run, new_run);
        self.run_stack.push(paused_run);
        Ok(())
    }

    fn add_range_proof(&mut self, expr: Expression) -> Result<(), VMError> {
        match expr {
            Expression::Constant(x) => {
                if x.in_range() {
                    Ok(())
                } else {
                    Err(VMError::InvalidBitrange)
                }
            }
            Expression::LinearCombination(terms, assignment) => rangeproof::range_proof(
                self.delegate.cs(),
                r1cs::LinearCombination::from_iter(terms),
                ScalarWitness::option_to_integer(assignment)?,
                BitRange::max(),
            )
            .map_err(|_| VMError::R1CSInconsistency),
        }
    }
}
