# Range Proof 

_RangeProof_ is a gadget for generating R1CS constrainted range proof assets based on the [Bulletproofs](https://crypto.stanford.edu/bulletproofs/) zero-knowledge proof system.

* [Requirements](#requirements)
* [Definitions](#definitions)
    * [Range proof](#range-proof)
* [Converting boolean expressions](#converting-boolean-expressions)



## Definitions

### Range proof

Proves that a given quantity is in a valid range using a binary representation:
the quantity is a sum of all bits in its bit-representation multiplied by corresponding powers of two, and each bit has either 0 or 1 value.

`n` multipliers `a_i*b_i = c_i` and `1 + 2*n` constraints:

    c_i == 0           for i in [0,n)  // n constraints on multipliers’ outputs
    a_i == (1 - b_i)   for i in [0,n)  // n constraints on multipliers’ inputs
    q = Sum(b_i * 2^i, i = 0..n-1)     // 1 constraint between quantity and the multipliers’ inputs

where:

* `b_i` is a bit and a left input to the `i`th multiplier.
* `a_i` is a right input to an `i`th multiplier set to `1 - b_i` .
* `c_i` is a multiplication result of `a_i` and `b_i`.
* `q` is a quantity.

Computing the proof:

1. The quantity is assumed to be known and be in range `[0, 2^64)`.
2. Create 64 multipliers.
3. Assign the inputs and outputs of the multipliers to the values specified above.


## Converting boolean expressions

Any gadget that expresses a boolean function of some statements needs to convert it into a form
required by _Rank-1 Constraint System_ (R1CS) which specifies linear constraints between external
commitments and multipliers.

#### 1. Normalize statements

Each statement of the form `a = b` must be converted to a form `c = 0`, where `c = a - b`.

    a = b    ->    a - b = 0

#### 2. Convert a disjunction of statements into a multiplication
    
Each statement of the form `or(a = 0, b = 0)` is converted into a statement about multiplication: `a*b = 0`.

    a = 0 or b = 0    ->    a*b = 0

This means, each `OR` function requires a multiplier.

#### 3. Convert a conjunction of statements into a polynomial

Each statement of the form `and(a = 0, b = 0)` is converted into a 1-degree polynomial with a unique free variable `x`:

    a + x*b = 0

As an optimization, conjuction of `n+1` statements can use `n`-degree polynomial of the free variable `x`:

    a = 0 and b = 0 and c = 0   ->   a + x*b + x*x*c = 0

Note: the `AND` does not require a multiplier because secrets are multiplied by a non-secret _constant_ `x`.
