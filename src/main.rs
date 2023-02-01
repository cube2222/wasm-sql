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
    let mut store: Store<()> = Store::default();
    let start = SystemTime::now();
    let module = Module::from_file(store.engine(), "src/memory.wat")?;
    println!("Module compiled in {:?}", start.elapsed()?);
    let instance = Instance::new(&mut store, &module, &[])?;
    println!("Instance created in {:?}", start.elapsed()?);

    // load_fn up our exports from the instance
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or(anyhow::format_err!("failed to find `memory` export"))?;
    let execute_fn = instance.get_typed_func::<(), ()>(&mut store, "execute")?;

    memory.grow(&mut store, 4)?;
    // assert_eq!(memory.size(&store), 4);
    // assert_eq!(memory.data_size(&store), 65_536*4);
    // memory.data_ptr()
    for i in (0 as usize)..(4 as usize) {
        (&mut memory.data_mut(&mut store)[i * 4..(i + 1) * 4]).copy_from_slice(&(i as u32).to_le_bytes());
    }
    let mut sum: u32 = 0;
    for i in 0..4 {
        sum += u32::from_le_bytes(memory.data(&store)[i * 4..(i + 1) * 4].try_into().unwrap());
        // sum += i;
    }

    println!("{:?}", module.imports().collect::<Vec<_>>());

    // let mut input_memory = Memory::new(&mut store, MemoryType::new(64, None)).unwrap();
    // for i in (0 as usize)..(4 as usize) {
    //     (&mut input_memory.data_mut(&mut store)[i * 4..(i + 1) * 4]).copy_from_slice(&(i as u32).to_le_bytes());
    // }
    //
    // let instance = Instance::new(&mut store, &module, &[
    //     Extern::Memory(input_memory),
    // ])?;
    // let execute_fn = instance.get_typed_func::<(), ()>(&mut store, "execute")?;


    println!("Executing after {:?}", start.elapsed()?);
    execute_fn.call(&mut store, ())?;
    println!("Executed once after {:?}", start.elapsed()?);
    // execute_fn.call(&mut store, ())?;
    // println!("Executed twice after {:?}", start.elapsed()?);

    assert_eq!(u32::from_le_bytes(memory.data(&store)[0..4].try_into().unwrap()), 4*4);
    assert_eq!(sum, 6);
    println!("Memory state: {:?}", memory.data(&store)[0..16].to_vec());
    assert_eq!(u32::from_le_bytes(memory.data(&store)[4..8].try_into().unwrap()), sum);

    Ok(())
}
