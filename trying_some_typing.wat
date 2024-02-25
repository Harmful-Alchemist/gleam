(module
;;(type $result_struct (struct (field $idx i32)))

(type $ok_struct (struct (field $idx i32) (field $val i32)))
(type $err_struct (struct (field $idx i32) (field $error i64)))

;;All well and good but not really recursive yeah????
;;(rec
;;    (type $ok_struct (sub final (struct (field $idx i32) (field $val i32))))
;;    (type $err_struct (sub final (struct (field $idx i32) (field $error i64))))
;;    (type $result_struct (struct (field $idx i32)))
;;    )

(func $matching (param $in (ref struct)) (result i64)
    (local $out i64)

    (if
    (local.get $in) (ref.test (ref $err_struct))
    (then
        (local.get $in)
        (ref.cast (ref $err_struct))
        (struct.get $err_struct $error)
        (local.set $out)
    )
    (else
     (if
         (local.get $in) (ref.test (ref $ok_struct))
         (then
              (local.get $in)
                 (ref.cast (ref $ok_struct))
                 (struct.get $ok_struct $val)
                 (i64.extend_i32_s)
                 (local.set $out)
         )
         ) ;;This is really bad...  But last if...
    )
    )
    (local.get $out)
;;    (block $ok_block (i64.extend_i32_s (local.get $in) (struct.get $ok_struct $val))
;;        (block $err_block (local.get $in) (struct.get $err_struct $error)
;;            (br_table $ok_block $err_block (local.get $in) struct.get $err_struct $idx) ;; TODO huh local.get???
;;    ))

;;(local.get $in)
;;(struct.get $result_struct $idx)
;;(i64.extend_i32_s)
     ;;(unroll (local.get $in)) how to unroll??? It's not an instruction... Nah rolling/unrollign is a validation step :P
)
(func $lessee (export "lessee") (result i64)
    (call $matching  (i32.const 10) (call $ok_constructor)))
(func $ok_constructor (param $val i32) (result (ref struct))
    (struct.new $ok_struct (i32.const 0) ;;This is the important part
     (local.get $val))
     (ref.cast (ref struct)))
(func $err_constructor (param $err i64) (result (ref struct))
    (struct.new $err_struct (i32.const 0) ;;This is the important part
     (local.get $err))
     (ref.cast (ref struct)))
)
