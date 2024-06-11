use anyhow::Result;
use criterion::{criterion_group, criterion_main, Criterion};

criterion_main!(benches);
criterion_group!(benches, benchmark_call_overhead);

fn benchmark_call_overhead(c: &mut Criterion) {
    fn inner(c: &mut Criterion) -> Result<()> {
        let mut config = wasmtime::Config::new();
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        let engine = wasmtime::Engine::new(&config)?;

        let mut store = wasmtime::Store::new(&engine, ());

        let mut linker = wasmtime::Linker::new(&engine);

        let module = wasmtime::Module::new(&engine, WASM_MODULE_A.as_bytes())?;

        linker.func_wrap("env", "native", |v: f32| v)?;

        let instance = linker.instantiate(&mut store, &module)?;

        let test_wasm = instance.get_typed_func::<i32, f32, _>(&mut store, "test_wasm")?;
        let test_native = instance.get_typed_func::<i32, f32, _>(&mut store, "test_native")?;

        c.bench_function("calling function defined in wasm", |b| {
            b.iter(|| {
                test_wasm.call(&mut store, 1_000_000).unwrap();
            })
        });

        c.bench_function("calling function defined in rust", |b| {
            b.iter(|| {
                test_native.call(&mut store, 1_000_000).unwrap();
            })
        });

        Ok(())
    }
    inner(c).unwrap()
}

const WASM_MODULE_A: &str = "
(module
    (import \"env\" \"native\" (func $func_native (param f32) (result f32)))

    (func $func_wasm (param $v f32) (result f32)
        (local.get $v)
    )

    (func (export \"test_wasm\") (param $i i32) (result f32)
        (local $sum f32)
        (loop $loop
            (local.set $sum
                (f32.add
                    (local.get $sum)
                    (call $func_wasm (f32.convert_i32_s (local.get $i)))
                )
            )
            (br_if $loop
                (local.tee $i
                    (i32.sub (local.get $i) (i32.const 1))
                )
            )
        )
        (local.get $sum)
    )

    (func (export \"test_native\") (param $i i32) (result f32)
        (local $sum f32)
        (loop $loop
            (local.set $sum
                (f32.add
                    (local.get $sum)
                    (call $func_native (f32.convert_i32_s (local.get $i)))
                )
            )
            (br_if $loop
                (local.tee $i
                    (i32.sub (local.get $i) (i32.const 1))
                )
            )
        )
        (local.get $sum)
    )
)
";
