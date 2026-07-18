module ExpressionsSolution where

open import Cubical.Foundations.Prelude
open import Cubical.Data.Nat

-- A simple arithmetic expression language
data Exp : Type where
  num : ℕ → Exp
  _＋_ : Exp → Exp → Exp
  _**_ : Exp → Exp → Exp

-- Semantics
⟦_⟧ : Exp → ℕ
⟦ num x ⟧ = x
⟦ e ＋ e' ⟧ = ⟦ e ⟧ + ⟦ e' ⟧
⟦ e ** e' ⟧ = ⟦ e ⟧ · ⟦ e' ⟧

-- A simple stack machine language
data Stack : Type where
  [] : Stack
  _∷_ : ℕ → Stack → Stack

data Instrs : Type where
  halt : Instrs
  push : ℕ → Instrs → Instrs
  add : Instrs → Instrs
  mul : Instrs → Instrs

infixr 20 _⨾_
_⨾_ : Instrs → Instrs → Instrs
halt ⨾ js = js
push x is ⨾ js = push x (is ⨾ js)
add is ⨾ js = add (is ⨾ js)
mul is ⨾ js = mul (is ⨾ js)

run : Instrs → Stack → Stack
run halt s = s
run (push x is) s = run is (x ∷ s)
run (add is) []            = run is (0 ∷ [])
run (add is) (x ∷ [])      = run is (x ∷ [])
run (add is) (x ∷ (y ∷ s)) = run is ((x + y) ∷ s)
run (mul is) []            = run is (1 ∷ [])
run (mul is) (x ∷ [])      = run is (x ∷ [])
run (mul is) (x ∷ (y ∷ s)) = run is ((x · y) ∷ s)

run-⨾ : ∀ is js s → run (is ⨾ js) s ≡ run js (run is s)
run-⨾ halt js s = refl
run-⨾ (push x is) js s = run-⨾ is _ _
run-⨾ (add is) js [] = run-⨾ is js (0 ∷ [])
run-⨾ (add is) js (x ∷ []) = run-⨾ is js (x ∷ [])
run-⨾ (add is) js (x ∷ (y ∷ s)) = run-⨾ is js ((x + y) ∷ s )
run-⨾ (mul is) js [] = run-⨾ is js (1 ∷ [])
run-⨾ (mul is) js (x ∷ []) = run-⨾ is js (x ∷ [])
run-⨾ (mul is) js (x ∷ (y ∷ s)) = run-⨾ is js ((x · y) ∷ s )

compile : Exp → Instrs
compile (num x) = push x halt
compile (e ＋ e') = compile e ⨾ compile e' ⨾ add halt
compile (e ** e') = compile e ⨾ compile e' ⨾ mul halt

correctness' : ∀ s e → run (compile e) s ≡ ⟦ e ⟧ ∷ s
correctness' s (num x) = refl
correctness' s (e ＋ e₁) =
  run-⨾ (compile e) (compile e₁ ⨾ add halt) s
  ∙ cong (run (compile e₁ ⨾ add halt)) (correctness' s e)
  ∙ run-⨾ (compile e₁) _ _
  ∙ cong (run (add halt)) (correctness' _ e₁)
  ∙ cong (_∷ s) (+-comm ⟦ e₁ ⟧ ⟦ e ⟧)

correctness' s (e ** e₁) = run-⨾ (compile e) _ _
  ∙ cong (run (compile e₁ ⨾ mul halt)) (correctness' s e)
  ∙ run-⨾ (compile e₁) _ _
  ∙ cong (run (mul halt)) (correctness' _ e₁)
  ∙ cong (_∷ s) (·-comm ⟦ e₁ ⟧ ⟦ e ⟧)

correctness : ∀ e → run (compile e) [] ≡ ⟦ e ⟧ ∷ []
correctness e = correctness' [] e

example : Exp
example = (num 0 ＋ (num 2 ** num 3)) ** num 7

compiled-example = compile example

test : compiled-example
  ≡ push 0 (push 2 (push 3 (mul (add (push 7 (mul halt)))))) 
test = refl

test2 : run compiled-example [] ≡ 42 ∷ []
test2 = refl

example-sm : Instrs
example-sm = push 6 (mul halt)

test2' : run example-sm (5 ∷ []) ≡ (30 ∷ [])
test2' = refl
