(module
    (import "imports" "console.log" (func $console.log (param i32)))
    (func $log (export "log")
     (i32.const 42)
     (call $console.log)
    )
  (@producers
    (language "Wat" "some version")
    (processed-by "manual labor" "35 years old")
    (processed-by "nothing more?" "other version")
  )
)