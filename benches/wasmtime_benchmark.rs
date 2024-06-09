use anyhow::Result;
use criterion::{criterion_group, criterion_main, Criterion};
use wasmtime::MemoryType;

fn benchmark_frame(c: &mut Criterion) {
    fn inner(c: &mut Criterion, wasm: &[u8], id: &str) -> Result<()> {
        let mut config = wasmtime::Config::new();
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        let engine = wasmtime::Engine::new(&config)?;

        let mut store = wasmtime::Store::new(&engine, ());

        let memory = wasmtime::Memory::new(&mut store, MemoryType::new(4, Some(4)))?;

        let mut linker = wasmtime::Linker::new(&engine);
        linker.define("env", "memory", memory)?;

        let module = wasmtime::Module::new(&engine, wasm)?;

        linker.func_wrap("env", "sin", |v: f32| v)?;

        let instance = linker.instantiate(&mut store, &module)?;

        let update = instance.get_typed_func::<(), (), _>(&mut store, "upd")?;

        c.bench_function(id, |b| {
            b.iter(|| {
                update.call(&mut store, ()).unwrap();
            })
        });

        Ok(())
    }
    inner(c, include_bytes!("technotunnel.wasm"), "technotunnel_upd").unwrap();
    inner(c, include_bytes!("technotunnel_nosin.wasm"), "technotunnel_nosin_upd").unwrap();
}

criterion_group!(benches, benchmark_frame);
criterion_main!(benches);
