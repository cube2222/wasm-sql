(module
(import "env" "input" (memory 0))
(memory (export "memory") 0)
(func (export "execute")
(local $output_ptr_0 i32)
;; declare sum output
(local $sum_1 i32)
;; declare map output
(local $a_2 i32)
(local $b_3 i32)
;; declare loop variable
(local $i_4 i32)
(i32.const 3)
(local.set $i_4)
(loop $loop_5
;; evaluate a
;; add
;; load variable
(local.get $i_4)
;; load variable
(local.get $i_4)
(i32.add)
(local.set $a_2)
;; evaluate b
;; add
;; load variable
(local.get $i_4)
;; load variable
(local.get $i_4)
(i32.add)
(local.set $b_3)
;; add to sum
(local.get $sum_1)
(local.get $a_2)
(i32.add)
(local.set $sum_1)
;; increment i
(local.get $i_4)
(i32.const 1)
(i32.add)
(local.set $i_4)
;; check if loop finished
(local.get $i_4)
(i32.const 1000000000)
(i32.lt_s)
(br_if $loop_5)
)
;; store output
(local.get $output_ptr_0)
(local.get $sum_1)
(i32.store 1)
;; update output pointer
(local.get $output_ptr_0)
(i32.const 4)
(i32.add)
(local.set $output_ptr_0)
)
)
