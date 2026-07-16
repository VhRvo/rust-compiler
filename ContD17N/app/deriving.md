# Deriving `convD` from `convK`

Assume the following equivalence holds for every `expr`, `dest`, and `k`:

```text
eq-spec :=
  forall expr dest k.
    convK expr dest k
    ===
    convD expr dest (k (Output.Var dest))
```

Here, an expression such as

```text
eq-spec expr dest k
```

means that the universally quantified theorem `eq-spec` is instantiated with
the given `expr`, `dest`, and `k`.

## Why this instantiation must come first

Our goal is to define `convD expr dest next` for an **arbitrary** `next`.
However, `eq-spec` does not initially expose `convD` at an arbitrary third
argument. It only exposes `convD` at an argument of the particular form

```text
k (Output.Var dest)
```

Therefore, before orienting the equation as a definition of `convD`, we must
answer the following question:

```text
Given an arbitrary next, how can we choose k so that
k (Output.Var dest) === next?
```

The constant function provides such a `k` for every `next`:

```text
k := const next
```

This choice is necessary because it bridges the different shapes of the third
arguments:

```text
convK:  k    :: Immediate -> Instructment
convD:  next :: Instructment
```

In other words, `const` turns an arbitrary `next` into a continuation of the
type expected by `convK`, while guaranteeing that applying this continuation
to `Output.Var dest` recovers exactly the same `next`.

Now fix arbitrary `expr`, `dest`, and `next`, and instantiate `eq-spec` with

```text
k := const next
```

This gives

```text
convK expr dest (const next)
===
convD expr dest ((const next) (Output.Var dest))
```

By the definition of `const`,

```text
(const next) (Output.Var dest)
===
(\_ -> next) (Output.Var dest)
===
next
```

Therefore,

```text
def-spec :=
  forall expr dest next.
    convK expr dest (const next)
    ===
    convD expr dest next
```

In other words,

```text
eq-spec
=>
def-spec
```

Equivalently, orienting the equation as a definition of `convD`,

```text
convD expr dest next
:=
convK expr dest (const next)
```

The term `k (Output.Var dest)` is not cancelled for an arbitrary `k`.
Instead, `k` is deliberately instantiated as the constant function
`const next`, after which the application disappears by beta reduction.
