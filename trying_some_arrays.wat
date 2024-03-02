(module
    (type $int_array (array i32))
    (func $lol (export "lol") (result (ref extern))
        (array.new_fixed $int_array 4 (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4)) ;; Cannot cast to (ref extern)
;;        This is a fine internal function but terrible to export, so layer over exported functions, copy array to memory and return length & index to memory?
        (extern.convert_any);;        (any.convert_extern)
    )

;;    (func $lol_wrapper (export "lol") (param $extern_array (ref extern)) (result (ref extern))
;;;;        (call $lol)
;;
;;        (ref.cast (ref array) (local.get $extern_array))
;;        (ref.cast (ref extern) )
;;    )

        (func $accessing (export "accessing") (param $idx i32) (result i32)
            (array.get $int_array
                (array.new_fixed $int_array 4 (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4))
                (local.get $idx))
        )
)
