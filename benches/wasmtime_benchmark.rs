use anyhow::Result;
use criterion::{criterion_group, criterion_main, Criterion};
use wasmtime::{GlobalType, MemoryType, Mutability, ValType};

fn benchmark_frame(c: &mut Criterion, timeout: bool) -> Result<()> {

    let mut config = wasmtime::Config::new();
    config.cranelift_opt_level(wasmtime::OptLevel::Speed);
    if timeout {
        config.epoch_interruption(true);
    }
    let engine = wasmtime::Engine::new(&config)?;

    let mut store = wasmtime::Store::new(&engine, ());
    store.set_epoch_deadline(60);

    let memory = wasmtime::Memory::new(&mut store, MemoryType::new(4, Some(4)))?;

    let mut linker = wasmtime::Linker::new(&engine);
    linker.define("env", "memory", memory)?;

    let platform_module = wasmtime::Module::new(&engine, include_bytes!("platform.wasm"))?;
    let module = wasmtime::Module::new(&engine, include_bytes!("technotunnel.wasm"))?;

    add_native_functions(&mut linker, &mut store)?;

    let _platform_instance = instantiate_platform(&mut linker, &mut store, &platform_module)?;
    let instance = linker.instantiate(&mut store, &module)?;

    let update = instance.get_typed_func::<(), (), _>(&mut store, "upd")?;

    let name = if timeout { "upd_timeout" } else { "upd" };

    c.bench_function(name, |b| b.iter(|| {
        store.set_epoch_deadline(10);
        update.call(&mut store, ()).unwrap();
    }));

    Ok(())
}

fn add_native_functions(
    linker: &mut wasmtime::Linker<()>,
    store: &mut wasmtime::Store<()>,
) -> Result<()> {
    linker.func_wrap("env", "acos", |v: f32| v.acos())?;
    linker.func_wrap("env", "asin", |v: f32| v.asin())?;
    linker.func_wrap("env", "atan", |v: f32| v.atan())?;
    linker.func_wrap("env", "atan2", |x: f32, y: f32| x.atan2(y))?;
    linker.func_wrap("env", "cos", |v: f32| v.cos())?;
    linker.func_wrap("env", "exp", |v: f32| v.exp())?;
    linker.func_wrap("env", "log", |v: f32| v.ln())?;
    linker.func_wrap("env", "sin", |v: f32| v.sin())?;
    linker.func_wrap("env", "tan", |v: f32| v.tan())?;
    linker.func_wrap("env", "pow", |a: f32, b: f32| a.powf(b))?;
    for i in 10..64 {
        linker.func_wrap("env", &format!("reserved{}", i), || {})?;
    }
    let log_line = std::sync::Mutex::new(String::new());
    linker.func_wrap("env", "logChar", move |c: i32| {
        let mut log_line = log_line.lock().unwrap();
        if c == 10 {
            println!("{}", log_line);
            log_line.clear();
        } else {
            log_line.push(c as u8 as char);
        }
    })?;
    for i in 0..16 {
        linker.define(
            "env",
            &format!("g_reserved{}", i),
            wasmtime::Global::new(
                &mut *store,
                GlobalType::new(ValType::I32, Mutability::Const),
                0.into(),
            )?,
        )?;
    }

    Ok(())
}

fn instantiate_platform(
    linker: &mut wasmtime::Linker<()>,
    store: &mut wasmtime::Store<()>,
    platform_module: &wasmtime::Module,
) -> Result<wasmtime::Instance> {
    let platform_instance = linker.instantiate(&mut *store, &platform_module)?;

    for export in platform_instance.exports(&mut *store) {
        linker.define(
            "env",
            export.name(),
            export
                .into_func()
                .expect("platform surely only exports functions"),
        )?;
    }

    Ok(platform_instance)
}

fn benchmark_frame_no_timeout(c: &mut Criterion) {
    benchmark_frame(c, false).unwrap();
}

fn benchmark_frame_timeout(c: &mut Criterion) {
    benchmark_frame(c, true).unwrap();
}

criterion_group!(benches, benchmark_frame_no_timeout, benchmark_frame_timeout);
criterion_main!(benches);