module Output where

import Common

data Immediate
  = Const Int
  | Var Identifier

data Operation
  = Add Immediate Immediate
  | Immediate Immediate

data Instructment
  = Return Immediate
  | Let Identifier Operation Instructment
