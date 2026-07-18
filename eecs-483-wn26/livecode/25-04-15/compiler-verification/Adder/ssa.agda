module Adder.SSA where

open import Cubical.Foundations.Prelude hiding (lower)
open import Cubical.Data.Nat
open import Cubical.Data.FinData
open import Cubical.Data.Vec as Vec

-- The number of variables that are in scope
Env : Type
Env = ℕ

Extend : Env → Env
Extend Γ = 1 + Γ

-- If there are n variables in scope, a variable can be represented as a number in the range [0..n)
Var : Env → Type
Var = Fin

-- This uses "intrinsic scoping" the variables in the program are
-- guaranteed to be in the provided scope Γ
data Immediate (Γ : Env) : Type where
  num : ℕ → Immediate Γ
  var : Var Γ → Immediate Γ
data Operation (Γ : Env) : Type where
  imm : Immediate Γ → Operation Γ
  _＋_ : Immediate Γ → Immediate Γ → Operation Γ
data StraightlineCode (Γ : Env) : Type where
  ret : Immediate Γ → StraightlineCode Γ
  op  : Operation Γ → StraightlineCode (Extend Γ) → StraightlineCode Γ

SSAProg : Type
SSAProg = StraightlineCode 1

Values : Env → Type
Values Γ = Vec ℕ Γ

⟦_⟧imm : ∀ {Γ} → Immediate Γ → Values Γ → ℕ
⟦ num x ⟧imm γ = x
⟦ var x ⟧imm γ = lookup x γ

⟦_⟧op :  ∀ {Γ} → Operation Γ → Values Γ → ℕ
⟦ imm i ⟧op γ = ⟦ i ⟧imm γ
⟦ i ＋ j ⟧op γ = ⟦ i ⟧imm γ + ⟦ j ⟧imm γ

⟦_⟧s : ∀ {Γ} → StraightlineCode Γ → Values Γ → ℕ
⟦ ret x ⟧s γ = ⟦ x ⟧imm γ
⟦ op o b ⟧s γ = ⟦ b ⟧s ((⟦ o ⟧op γ) ∷ γ)

⟦_⟧ssa : SSAProg → ℕ → ℕ
⟦ p ⟧ssa x = ⟦ p ⟧s (x ∷ [])
