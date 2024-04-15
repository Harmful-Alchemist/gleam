(module

(type $heap_type (sub (struct (field $tag i64))))
(rec
  (type $list (sub $heap_type (struct (field $tag i64))))
  (type $cons (sub final $list (struct (field $tag i64) (field $value (ref eq)) (field $prev (ref $list)))))
  (type $empty (sub final $list (struct (field $tag i64))))
)

;;TODO aha, wasocaml makes nested if's for a match (can do eq on tag) (hide em in a f64? NaNbox?). guile Hoot defines it's own less powerful match out of existing if etc. primitives it has already translated...
(func $sum (param $listy (ref $list)) (result (ref i31))
    (local $h (ref i31)) ;;TODO changed the type here! It's a types list....
    (local $t (ref $list))
    (local.get $listy)
    ;;(i32.const 132)
    (block $first_block (param (ref $list)) (result (ref 3))
        (block $last_block (param (ref $list)) (result (ref 3))
            (br_on_cast $last_block (ref eq) (ref $empty) (local.get $listy)) ;;TODO changed dest to one less block deep...
              ;;(i32.const 1) (ref.i31)
                    ;; drop ;; drop ;;Stack  ref eq & i32 current stack
             drop drop
             (local.get $listy)
             (ref.cast (ref $cons)) ;;TODO doesn't get picked up???????
             (struct.get $cons $value)
             (ref.cast (ref i31)) ;;TODO added!
             (local.set $h)
                               ;;(local.get $listy) ;;TODO was on the stack twice else do a local get!
             (local.get $listy)
             (ref.cast (ref $cons))
             (struct.get $cons $prev)
             (local.set $t)
             (i32.add (local.get $h) (i31.get_s) (call $sum (local.get $t)) (i31.get_s))
             (ref.i31)

             return
        ) ;;end block $last_block
        (i32.const 0) (ref.i31)
        return
    ) ;;end block $first_block
    unreachable ;; TODO not actually unreacable........ oh right jump to end lol
    ;;(i32.const 0) (ref.i31)
    ;;return
    ;;drop
    ;;(local.get $h) ;; Uninitted!
)
(func $add (export "add") (param $x (ref i31)) (param $y (ref i31)) (result (ref i31))
    (local $z (ref $list))
    (ref.cast (ref $list)
    (struct.new $cons (i64.const -935051138086808991) (i32.const 3) (ref.i31) (ref.cast (ref $list)  (struct.new $cons (i64.const 2125840804111237141) (local.get $y) (ref.cast (ref $list)  (struct.new $cons (i64.const -8287236360266773267) (local.get $x) (ref.cast (ref $list)  (struct.new $empty (i64.const -1674176718068870399))))))))
    )
    (local.set $z)
    (call $sum (local.get $z)))
(@producers (language "Gleam" "1.0.0"))
)