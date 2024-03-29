(module

(type $heap_type (sub (struct (field $tag i64))))
(rec
  (type $list (sub $heap_type (struct (field $tag i64))))
  (type $cons (sub final $list (struct (field $tag i64) (field $value (ref eq)) (field $prev (ref $list)))))
  (type $empty (sub final $list (struct (field $tag i64))))
)



(func $sum (param $listy (ref $list)) (param $acc (ref i31)) (result (ref i31))
    (local $h (ref i31))
    (local $t (ref $list))
    (local $return i32)
;;    (block $first_block
;; (block $last_block

    (if
    (i64.eq
        (i64.const 3)
;;        (i64.const 3)
        (struct.get $list 0 (local.get $listy))
    )
    (then
;;        (local.set $return (i32.const 1))
        (ref.cast (ref $cons) (local.get $listy))
         (ref.cast (ref i31) (struct.get $cons $value))
         (local.set $h)
;;         (local.set $return (i32.add (i32.const 5)(i31.get_s (local.get $h)))) ;;This works..
         (local.get $listy)
         (ref.cast (ref $cons))
         (struct.get $cons $prev)
         (local.set $t)
         (local.set $return
;;            (i32.add
;;;;                (i32.const -2) ;; This works too
;;                (i31.get_s
            (return_call $sum
            (local.get $t)
            (ref.i31 (i32.add (i31.get_s(local.get $h)) (i31.get_s(local.get $acc))))
            )
         ) ;;TODO this does not work??? Putting in listy does get infinite
;;                (i31.get_s(local.get $h))
;;            )
         )


    (else
        (local.set $return (i31.get_s (local.get $acc))) ;;Changing this makes it more...
;;        return
    )
    )
    (ref.i31 (local.get $return))
;; (local.get $listy) (br_on_cast 1 (;$first_block;) (ref $list) (ref $empty))  ;;) ;;end block $last_block
;;;; (local.get $listy)
;;;; ref.cast (ref $cons)
;;;; (struct.get $cons $value)
;;;; (local.set $h)
;;;; (local.get $listy)
;;;; (struct.get $cons $prev)
;;;; (local.set $t)
;;;; (i32.add (local.get $h) (i31.get_s) (call $sum (local.get $t)))
;;;; (ref.i31)
;;;;return
;;(i32.const 2)
;;(ref.i31)
;;return
;;) ;;end block $first_block
;; (i32.const 0) (ref.i31)
)


(func $add (export "add") (param $x (ref i31)) (param $y (ref i31)) (result (ref i31))
    (local $z (ref $list))
       (struct.new
            $cons
            (i64.const 3)
            (local.get $y)
            (ref.cast (ref $list)
    (struct.new
        $cons
        (i64.const 3)
        (local.get $x)
        (ref.cast (ref $list)
        (struct.new $empty
                (i64.const 4))))))
;;    ref.cast (ref $list)
;;    (local.get $y)
;;    (i64.const 2125840804111237141)
;;    (struct.new $cons)
;;    ref.cast (ref $list)
;;    (i32.const 3) (ref.i31)
;;    (i64.const -935051138086808991)
;;    (struct.new $cons)
;;    ref.cast (ref $list)
    (local.set $z)
    (call $sum (local.get $z) (ref.i31 (i32.const 0)))
)
(@producers (language "Gleam" "1.0.0"))
)