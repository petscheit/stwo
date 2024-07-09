# Factorial Constraints for the Stwo Prover

The goal for this submission is the implementation of the constraints required for verifying the correct computation of a factorial in a given trace. Since this is happening with the new Stwo prover, the underlying polynomial operations are represented as points on a circle. The underlying operations used with selector and vanishing polynomials are represented as operations on a circle. While the factorial computation is a simple operation, it adds a level of complexity to the constraint construction, as we need to evaluate the points with an offset. For a factorial computation, we cant check each constraint for each step, but only every second step. This falls under the topic of periodicity in STARKs.
(Thanks for the good [blog post](https://blog.lambdaclass.com/periodic-constraints-and-recursion-in-zk-starks/) explaining this!)


## How to Run

We have two tests that check the trace constraints and the verification of the proofs. For the constraint verification, we pass a set of constructed traces, and ensure they fail or pass correctly, depending on the inputs. The verification tests, generates a proof for a given factorial, and then validates the proof. 

These can be run with the following command:

```
cargo test -- examples::factorial --nocapture
```

## Trace structure

For simplicity, we decided to use only one column for the trace. To calculate factorial we need to keep track of two variables - iterator and accumulator. The simplest implementation would go like this:

```
iterator - input number
accumulator = 1

while iterator > 1:
    accumulator *= iterator
    iterator -= 1
```

So in our trace, values in even rows (starting from 0) represent the iterator and odd rows accumulaor.
Here's an example trace for 4 factorial.

|             |    |
|-------------|----|
| iterator    | 4  |
| accumulator | 1  |
| iterator    | 3  |
| accumulator | 4  |
| iterator    | 2  |
| accumulator | 12 |
| iterator    | 1  |
| accumulator | 24 |

## Constraints

To ensure the trace is valid we need to add two types of constraints: transitional (also called step) and boundary.

### Transitional constraints

We need to ensure two things for transition to be valid:

1. Accumulator in $i$-th row is equal to accumulator in $i-2$-th row multiplied by iterator in $i-3$-th row.
2. Iterator in $i$-th row is equal to iterator in $i-2$-th row minus 1.

### Boundary constraints

Besides that, we also need to ensure few boundary constraints:

1. Accumulator starts at 1.
2. Iterator starts at input number.
3. Iterator finishes at 1.

## Understanding of Circle STARK

We managed to understand how two and single point vanishing polynomials work. We also understood the boundary constraint that is used in fibonnaci example. Those boundary constraints ensure the following properties:

```
t(g^0) = 1
t(g^-1) = claim
```

We tried to modify them to achieve

```
t(g^1) = 1
t(g^-1) = claim
```

So we changed the following constraint:

```
1 + y * (self.claim - 1) * p.y^-1
```

in which `y * p.y^-1` is a term that equals to 0 when evaluation points is (1,0) = g^0 (because y=0), so in that case it simplifies to `x-1`, which ensures that the `t(g^0)=1`. On the other hand, when evaluation points is `g^-1`, `y` and `p.y` are equal to each other, so the term simplifies to `self.claim - 1`, which ensures that `t(g^-1) = claim`.

We tried to change that so this term is equal to 0 at g^1 point and 1 at g^-1 point. For us, maths seems to be right, however we didn't manage to correctly proof the valid trace.

## Limitations

We didnt manage to implement all of the constraints to ensure a sound verification of a given trace.

### 1. Combining 2 step constraints

We where unable to combine the two step constraints that are required to verify the correctness of a trace. In our solution, we combined them in a very naive (and unsound) way. After exploring the fibonacci implementation closer (and actually understanding the constraints fully) we should have probably tried constructing a single constraint in a better way, using the circle properties to act as on/off switches at specific points.

### 2. Boundry Constraints

We didnt manage to get the boundry constraints to work. We finally managed to come up with a nice solution, but sadly only 70 mins before submission. The logic is there however. This constraint should check that we initialize the accumilator (`a`) with 1, and that the last point evaluates to the claimed result of the factorial computation.


## Open topics (no time)

In the current implementation, we can only compute a factorial that requires 2^n steps. This is due to the fact, that a STARK trace must always have a length of 2^n. This is a limitation of the current implementation, as we can't compute a factorial with an arbitrary length. To solve this, we need to add dummy constraints to the trace, which satisfy all other constraints.

We didnt have time to properly think about this, but essentially, the idea was to prevent the `n` value to reduce past value 1. This would keep the factorial computation stuck in a loop, merely copying `n` and `a` until we fill the trace length sufficiently and can end the computation.