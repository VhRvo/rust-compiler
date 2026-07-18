module Expressions where

open import Cubical.Foundations.Prelude
open import Cubical.Data.Nat

-- Arithmetic expression language
data Exp : Type where
  num : ℕ → Exp
  _＋_ : Exp → Exp → Exp
  _**_ : Exp → Exp → Exp

example : Exp
example = num 1 ＋ ((num 0 ＋ (num 2 ** num 3)) ** num 7)

-- Semantics
⟦_⟧ : Exp → ℕ
⟦ num x ⟧ = x
⟦ x ＋ x₁ ⟧ = ⟦ x ⟧ + ⟦ x₁ ⟧
⟦ x ** x₁ ⟧ = ⟦ x ⟧ · ⟦ x₁ ⟧

-- Unit tests
test1 : ⟦ example ⟧ ≡ 43
test1 = refl

-- A stack machine "assembly code"
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
run (add is) [] = run is []
run (add is) (x ∷ []) = run is (x ∷ [])
run (add is) (x ∷ (y ∷ s)) = run is ((x + y) ∷ s)
run (mul is) [] = run is []
run (mul is) (x ∷ []) = run is (x ∷ [])
run (mul is) (x ∷ (y ∷ s)) = run is ((x · y) ∷ s)

example-sm : Instrs
example-sm = push 6 (mul halt)

test2 : run example-sm (5 ∷ []) ≡ (30 ∷ [])
test2 = refl

run-⨾ : ∀ is js s → run (is ⨾ js) s ≡ run js (run is s)
run-⨾ halt js s = refl
run-⨾ (push x is) js s = run-⨾ is js (x ∷ s)
run-⨾ (add is) js [] = run-⨾ is js []
run-⨾ (add is) js (x ∷ []) = run-⨾ is js (x ∷ [])
run-⨾ (add is) js (x ∷ (y ∷ s)) = run-⨾ is js ((x + y) ∷ s )
run-⨾ (mul is) js [] = run-⨾ is js []
run-⨾ (mul is) js (x ∷ []) = run-⨾ is js (x ∷ [])
run-⨾ (mul is) js (x ∷ (y ∷ s)) = run-⨾ is js ((x · y) ∷ s )

-- Compiler from expressions to stack machine instructions
compile : Exp → Instrs
compile (num x) = push x halt
compile (e ＋ e') = compile e ⨾ compile e' ⨾ add halt
compile (e ** e') = compile e' ⨾ compile e ⨾ mul halt

correctness : ∀ s e → run (compile e) s ≡ ⟦ e ⟧ ∷ s
correctness s (num x) = refl
correctness s (e ＋ e₁) =
  run-⨾ (compile e) _ _
  ∙ cong (run (compile e₁ ⨾ add halt)) (correctness s e)
  ∙ run-⨾ (compile e₁) _ _
  ∙ cong (run (add halt)) (correctness (⟦ e ⟧ ∷ s) e₁)
  ∙ cong (_∷ s) (+-comm ⟦ e₁ ⟧ ⟦ e ⟧)
correctness s (e ** e₁) =
-- alternative style that is more readable:
  run (compile (e ** e₁)) s
    ≡⟨ run-⨾ (compile e₁) (compile e ⨾ mul halt) s  ⟩
  run (compile e ⨾ mul halt) (run (compile e₁) s)
    ≡⟨ run-⨾ (compile e) _ _ ⟩
  run (mul halt) (run (compile e) (run (compile e₁) s))
    -- notation for congruence proofs
    ≡[ i ]⟨ run (mul halt) (run (compile e) (correctness s e₁ i)) ⟩
  run (mul halt) (run (compile e) (⟦ e₁ ⟧ ∷ s))
    ≡[ i ]⟨ run (mul halt) (correctness (⟦ e₁ ⟧ ∷ s) e i) ⟩
  run (mul halt) (⟦ e ⟧ ∷ (⟦ e₁ ⟧ ∷ s))
    ≡⟨ refl ⟩
  (⟦ e ** e₁ ⟧ ∷ s) ∎

correctness-og :  ∀ e → run (compile e) [] ≡ ⟦ e ⟧ ∷ []
correctness-og e = correctness [] e
