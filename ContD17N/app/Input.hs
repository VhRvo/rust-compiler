module Input where

import Common

data Expr
  = Const Int
  | Var Identifier
  | Add Expr Expr

-- \| Let Identifier Expr Expr
