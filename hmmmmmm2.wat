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
    (local $return (ref i31))
    (if
    (i64.eq
        (i64.const 3)
        (struct.get $list 0 (local.get $listy))
    )
    (then
        (ref.cast (ref $cons) (local.get $listy))
         (ref.cast (ref i31) (struct.get $cons $value))
         (local.set $h)
         (local.get $listy)
         (ref.cast (ref $cons))
         (struct.get $cons $prev)
         (local.set $t)
            (return_call $sum
            (local.get $t)
            (ref.i31 (i32.add (i31.get_s(local.get $h)) (i31.get_s(local.get $acc))))
            )
;;          return ;; Need this if using return
;;          (local.set $return)
         )
;;    (else
;;        (local.get $acc) ;;Changing this makes it more...
;;        (ref.i31 (i32.const 0))
;;        return
;;        (local.set $return)
;;    )
    )
;;    (ref.i31 (i32.const 0))
    (local.get $acc)
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
    (local.set $z)
    (call $sum (local.get $z) (ref.i31 (i32.const 0)))
)
(@producers (language "Gleam" "1.0.0"))
)