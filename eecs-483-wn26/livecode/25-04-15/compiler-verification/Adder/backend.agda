module Adder.Backend where

open import Cubical.Foundations.Prelude hiding (lower)
open import Cubical.Foundations.HLevels
open import Cubical.Data.List as List
open import Cubical.Data.Nat
open import Cubical.Data.Nat.Order.Recursive
open import Cubical.Data.Empty as Empty
open import Cubical.Data.Maybe as Maybe
open import Cubical.Data.FinData
open import Cubical.Data.Vec as Vec
open import Cubical.Data.Sigma
open import Cubical.Relation.Nullary.Base

open import Adder.SSA
open import Adder.Asm

open x86State

-- varToAddr Γ x = Γ - x
varToAddr : ∀ Γ → Var Γ → Addr
varToAddr (suc Γ) zero = Γ
varToAddr (suc Γ') (suc x) = varToAddr Γ' x

immToReg_,_ : ∀ {Γ} → Register → Immediate Γ → Instr
immToReg r , (num n) = movConst r , n
immToReg r , (var x) = load r , varToAddr _ x

-- stores the result in rax, leaves the memory/rdi unchanged
emitOp : ∀ {Γ} → Operation Γ → x86Prog
emitOp (imm i) = [ immToReg rax , i ]
emitOp (i ＋ j) =
  (immToReg rax , i) ∷
  (immToReg rdi , j) ∷
  (add rax , rdi) ∷
  []

-- stores the result in rax
emit : ∀ {Γ} → StraightlineCode Γ → x86Prog
emit (ret i) = [ immToReg rax , i ]
emit {Γ} (op o b) =
  emitOp o List.++
  (store varToAddr (Extend Γ) zero , rax) ∷
  emit b

backEnd : SSAProg → x86Prog
backEnd p =
  (store varToAddr 1 zero , rdi) ∷
  emit p

-- | Correctness of backEnd/emit
valuesFromMem : ∀ Γ → Memory → Values Γ
valuesFromMem zero mem = []
valuesFromMem (suc Γ) mem = mem Γ ∷ valuesFromMem Γ mem

varToAddrCorrect : ∀ Γ (x : Var Γ) mem
  → mem (varToAddr Γ x)
    ≡ lookup x (valuesFromMem Γ mem)
varToAddrCorrect (suc Γ) zero mem = refl
varToAddrCorrect (suc Γ) (suc x) mem = varToAddrCorrect Γ x mem

writeRegFrame : ∀ r c s →
  writeReg r c s .memory ≡ s .memory
writeRegFrame rax c s = refl
writeRegFrame rdi c s = refl

immToRegCorrect : ∀ {Γ} r (imm : Immediate Γ) s
  → run (immToReg r , imm) s
  ≡ writeReg r (⟦ imm ⟧imm (valuesFromMem Γ (s .memory))) s
immToRegCorrect r (num x) s = refl
immToRegCorrect r (var x) s i =
  writeReg r (varToAddrCorrect _ x (s .memory) i) s

immToRegFrame : ∀ {Γ} r (i : Immediate Γ) s →
  run (immToReg r , i) s .memory ≡ s .memory
immToRegFrame r i s = cong memory (immToRegCorrect r i s) ∙ writeRegFrame r _ s

discℕRefl : ∀ a → discreteℕ a a ≡ yes refl
discℕRefl a with discreteℕ a a
... | yes p = cong yes (isSetℕ _ _ _ _)
... | no ¬p = Empty.rec (¬p refl)

discℕ< : ∀ a b → (p : b < a) → discreteℕ a b ≡ no (λ a≡b → <→≢ p (sym a≡b))
discℕ< a b p with discreteℕ a b
... | yes q = Empty.rec (<→≢ p (sym q))
... | no ¬q = cong no (isProp→ isProp⊥ _ _)

stoMemβ : ∀ a c mem → stoMem a c mem a ≡ c
stoMemβ a c mem = cong (decRec (λ _ → c) (λ _ → mem a)) (discℕRefl a)

stoβ₁ : ∀ a c s → sto a c s .memory a ≡ c
stoβ₁ a c s = cong (decRec (λ _ → c) (λ _ → s .memory a)) (discℕRefl a)

stoValuesFrameβ : ∀ Γ a c mem → Γ < a → stoMem a c mem Γ ≡ mem Γ
stoValuesFrameβ Γ a c mem Γ<a = cong (decRec (λ _ → c) (λ _ → mem Γ)) (discℕ< a Γ Γ<a)

stoValuesFrame : ∀ Γ a c mem
  → Γ ≤ a
  → valuesFromMem Γ (stoMem a c mem) ≡ valuesFromMem Γ mem
stoValuesFrame zero a c mem Γ≤a = refl

stoValuesFrame (suc Γ) a c mem Γ≤a =
  cong₂ _∷_
    (stoValuesFrameβ Γ a c mem Γ≤a)
    (stoValuesFrame Γ a c mem (≤-trans {k = Γ}{m = suc Γ}{n = a} (<-weaken {m = Γ}{n = suc Γ} (≤-refl Γ)) Γ≤a))

stoValues : ∀ Γ c mem
  → valuesFromMem Γ (stoMem Γ c mem) ≡ valuesFromMem Γ mem
stoValues Γ c mem = stoValuesFrame Γ Γ c mem (≤-refl Γ)

-- If I run an operation, it stores the correct value in rax
emitOpRax : ∀ {Γ} (o : Operation Γ) s
  → (runProg (emitOp o) s .raxV)
    ≡ (⟦ o ⟧op (valuesFromMem Γ (s .memory)))
emitOpRax (imm i) s k = immToRegCorrect rax i s k .raxV
emitOpRax (i ＋ j) s = cong₂ _+_
  (cong raxV (immToRegCorrect rdi j _) ∙ cong raxV (immToRegCorrect rax i _))
  (cong rdiV (immToRegCorrect rdi j _) ∙ λ k → ⟦ j ⟧imm (valuesFromMem _ (immToRegFrame rax i s k)))

-- If I run an operation, the memory is unchanged
emitOpFrame : ∀ {Γ} (o : Operation Γ) s
  → runProg (emitOp o) s .memory ≡ s .memory
emitOpFrame (imm i) s = immToRegFrame rax i s
emitOpFrame (i ＋ j) s = immToRegFrame rdi j _ ∙ immToRegFrame rax i _

run++ : ∀ p q s → runProg (p List.++ q) s ≡ runProg q (runProg p s)
run++ [] q s = refl
run++ (x ∷ p) q s = run++ p q (run x s)

-- stores the result in rax
emitCorrect : ∀ {Γ} (b : StraightlineCode Γ) s
 → (runProg (emit b) s .raxV ≡ ⟦ b ⟧s (valuesFromMem Γ (s .memory)))
emitCorrect (ret i) s = cong raxV (immToRegCorrect rax i s)
emitCorrect {Γ} (op o b) s =
  (λ i → run++ (emitOp o) (((store Γ , rax) ∷ emit b)) s i .raxV)
  ∙ (λ i → runProg (emit b) (sto Γ (emitOpRax o s i) (runProg (emitOp o) s)) .raxV)
  ∙ emitCorrect b _
  ∙ (λ i → ⟦ b ⟧s (stoβ₁ Γ (⟦ o ⟧op (valuesFromMem Γ (s .memory))) (runProg (emitOp o) s) i
    ∷ valuesFromMem Γ (stoMem Γ (⟦ o ⟧op (valuesFromMem Γ (s .memory))) (emitOpFrame o s i))))
  ∙ λ i → ⟦ b ⟧s (⟦ o ⟧op (valuesFromMem Γ (s .memory)) ∷ stoValues Γ (⟦ o ⟧op (valuesFromMem Γ (s .memory))) (s .memory) i)

backEndCorrect : ∀ p s
  → runProg (backEnd p) s .raxV ≡ ⟦ p ⟧ssa (s .rdiV)
backEndCorrect p s = emitCorrect p _
