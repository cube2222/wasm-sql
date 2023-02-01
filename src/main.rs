//! An example of how to interact with wasm memory.
//!
//! Here a small wasm module is used to show how memory is initialized, how to
//! read and write memory through the `Memory` object, and how wasm functions
//! can trap when dealing with out-of-bounds addresses.

// You can execute this example with `cargo run --example example`

use std::time::SystemTime;
use anyhow::Result;
use wasmtime::*;

fn main() -> Result<()> {
    // Create our `store_fn` context and then compile a module and create an
    // instance from the compiled module all in one go.
    let mut config = Config::default();
    config.wasm_multi_memory(true);
    let mut store: Store<()> = Store::new(&Engine::new(&config).unwrap(), ());
    let start = SystemTime::now();
    let module = Module::from_file(store.engine(), "src/memory.wat")?;
    println!("Module compiled in {:?}", start.elapsed()?);
    let input_memory = Memory::new(&mut store, MemoryType::new(64, None)).unwrap();
    input_memory.grow(&mut store, 4).unwrap();
    let count = 4;
    let start_pointer = 4*2;
    let end_pointer = start_pointer + 4*count;
    (&mut input_memory.data_mut(&mut store)[0..4]).copy_from_slice(&(start_pointer as u32).to_le_bytes());
    (&mut input_memory.data_mut(&mut store)[4..8]).copy_from_slice(&(end_pointer as u32).to_le_bytes());
    for i in (0 as usize)..(count as usize) {
        (&mut input_memory.data_mut(&mut store)[start_pointer + i * 4..start_pointer + (i + 1) * 4]).copy_from_slice(&(i as u32).to_le_bytes());
    }

    let instance = Instance::new(&mut store, &module, &[
        Extern::Memory(input_memory),
    ])?;
    println!("Instance created in {:?}", start.elapsed()?);

    // load_fn up our exports from the instance
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or(anyhow::format_err!("failed to find `memory` export"))?;
    memory.grow(&mut store, 4)?;
    // assert_eq!(memory.size(&store), 1024);
    // assert_eq!(memory.data_size(&store), 65_536*1024);
    // // memory.data_ptr()
    // for i in (0 as usize)..(10_000_000 as usize) {
    //     (&mut memory.data_mut(&mut store)[i * 4..(i + 1) * 4]).copy_from_slice(&((i%10) as u32).to_le_bytes());
    // }
    // let mut sum: u32 = 0;
    // for i in 0..10_000_000 {
    //     sum += u32::from_le_bytes(memory.data(&store)[i * 4..(i + 1) * 4].try_into().unwrap());
    //     // sum += i;
    // }

    println!("{:?}", module.imports().collect::<Vec<_>>());

    let execute_fn = instance.get_typed_func::<(), ()>(&mut store, "execute")?;


    println!("Executing after {:?}", start.elapsed()?);
    execute_fn.call(&mut store, ())?;
    println!("Executed once after {:?}", start.elapsed()?);
    // execute_fn.call(&mut store, ())?;
    // println!("Executed twice after {:?}", start.elapsed()?);

    // let memory = instance
    //     .get_memory(&mut store, "memory")
    //     .ok_or(anyhow::format_err!("failed to find `memory` export"))?;

    let output_memory = instance.exports(&mut store).next().unwrap().into_memory().unwrap();

    // assert_eq!(u32::from_le_bytes(memory.data(&store)[0..4].try_into().unwrap()), 4*10_000_000);
    // assert_eq!(sum, 45000000);
    println!("Memory state: {:?}", input_memory.data(&store)[0..16].to_vec());
    println!("Memory state: {:?}", output_memory.data(&store)[0..16].to_vec());
    assert_eq!(u32::from_le_bytes(output_memory.data(&store)[4..8].try_into().unwrap()), 6);

    Ok(())
}
