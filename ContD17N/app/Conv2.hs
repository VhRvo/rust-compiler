{-# LANGUAGE OverloadedStrings #-}

module Conv2 where

import Common
import Input
import Output

conv :: Input.Expr -> Output.Instructment
conv expr = convK expr "$result" (\a -> Return a)

convK :: Input.Expr -> Identifier -> (Immediate -> Instructment) -> Instructment
convK (Input.Const x) dest k =
  Output.Let
    dest
    (Output.Immediate (Output.Const x))
    (k (Output.Var dest))
convK (Input.Var x) dest k =
  Output.Let
    dest
    (Output.Immediate (Output.Var x))
    (k (Output.Var dest))
convK (Input.Add e1 e2) dest k =
  convK
    e1
    "lhsAtomized"
    ( \a1 ->
        convK
          e2
          "rhsAtomized"
          ( \a2 ->
              Output.Let
                dest
                (Output.Add a1 a2)
                (k (Output.Var dest))
          )
    )
