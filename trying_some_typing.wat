(module
(type $result_struct (sub (struct (field $idx i32)))) ;;Ordering not sub-types after super types!
(type $ok_struct (sub final $result_struct (struct (field $idx i32) (field $val i32))))
(type $err_struct (sub final $result_struct (struct (field $idx i32) (field $error i64))))

(func $matching (param $in (ref $result_struct)) (result i64)
    (block $ok_block
        (block $err_block
            (br_table $ok_block $err_block (local.get $in) struct.get $result_struct $idx)
    ) ;; end err_block
    (local.get $in)
    (ref.cast (ref $err_struct))
    (struct.get $err_struct $error)
    return
    ) ;;end ok_block
    (local.get $in)
    (ref.cast (ref $ok_struct))
    (struct.get $ok_struct $val)
    (i64.extend_i32_s)
)
(func $lessee (export "lessee") (result i64)
    (call $matching  (i64.const 11) (call $err_constructor)))
(func $ok_constructor (param $val i32) (result (ref $result_struct))
    (struct.new $ok_struct (i32.const 0) ;;This is the important part
     (local.get $val))
     (ref.cast (ref $result_struct)))
(func $err_constructor (param $err i64) (result (ref $result_struct))
    (struct.new $err_struct (i32.const 1) ;;This is the important part
     (local.get $err))
     (ref.cast (ref $result_struct)))
)
