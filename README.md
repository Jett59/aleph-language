# aleph language

Aleph will be a pure functional language with automatic verification of correctness. It will use a simple, mathematical syntax inspired from Haskell.

## Example syntax

### Simple function definition

f: Natural -> Natural
f(0) = 1
f(n) = n * f(n-1)

### Automatic verification

f: Real -> Real
f(x) = x * (x+1)

require (x: Integer)
    f(x): Even
