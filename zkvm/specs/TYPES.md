# ZkVM Specification: Types

> Part of the ZkVM Specification.  
> &laquo; [Introduction](./README.md) | [Home](./README.md) | [Next: Definitions &raquo;](./DEFINITIONS.md)
---

## Types

This document specifies the data types used by the ZkVM. The VM is strongly-typed, and understanding these types, especially the distinction between Linear and Copyable types, is crucial for writing correct and secure ZkVM programs.


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

---
> &laquo; [Introduction](./README.md) | [Home](./README.md) | [Next: Definitions &raquo;](./DEFINITIONS.md)