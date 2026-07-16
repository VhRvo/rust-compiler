{-# LANGUAGE OverloadedStrings #-}

module Conv1 where

import Common
import Input
import Output

conv :: Input.Expr -> Output.Instructment
conv expr = convK expr (\a -> Return a)

convK :: Input.Expr -> (Immediate -> Instructment) -> Instructment
convK (Input.Const x) k =
  Output.Let
    "$result$"
    (Output.Immediate (Output.Const x))
    (k (Output.Var "$result$"))
convK (Input.Var x) k =
  Output.Let
    "$result$"
    (Output.Immediate (Output.Var x))
    (k (Output.Var "$result$"))
convK (Input.Add e1 e2) k =
  convK
    e1
    ( \a1 ->
        convK
          e2
          ( \a2 ->
              Output.Let
                "$result$"
                (Output.Add a1 a2)
                (k (Output.Var "$result$"))
          )
    )
