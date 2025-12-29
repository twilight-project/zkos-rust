# ZkVM Specification: Instructions (VM Opcodes)

> Part of the ZkVM Specification.  
> &laquo; [Definitions](./DEFINITIONS.md) | [Home](./README.md) | [Next: Operations &raquo;](./OPERATIONS.md)
---

This document provides a complete reference for the ZkVM instruction set. Each instruction is detailed with its opcode, stack diagram, and operational effects, providing a guide for both VM implementers and program authors.

## Stack Instructions

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
(#constraint-system), 



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

All unassigned instruction codes are interpreted as no-ops. This are reserved for use in the future versions of the VM.

---
> &laquo; [Definitions](./DEFINITION.md) | [Home](./README.md) | [Next: Operations &raquo;](./OPERATIONS.md)