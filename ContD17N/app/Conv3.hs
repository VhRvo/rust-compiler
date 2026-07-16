{-# LANGUAGE OverloadedStrings #-}

module Conv3 where

import Common
import Conv2 qualified
import Input
import Output

conv :: Input.Expr -> Output.Instructment
conv expr = convK expr "$result" (Return (Output.Var "$result"))

-- Conv2.convK expr dest k = Conv3.convK expr dest (k dest)
-- Conv3.convK expr dest (k dest) = Conv2.convK expr dest k
-- let k := const next
-- Conv3.convK expr dest next = Conv2.convK expr dest (const next)

convK :: Input.Expr -> Identifier -> Instructment -> Instructment
-- Conv3.convK expr dest next = Conv2.convK expr dest (const next)
convK (Input.Const x) dest next =
  -- Output.Let
  --   dest
  --   (Output.Immediate (Output.Const x))
  --   ((const next) (Output.Var dest))
  -- =>
  Output.Let
    dest
    (Output.Immediate (Output.Const x))
    next
convK (Input.Var x) dest next =
  -- Output.Let
  --   dest
  --   (Output.Immediate (Output.Var x))
  --   ((const next) (Output.Var dest))
  -- =>
  Output.Let
    dest
    (Output.Immediate (Output.Var x))
    next
convK (Input.Add e1 e2) dest next =
  -- Conv2.convK
  --   e1
  --   "lhsAtomized"
  --   ( \a1 ->
  --       Conv2.convK
  --         e2
  --         "rhsAtomized"
  --         ( \a2 ->
  --             Output.Let
  --               dest
  --               (Output.Add a1 a2)
  --               next
  --         )
  --   )
  -- =>
  Conv3.convK
    e1
    "lhsAtomized"
    ( Conv3.convK
        e2
        "rhsAtomized"
        ( Output.Let
            dest
            ( Output.Add
                (Output.Var "lhsAtomized")
                (Output.Var "rhsAtomized")
            )
            next
        )
    )
