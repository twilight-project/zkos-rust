This is the specification for ZkVm, the zero-knowledge stack based virtual machine. ZkVM is inspired and extended from [Slingshot/Zkvm](https://github.com/stellar/slingshot/edit/main/zkvm/docs/zkvm-spec.md#stack-instructions) 

ZkVM defines a procedural representation for blockchain transaction proof verification and the rules for a virtual machine to interpret them and ensure their validity.

* [Overview](#overview)
    * [Concepts](#concepts)
* [Types](#types)
    * [Linear types](#linear-types)
    * [Program](#program-type)
    * [Variable](#variable-type)
    * [Expression](#expression-type)
    * [Constraint](#constraint-type)
    * [Value](#value-type)
    * [Wide value](#wide-value-type)
* [Definitions](#definitions)
    * [LE32](#le32)
    * [LE64](#le64)
    * [Scalar](#scalar-value)
    * [Point](#point)
    * [Base points](#base-points)
    * [Pedersen commitment](#pedersen-commitment)
    * [Transcript](#transcript)
    * [Constraint system](#constraint-system)
    * [Constraint system proof](#constraint-system-proof)
 * [VM operation](#vm-operation)
    * [VM state](#vm-state)
    * [VM execution](#vm-execution)
    * [Deferred point operations](#deferred-point-operations)
* [Instructions](#instructions)
    * [Stack instructions](#stack-instructions)
    * [Constraint system instructions](#constraint-system-instructions)
    * [Value instructions](#value-instructions)


### Concepts

A ZkOS Script transaction is represented by a [transaction](add link here for tx) object that
contains a [program](#program-type) that runs in the context of a stack-based virtual machine.

When the virtual machine executes a program, it manipulates data of various types to create a zero knowledge proof of the computations.

A [**value**](#value-type) is a specific _quantity_ of a certain _flavor_ that can be
merged or split, issued or retired, but not otherwise created or destroyed. The value can be loaded from global state during stack initialization process. 

Custom logic is represented via programmable [**constraints**](#constraint-type)
applied to [**variables**](#variable-type) and [**expressions**](#expression-type)
(linear combinations of variables). Variables represent quantities and flavors of values,
and user-defined secret parameters. All constraints are arranged in
a single [constraint system](#constraint-system) which is proven to be satisfied after the VM
has finished execution.

A ZkVM proof is valid if and only if it runs to completion
without encountering failure conditions and without leaving any data
on the stack.

## Types

The items on the ZkVM stack are typed. 

### Linear types

Linear types are subject to special rules as to when and how they may be created
and destroyed, and may never be copied.

* [Program](#program-type)
* [Expression](#expression-type)
* [Constraint](#constraint-type)
* [Contract](#contract-type)
* [Wide value](#wide-value-type)
* [Value](#value-type)

### String type

A _string_ is a variable-length byte array used to represent [commitments](#pedersen-commitment), [scalars](#scalar-value), signatures, and proofs.

A string cannot be larger than the entire transaction program and cannot be longer than `2^32-1` bytes (see [LE32](#le32)).


### Program type

A _program type_ is a variable-length byte array representing a sequence of ZkVM [instructions](#instructions).

Program cannot be larger than the entire transaction program and cannot be longer than `2^32-1` bytes (see [LE32](#le32)).

### Variable type

_Variable_ represents a secret [scalar](#scalar-value) value in the [constraint system](#constraint-system)
bound to its [Pedersen commitment](#pedersen-commitment).

A [point](#point) that represents a commitment to a secret scalar can be turned into a variable using the [`commit`](#commit) instruction.

A cleartext [scalar](#scalar-value) can be turned into a single-term [expression](#expression-type) using the [`scalar`](#scalar) instruction (which does not allocate a variable). Since we do not need to hide their values, a Variable is not needed to represent the cleartext constant.

Variables can be copied and dropped at will.

[Value quantities and flavors](#value-type) are represented as variables.

Constraint system also contains _low-level variables_ that are not individually bound to [Pedersen commitments](#pedersen-commitment):
when these are exposed to the VM (for instance, from [`mul`](#mul)), they have the [expression type](#expression-type).


### Expression type

_Expression_ is a linear combination of constraint system variables with cleartext [scalar](#scalar-value) weights.

    expr = { (weight0, var0), (weight1, var1), ...  }

A [variable](#variable-type) can be converted to an expression using [`expr`](#expr):
the result is a linear combination with one term with weight 1:

expr = { (1, var) }

Expressions can be [added](#add) and [multiplied](#mul), producing new expressions.


### Constant expression

An [expression](#expression-type) that contains one term with the [scalar](#scalar-value) weight assigned to the R1CS `1` is considered
a _constant expression_:

    const_expr = { (weight, 1) }

Instructions [`add`](#add) and [`mul`](#mul) preserve constant expressions as an optimization in order to avoid
allocating unnecessary multipliers in the [constraint system](#constraint-system).


### Constraint type

_Constraint_ is a statement within the [constraint system](#constraint-system). Constraints are formed using [expressions](#expression-type)
and can be combined using logical operators [`and`](#and) and [`or`](#or).

There are three kinds of constraints:
1. **Linear constraint** is created using the [`eq`](#eq) instruction over two [expressions](#expression-type).
2. **Conjunction constraint** is created using the [`and`](#and) instruction over two constraints of any type.
3. **Disjunction constraint** is created using the [`or`](#or) instruction over two constraints of any type.
4. **Inversion constraint** is created using the [`not`](#not) instruction over a constraint of any type.
5. **Cleartext constraint** is created as a result of _guaranteed optimization_ of the above instructions when executed with [constant expressions](#constant-expression). Cleartext constraint contains a cleartext boolean `true` or `false`.

Constraints only have an effect if added to the constraint system using the [`verify`](#verify) instruction.


### Value type

A value is a [linear type](#linear-types) representing a pair of *quantity* and *flavor*.
Both quantity and flavor are represented as [variables](#variable-type).
Quantity is guaranteed to be in a 64-bit range (`[0..2^64-1]`).

Values are created with [`issue`](#issue) and destroyed with [`retire`](#retire).

[`borrow`](#borrow) instruction produces two items: a non-negative value and a negated [wide value](#wide-value-type),
which must be cleared using appropriate combination of non-negative values.


### Wide value type

_Wide value_ is an extension of the [value type](#value-type) where
quantity is guaranteed to be in a wider, 65-bit range `[-(2^64-1) .. 2^64-1]`.

The subtype [Value](#value-type) is most commonly used because it guarantees the non-negative quantity
and the wide value is only used as an output of [`borrow`](#borrow) when a negative value representation is needed.

## Definitions

### LE32

A non-negative 32-bit integer encoded using little-endian convention.
Used to encode lengths of [strings](#string-type),  e.g., stack indices.

### LE64

A non-negative 64-bit integer encoded using little-endian convention.
Used to encode [value quantities](#value-type).


### Scalar value

A _scalar_ is an integer modulo [Ristretto group](https://ristretto.group) order `|G| = 2^252 + 27742317777372353535851937790883648493`.

Scalars are encoded as 32-byte [strings](#string-type) using little-endian convention.

Every scalar in the VM is guaranteed to be in a canonical (reduced) form: an instruction that operates on a scalar
checks if the scalar is canonical.


### Point

A _point_ is an element in the [Ristretto group](https://ristretto.group).

Points are encoded as 32-byte [strings](#string-type) in _compressed Ristretto form_.

Each point in the VM is guaranteed to be a valid Ristretto point.


### Base points

ZkVM defines two base points: primary `B` and secondary `B2`.

```
B  = e2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d76
B2 = hash-to-ristretto255(SHA3-512(B))
```

Both base points are orthogonal (the discrete log between them is unknown)
and used in [Pedersen commitments](#pedersen-commitment).


### Pedersen commitment

Pedersen commitment to a secret [scalar](#scalar-value)
is defined as a point with the following structure:

```
P = Com(v, f) = v·B + f·B2
```

where:

* `P` is a point representing commitment,
* `v` is a secret scalar value being committed to,
* `f` is a secret blinding factor (scalar),
* `B` and `B2` are [base points](#base-points).

Pedersen commitments can be used to allocate new [variables](#variable-type) using the [`commit`](#commit) instruction.

Pedersen commitments can be opened using the [`unblind`](#unblind) instruction.


### Transcript

Transcript is an instance of the [Merlin](https://doc.dalek.rs/merlin/) construction,
which is itself based on [STROBE](https://strobe.sourceforge.io/) and [Keccak-f](https://keccak.team/keccak.html)
with 128-bit security parameter.

Transcript is used throughout ZkVM to generate challenge [scalars](#scalar-value) and commitments.

Transcripts have the following operations, each taking a label for domain separation:

1. **Initialize** transcript:
    ```
    T := Transcript(label)
    ```
2. **Append bytes** of arbitrary length prefixed with a label:
    ```
    T.append(label, bytes)
    ```
3. **Challenge bytes**
    ```    
    T.challenge_bytes<size>(label) -> bytes
    ```
4. **Challenge scalar** is defined as generating 64 challenge bytes and reducing the 512-bit little-endian integer modulo Ristretto group order `|G|`:
    ```    
    T.challenge_scalar(label) -> scalar
    T.challenge_scalar(label) == T.challenge_bytes<64>(label) mod |G|
    ```

Labeled instances of the transcript can be precomputed
to reduce number of Keccak-f permutations to just one per challenge.


### Constraint system

The constraint system is the part of the [VM state](#vm-state) that implements
[Bulletproof's rank-1 constraint system](https://doc-internal.dalek.rs/develop/bulletproofs/notes/r1cs_proof/index.html).

It also keeps track of the [variables](#variable-type) and [constraints](#constraint-type),
and is used to verify the [constraint system proof](#constraint-system-proof).


### Constraint system proof

A proof of satisfiability of a [constraint system](#constraint-system) built during the VM execution.

The proof is provided to the VM at the beginning of execution and verified when the VM is [finished](#vm-execution).


### Merkle binary tree

The construction of a merkle binary tree is based on the [RFC 6962 Section 2.1](https://tools.ietf.org/html/rfc6962#section-2.1)
with hash function replaced with a [transcript](#transcript).

Leafs and nodes in the tree use the same instance of a transcript provided by the upstream protocol:

```
T = Transcript(<label>)
```

The hash of an empty list is a 32-byte challenge string with the label `merkle.empty`:

```
MerkleHash(T, {}) = T.challenge_bytes("merkle.empty")
```

The hash of a list with one entry (also known as a leaf hash) is
computed by committing the entry to the transcript (defined by the item type),
and then generating 32-byte challenge string the label `merkle.leaf`:

```
MerkleHash(T, {item}) = {
    T.append(<field1 name>, item.field1)
    T.append(<field2 name>, item.field2)
    ...
    T.challenge_bytes("merkle.leaf")
}
```

For n > 1, let k be the largest power of two smaller than n (i.e., k < n ≤ 2k). The merkle hash of an n-element list is then defined recursively as:

```
MerkleHash(T, list) = {
    T.append("L", MerkleHash(list[0..k]))
    T.append("R", MerkleHash(list[k..n]))
    T.challenge_bytes("merkle.node")
}
```

Note that we do not require the length of the input list to be a power of two.
The resulting merkle binary tree may thus not be balanced; however,
its shape is uniquely determined by the number of leaves.

The Merkle binary tree is used to construct a Contract 

## VM operation

### VM state

The ZkVM state consists of the static attributes and the state machine attributes.

1. [Script Transaction](add link here):
    * `program`
    * `cs_proof`
    * `Input`
    * `Output`
2. Data stack (array of [items](#types))
3. Program stack (array of [programs](#program-type) with their offsets)
4. Current [program](#program-type) with its offset
5. [Deferred point operations](#deferred-point-operations)
6. [Constraint system](#constraint-system)


### VM execution

The VM is initialized with the following state:

1. [Transaction](#....) as provided by the user.
2. Data stack is loaded with
   * Parse the [Inputs](#...) and the corresponding [Outputs](#...) to detect types
   * If the `Input` is `Coin`, load the data stack with corresponding `Memo` Output
   * If the `Input` is `Memo`, the stack is loaded with the input values and data
   * If the `Input` is `State`, load stack with `OutputState`(#...) followed by `InputState`(#...)
   * `tx_data` is loaded if present. 

3. Current program set to the transaction program; with zero offset.
4. Array of deferred point operations is empty.
5. Constraint system: empty, with [transcript](#transcript) initialized with label `ZkVM.r1cs`:
    ```
    r1cs_transcript = Transcript("ZkVM.r1cs")
    ```

Then, the VM executes the current program till completion:

1. Each instruction is read at the current program offset, including its immediate data (if any).
2. Program offset is advanced immediately after reading the instruction to the next instruction.
3. The instruction is executed per [specification below](#instructions). If the instruction fails, VM exits early with an error result.
4. If the offset is less than the current program’s length, a new instruction is read (go back to step 1).
5. If the program stack is empty, the execution is considered _finalized_ and VM successfully exits.

If the execution finishes successfully, VM performs the finishing tasks:

1. Checks if the stack is empty; fails otherwise.
2. Computes a verification statement for [constraint system proof](#constraint-system-proof).
3. Executes all [deferred point operations](#deferred-point-operations) using a single multi-scalar multiplication. Fails if the result is not an identity point.


### Deferred point operations

VM defers operations on [points](#point) till the end of the VM execution in order
to batch them with the verification of [constraint system proof](#constraint-system-proof).

Each deferred operation at index `i` represents a statement:
```
0  ==  sum{s[i,j]·P[i,j], for all j}  +  a[i]·B  +  b[i]·B2
```
where:
1. `{s[i,j],P[i,j]}` is an array of ([scalar](#scalar-value),[point](#point)) tuples,
2. `a[i]` is a [scalar](#scalar-value) weight of a [primary base point](#base-points) `B`,
3. `b[i]` is a [scalar](#scalar-value) weight of a [secondary base point](#base-points) `B2`.

All such statements are combined using the following method:

1. For each statement, a random [scalar](#scalar-value) `x[i]` is sampled.
2. Each weight `s[i,j]` is multiplied by `x[i]` for all weights per statement `i`:
    ```
    z[i,j] = x[i]·s[i,j]
    ```
3. All weights `a[i]` and `b[i]` are independently added up with `x[i]` factors:
    ```
    a = sum{a[i]·x[i]}
    b = sum{b[i]·x[i]}
    ```
4. A single multi-scalar multiplication is performed to verify the combined statement:
    ```
    0  ==  sum{z[i,j]·P[i,j], for all i,j}  +  a·B  +  b·B2
    ```

## Instructions

Each instruction is represented by a one-byte **opcode** optionally followed by **immediate data**.
Immediate data is denoted by a colon `:` after the instruction name.

Each instruction defines the format for immediate data. See the reference below for detailed specification.

Code | Instruction                | Stack diagram                              | Effects
-----|----------------------------|--------------------------------------------|----------------------------------
 |     [**Stack**](#stack-instructions)               |                        |
0x00 | [`push:n:x`](#push)        |                 ø → _data_                 |
0x01 | [`drop`](#drop)            |               _x_ → ø                      |
0x02 | [`dup:k`](#dup)            |     _x[k] … x[0]_ → _x[k] ... x[0] x[k]_   |
0x03 | [`roll:k`](#roll)          |     _x[k] … x[0]_ → _x[k-1] ... x[0] x[k]_ |
 |                                |                                            |
 |     [**Constraints**](#constraint-system-instructions)  |                   | 
0x05 | [`scalar`](#scalar)        |          _scalar_ → _expr_                 | 
0x06 | [`commit`](#commit)        |           _point_ → _var_                  | Adds an external variable to [CS](#constraint-system)
0x07 | [`alloc`](#alloc)          |                 ø → _expr_                 | Allocates a low-level variable in [CS](#constraint-system)
0x0a | [`expr`](#expr)            |             _var_ → _expr_                 | Allocates a variable in [CS](#constraint-system)
0x0b | [`neg`](#neg)              |           _expr1_ → _expr2_                |
0x0c | [`add`](#add)              |     _expr1 expr2_ → _expr3_                |
0x0d | [`mul`](#mul)              |     _expr1 expr2_ → _expr3_                | Potentially adds multiplier in [CS](#constraint-system)
0x0e | [`eq`](#eq)                |     _expr1 expr2_ → _constraint_           | 
0x0f | [`range`](#range)          |            _expr_ → _expr_                 | Modifies [CS](#constraint-system)
0x10 | [`and`](#and)              | _constr1 constr2_ → _constr3_              |
0x11 | [`or`](#or)                | _constr1 constr2_ → _constr3_              |
0x12 | [`not`](#not)              |         _constr1_ → _constr2_              | Modifies [CS](#constraint-system)
0x13 | [`verify`](#verify)        |      _constraint_ → ø                      | Modifies [CS](#constraint-system) 
0x14 | [`unblind`](#unblind)      |             _V v_ → _V_                    | [Defers point ops](#deferred-point-operations)
 |                                |                                            |
 |     [**Values**](#value-instructions)              |                        |
0x15 | [`issue`](#issue)          |    _qty flv data_ → ø                      | Modifies [CS](#constraint-system), [defers point ops](#deferred-point-operations)
0x16 | [`borrow`](#borrow)        |         _qty flv_ → _–V +V_                | Modifies [CS](#constraint-system)
0x17 | [`retire`](#retire)        |           _value_ → ø                      | Modifies [CS](#constraint-system), 
0x18 | [`fee`](#fee)              |             _qty_ → _widevalue_            | Modifies [CS](#constraint-system), 



 ### Stack instructions

#### push

**push:_n_:_x_** → _data_

Pushes a [string](#string-type) `x` containing `n` bytes. 
Immediate data `n` is encoded as [LE32](#le32)
followed by `x` encoded as a sequence of `n` bytes.


#### drop

_x_ **drop** → ø

Drops `x` from the stack.

Fails if `x` is not a [droppable type](#droppable-types).


#### dup

_x[k] … x[0]_ **dup:_k_** → _x[k] ... x[0] x[k]_

Copies k’th item from the top of the stack.
Immediate data `k` is encoded as [LE32](#le32).

Fails if `x[k]` is not a [copyable type](#copyable-types).


#### roll

_x[k] x[k-1] ... x[0]_ **roll:_k_** → _x[k-1] ... x[0] x[k]_

Looks past `k` items from the top, and moves the next item to the top of the stack.
Immediate data `k` is encoded as [LE32](#le32).

Note: `roll:0` is a no-op, `roll:1` swaps the top two items.




### Constraint system instructions

#### scalar

_a_ **scalar** → _expr_

1. Pops a [scalar](#scalar-value) `a` from the stack.
2. Creates an [expression](#expression-type) `expr` with weight `a` assigned to an R1CS constant `1`.
3. Pushes `expr` to the stack.

Fails if `a` is not a valid [scalar](#scalar-value).

#### commit

_P_ **commit** → _v_

1. Pops a [point](#point) `P` from the stack.
2. Creates a [variable](#variable-type) `v` from a [Pedersen commitment](#pedersen-commitment) `P`.
3. Pushes `v` to the stack.

Fails if `P` is not a valid [point](#point).

#### alloc

**alloc** → _expr_

1. Allocates a low-level variable in the [constraint system](#constraint-system) and wraps it in the [expression](#expression-type) with weight 1.
2. Pushes the resulting expression to the stack.

This is different from [`commit`](#commit): the variable created by `alloc` is _not_ represented by an individual Pedersen commitment and therefore can be chosen freely when the transaction is constructed.


#### expr

_var_ **expr** → _ex_

1. Pops a [variable](#variable-type) `var`.
2. Allocates a high-level variable in the constraint system using its Pedersen commitment.
3. Pushes a single-term [expression](#expression-type) with weight=1 to the stack: `expr = { (1, var) }`.

Fails if `var` is not a [variable type](#variable-type).

#### neg

_ex1_ **neg** → _ex2_

1. Pops an [expression](#expression-type) `ex1`.
2. Negates the weights in the `ex1` producing new expression `ex2`.
3. Pushes `ex2` to the stack.

Fails if `ex1` is not an [expression type](#expression-type).

#### add

_ex1 ex2_ **add** → ex3_

1. Pops two [expressions](#expression-type) `ex2`, then `ex1`.
2. If both expressions are [constant expressions](#constant-expression):
    1. Creates a new [constant expression](#constant-expression) `ex3` with the weight equal to the sum of weights in `ex1` and `ex2`.
3. Otherwise, creates a new expression `ex3` by concatenating terms in `ex1` and `ex2`.
4. Pushes `ex3` to the stack.

Fails if `ex1` and `ex2` are not both [expression types](#expression-type).

#### mul

_ex1 ex2_ **mul** → _ex3_

Multiplies two [expressions](#expression-type) producing another [expression](#expression-type) representing the result of multiplication.

This performs a _guaranteed optimization_: if one of the expressions `ex1` or `ex2` contains
only one term and this term is for the variable representing the R1CS constant `1`
(in other words, the statement is a cleartext constant),
then the other expression is multiplied by that constant in-place without allocating a multiplier in the [constraint system](#constraint-system).

This optimization is _guaranteed_ because it affects the state of the constraint system:
not performing it would make the existing proofs invalid.

1. Pops two [expressions](#expression-type) `ex2`, then `ex1`.
2. If either `ex1` or `ex2` is a [constant expression](#constant-expression):
    1. The other expression is multiplied in place by the scalar from that expression.
    2. The resulting expression is pushed to the stack.
3. Otherwise:
    1. Creates a multiplier in the constraint system.
    2. Constrains the left wire to `ex1`, and the right wire to `ex2`.
    3. Creates an [expression](#expression-type) `ex3` with the output wire in its single term.
    4. Pushes `ex3` to the stack.

Fails if `ex1` and `ex2` are not both [expression types](#expression-type).

Note: if both `ex1` and `ex2` are [constant expressions](#constant-expression),
the result does not depend on which one treated as a constant,
and the resulting expression is also a constant expression.

#### eq

_ex1 ex2_ **eq** → _constraint_

1. Pops two [expressions](#expression-type) `ex2`, then `ex1`.
2. If both `ex1` or `ex2` are [constant expressions](#constant-expression):
    1. Creates a [cleartext constraint](#constraint-type) with a boolean `true` if the weights are equal, `false` otherwise.
3. Otherwise:
    1. Creates a [constraint](#constraint-type) that represents statement `ex1 - ex2 = 0`.
4. Pushes the constraint to the stack.

Fails if `ex1` and `ex2` are not both [expression types](#expression-type).

#### range

_expr_ **range** → _expr_

1. Pops an [expression](#expression-type) `expr`.
2. Adds an 64-bit range proof for `expr` to the [constraint system](#constraint-system).
3. Pushes `expr` back to the stack.

Fails if `expr` is not an [expression type](#expression-type).

#### and

_c1 c2_ **and** → _c3_

1. Pops [constraints](#constraint-type) `c2`, then `c1`.
2. If either `c1` or `c2` is a [cleartext constraint](#constraint-type):
    1. If the cleartext constraint is `false`, returns it; otherwise returns the other constraint.
3. Otherwise:
    1. Creates a _conjunction constraint_ `c3` containing `c1` and `c2`.
3. Pushes `c3` to the stack.

No changes to the [constraint system](#constraint-system) are made until [`verify`](#verify) is executed.

Fails if `c1` and `c2` are not [constraints](#constraint-type).

#### or

_constraint1 constraint2_ **or** → _constraint3_

1. Pops [constraints](#constraint-type) `c2`, then `c1`.
2. If either `c1` or `c2` is a [cleartext constraint](#constraint-type):
    1. If the cleartext constraint is `true`, returns it; otherwise returns the other constraint.
3. Otherwise:
    1. Creates a _disjunction constraint_ `c3` containing `c1` and `c2`.
3. Pushes `c3` to the stack.

No changes to the [constraint system](#constraint-system) are made until [`verify`](#verify) is executed.

Fails if `c1` and `c2` are not [constraints](#constraint-type).

#### not

_constr1_ **not** → _constr2_

1. Pops [constraint](#constraint-type) `c1`.
2. If `c1` is a [cleartext constraint](#constraint-type), returns its negation.
3. Otherwise:
    1. Create two constraints:
       ```
       x * y = 0
       x * w = 1-y
       ```
       where `w` is a free variable and `x` is the evaluation of constraint `c1`.
    2. Wrap the output `y` in a constraint `c2`.
    3. Push `c2` to the stack.

This implements the boolean `not` trick from [Setty, Vu, Panpalia, Braun, Ali, Blumberg, Walfish (2012)](https://eprint.iacr.org/2012/598.pdf) and implemented in [libsnark](https://github.com/scipr-lab/libsnark/blob/dfa74ff270ca295619be1fdf7661f76dff0ae69e/libsnark/gadgetlib1/gadgets/basic_gadgets.hpp#L162-L169).


#### verify

_constr_ **verify** → ø

1. Pops [constraint](#constraint-type) `constr`.
2. If `constr` is a [cleartext constraint](#constraint-type):
    1. If it is `true`, returns immediately.
    2. If it is `false`, fails execution.
3. Otherwise, transforms the constraint `constr` recursively using the following rules:
    1. Replace conjunction of two _linear constraints_ `a` and `b` with a linear constraint `c` by combining both constraints with a random challenge `z`:
        ```
        z = transcript.challenge_scalar(b"ZkVM.verify.and-challenge");
        c = a + z·b
        ```
    2. Replace disjunction of two _linear constraints_ `a` and `b` by constrainting an output `o` of a newly allocated multiplier `{r,l,o}` to zero, while adding constraints `r == a` and `l == b` to the constraint system.
        ```
        r == a # added to CS
        l == b # added to CS
        o == 0 # replaces OR(a,b)
        ```
    3. Conjunctions and disjunctions of non-linear constraints are transformed via rules (1) and (2) using depth-first recursion.
3. The resulting single linear constraint is added to the constraint system.

Fails if `constr` is not a [constraint](#constraint-type).


#### unblind

_V v_ **unblind** → _V_

1. Pops [scalar](#scalar-value) `v`.
2. Pops [point](#point) `V`.
3. Verifies the [unblinding proof](#unblinding-proof) for the commitment `V` and scalar `v`, [deferring all point operations](#deferred-point-operations)).
4. Pushes [point](#point) `V`.

Fails if: 
* `v` is not a valid [scalar](#scalar-value), or
* `V` is not a valid [point](#point), or



### Value instructions

#### issue

_qty flv metadata_ **issue** → ø

1. Pops [string](#string-type) `metadata`.
2. Pops [variable](#variable-type) `flv` and commits it to the constraint system.
3. Pops [variable](#variable-type) `qty` and commits it to the constraint system.
4. Creates a [value](#value-type) with variables `qty` and `flv` for quantity and flavor, respectively. 
5. Computes the _flavor_ scalar using the following [transcript-based](#transcript) protocol:
    ```
    T = Transcript("ZkVM.issue")
    T.append("metadata", metadata)
    flavor = T.challenge_scalar("flavor")
    ```
6. Checks that the `flv` has unblinded commitment to `flavor`
   by [deferring the point operation](#deferred-point-operations):
    ```
    flv == flavor·B
    ```
7. Adds a 64-bit range proof for the `qty` to the [constraint system](#constraint-system).

This creats a proof of value creation into the contract that must be unlocked by providing the signatrure to the owner adrress defined in the output.

Fails if:
* `flv` or `qty` are not [variable types](#variable-type).

#### borrow

_qty flv_ **borrow** → _–V +V_

1. Pops [variable](#variable-type) `flv` and commits it to the constraint system.
2. Pops [variable](#variable-type) `qty` and commits it to the constraint system.
3. Creates a [value](#value-type) `+V` with variables `qty` and `flv` for quantity and flavor, respectively.
4. Adds a 64-bit range proof for `qty` variable to the [constraint system](#constraint-system).
5. Creates [wide value](#wide-value-type) `–V`, allocating a low-level variable `qty2` for the negated quantity and reusing the flavor variable `flv`.
6. Adds a constraint `qty2 == -qty` to the constraint system.
7. Pushes `–V`, then `+V` to the stack.

Fails if `qty` and `flv` are not [variable types](#variable-type).


#### retire

_value_ **retire** → ø

1. Pops a [value](#value-type) from the stack.

This creates a proof of retiring an asset. 
Fails if the value is not a [non-negative value type](#value-type).

#### fee

_qty_ **fee** → _widevalue_

1. Pops an 4-byte [string](#string-type) `qty` from the stack and decodes it as [LE32](#le64) integer.
2. Checks that `qty`  and accumulated fee is less or equal to `2^24`.
3. Pushes [wide value](#wide-value-type) `–V`, with quantity variable constrained to `-qty` and with flavor constrained to 0.
   Both variables are allocated from a single multiplier.

Fails if the resulting amount of fees is exceeding `2^24`.

All unassigned instruction codes are interpreted as no-ops. This are reserved for use in the future versions of the VM.
