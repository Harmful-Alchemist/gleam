(module
    (type $int_array (array i32))
    (func $lol (export "lol") (result (ref array))
        (array.new_fixed $int_array 4 (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4)) ;; Cannot cast to (ref extern)
;;        This is a fine internal function but terrible to export, so layer over exported functions, copy array to memory and return length & index to memory?
;;        (ref.cast anyref )
;;        (extern.convert_any)
    )

;;    (func $lol_wrapper (export "lol") (param $extern_array (ref null extern)) (result (ref null extern))
;;;;        (call $lol)
;;        (local $ehm (ref any))
;;        (any.convert_extern (local.get $extern_array))
;;        (ref.as_non_null)
;;        (local.set $ehm)
;;;;        (ref.cast (ref null any) (local.get $ehm)) ;;sure...
;;        (ref.cast (ref array) (local.get $ehm)) ;;illegal cast, why? array /<= any? No the spec says any should be the supertype of all aggragate types... But can't even cast to eq........
;;        ;; or the other way around ofc any /<= array makes sense, But in trying the other one I can up- and down-cast.
;;
;;        ;; The algorithm ToWebAssemblyValue(v, type) coerces a JavaScript value to a WebAssembly value by performing the following steps:
;;        ;; from: https://webassembly.github.io/spec/js-api/#retrieving-an-extern-value
;;
;;        (extern.convert_any)
;;    )

        (func $accessing (export "accessing") (param $ext_arr externref) (param $idx i32) (result i32)
            (local $array_here (ref $int_array))
            (any.convert_extern (local.get $ext_arr))
            (ref.as_non_null)
            (ref.cast (ref $int_array))
            (local.set $array_here)
            (array.get $int_array
                (local.get $array_here)
;;                (array.new_fixed $int_array 4 (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4))
                (local.get $idx))
        )
)
