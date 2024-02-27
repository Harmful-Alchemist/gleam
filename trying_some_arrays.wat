(module
    (type $int_array (array i32))
    (func $lol (export "lol") (result (ref array))
        (array.new_fixed $int_array 4 (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4))
;;        This is a fine internal function but terrible to export, so layer over exported functions, copy array to memory and return length & index to memory?
    )

        (func $accessing (export "accessing") (param $idx i32) (result i32)
            (array.get $int_array
                (array.new_fixed $int_array 4 (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4))
                (local.get $idx))
        )
)
