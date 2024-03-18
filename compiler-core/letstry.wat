(module
(type $heap_type (sub (struct (field $tag i64))))
(func $internal_add (param $x (ref i31)) (param $y (ref i31)) (result (ref i31))
    (i32.add (local.get $x) (i31.get_s) (local.get $y) (i31.get_s))
    (ref.i31))
(func $add (export "add") (param $x (ref i31)) (param $y (ref i31)) (result (ref i31))
    (call $internal_add (i32.add (local.get $x) (i31.get_s) (i32.const 1) (ref.i31) (i31.get_s)) (ref.i31) (local.get $y)))
(@producers (language "Gleam" "1.0.0"))
)