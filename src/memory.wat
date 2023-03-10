(module
  (import "env" "input" (memory 0))
  (memory (export "memory") 0)

;;  (func (export "size") (result i32) (memory.size))
;;  (func (export "load") (param i32) (result i32)
;;    (i32.load8_s (local.get 0))
;;  )
;;  (func (export "store") (param i32 i32)
;;    (i32.store8 (local.get 0) (local.get 1))
;;  )

  (func (export "execute")
    (local $i i32)
    (local $end_pointer i32)
    (local $sum i32)
;;    (i32.const 0)
;;
;;    (i32.const 0)
;;    (i32.load8_u)
;;
;;    (i32.const 1)
;;    (i32.add)
;;
;;    (i32.store8)

    (i32.const 0)
    (i32.load 0)
    (local.set $i)

    (i32.const 4)
    (i32.load 0)
    (local.set $end_pointer)

    (loop $my_loop
      ;; do something with $i
      (local.get $i)
      (i32.load 0)
      (local.get $sum)
      (i32.add)
      (local.set $sum)

;;      (local.get $i)
;;      (local.get $i)
;;      (i32.load)
;;      (i32.const 2)
;;      (i32.add)
;;      (i32.store)

      ;; move $i by one element
      (local.get $i)
      (i32.const 4)
      (i32.add)
      (local.set $i)

      (local.get $i)
      (local.get $end_pointer)
      (i32.lt_s)
      (br_if $my_loop)
    )
;;    (i32.const 0)
;;    (i32.const 0)
;;    (i32.const 32)
;;    (memory.fill)

    (i32.const 0)
    (local.get $i)
    (i32.store 1)

    (i32.const 4)
    (local.get $sum)
    (i32.store 1)
  )

;;  (func (export "execute")
;;    (i32.const 4)
;;    (i32.const 8)
;;    (i32.store)
;;
;;    (i32.const 4)
;;    (i32.const 12)
;;    (i32.store)
;;  )

;;  (data (i32.const 0x1000) "\01\02\03\04")
)