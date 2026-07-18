module Adder.Asm where

open import Cubical.Foundations.Prelude
open import Cubical.Data.List as List
open import Cubical.Data.Nat
open import Cubical.Relation.Nullary.Base

-- we can pretend
i64 = ℕ
Addr = ℕ

data Register : Type where
  rax rdi : Register

data Instr : Type where
  movReg_,_    : Register → Register → Instr
  load_,_   : Register → Addr → Instr
  store_,_  : Addr → Register → Instr
  movConst_,_  : Register → i64 → Instr
  add_,_    : Register → Register → Instr

x86Prog : Type
x86Prog = List Instr

Memory : Type
Memory = Addr → i64

record x86State : Type where
  field
    raxV : i64
    rdiV : i64
    memory : Addr → i64

open x86State
readReg : Register → x86State → i64
readReg rax s = s .raxV
readReg rdi s = s .rdiV

writeReg : Register → i64 → x86State → x86State
writeReg rax c s = record { raxV = c ; rdiV = s .rdiV ; memory = s .memory }
writeReg rdi c s = record { raxV = s .raxV ; rdiV = c ; memory = s .memory }

stoMem : Addr → i64 → Memory → Memory
stoMem a c mem a' = decRec (λ _ → c) (λ _ → mem a') (discreteℕ a a')
    
sto : Addr → i64 → x86State → x86State
sto a c s .raxV = s .raxV
sto a c s .rdiV = s .rdiV
sto a c s .memory = stoMem a c (s .memory)

run : Instr → x86State → x86State
run (movReg r , r') s  = writeReg r (readReg r s) s
run (load r , a) s     = writeReg r (s .memory a) s
run (store a , r) s    = sto a (readReg r s) s
run (movConst r , c) s = writeReg r c s
run (add r , r') s     = writeReg r (readReg r s + readReg r' s) s

runProg : x86Prog → x86State → x86State
runProg [] s = s
runProg (i ∷ p) s = runProg p (run i s)

