// You can execute this example with `cargo run --example example`

use std::collections::HashMap;
use std::fmt::format;
use std::fs::File;
use std::io::Write;
use std::iter::Map;
use std::time::SystemTime;
use anyhow::Result;
use wasmtime::*;

enum Node {
    Map(Box<Node>, Vec<Expr>, Vec<String>),
    // Filter(Box<Node>, Expr),
    Range(usize, usize),
    Output(Box<Node>, String, usize),
    // source, field name, memory index
    Sum(Box<Node>, String),    // source, field name
}

// TODO: Also store field mapping in context, so that they can come from higher scopes.
type ProduceFn<'a> = Box<dyn Fn(&mut ProduceContext, &VariableMapping) + 'a>;
type VariableMapping = HashMap<String, (String, ValueMetadata)>;

#[derive(Clone)]
enum ValueType {
    Int,
}

impl ValueType {
    fn primitive_type_name(&self) -> String {
        match self {
            ValueType::Int => "i32".to_string(),
        }
    }
}

#[derive(Clone)]
struct ValueMetadata {
    value_type: ValueType,
    nullable: bool,
}

impl Node {
    fn generate(&self, ctx: &mut GenContext, produce: ProduceFn) -> Result<()> {
        match self {
            Node::Range(start, end) => {
                ctx.buffer.push_str(";; declare loop variable\n");
                let i_name = ctx.get_unique("i");
                ctx.buffer.push_str(&format!("(local ${} i32)\n", i_name));
                ctx.buffer.push_str(&format!("(i32.const {})\n", start));
                ctx.buffer.push_str(&format!("(local.set ${})\n", i_name));
                let loop_name = ctx.get_unique("loop");
                let mut fields: VariableMapping = HashMap::new();
                fields.insert("i".to_string(), (i_name.to_string(), ValueMetadata { value_type: ValueType::Int, nullable: false }));
                ctx.buffer.push_str(&format!("(loop ${}\n", &loop_name));
                produce(&mut ProduceContext {
                    continue_label: loop_name.clone(),
                    gen_ctx: ctx,
                }, &fields);
                ctx.buffer.push_str(";; increment i\n");
                ctx.buffer.push_str(&format!("(local.get ${})\n", i_name));
                ctx.buffer.push_str(&format!("(i32.const 1)\n"));
                ctx.buffer.push_str(&format!("(i32.add)\n"));
                ctx.buffer.push_str(&format!("(local.set ${})\n", i_name));
                ctx.buffer.push_str(";; check if loop finished\n");
                ctx.buffer.push_str(&format!("(local.get ${})\n", i_name));
                ctx.buffer.push_str(&format!("(i32.const {})\n", end));
                ctx.buffer.push_str(&format!("(i32.lt_s)\n"));
                ctx.buffer.push_str(&format!("(br_if ${})\n", &loop_name));
                ctx.buffer.push_str(&format!(")\n"));
            }
            Node::Output(source, field, memory_index) => {
                let output_ptr_name = ctx.get_unique("output_ptr");
                ctx.buffer.push_str(&format!("(local ${} i32)\n", &output_ptr_name));
                source.generate(ctx, Box::new(|ctx, fields| {
                    let field_name = fields.get(field).unwrap();
                    ctx.gen_ctx.buffer.push_str(";; store output\n");
                    ctx.gen_ctx.buffer.push_str(&format!("(local.get ${})\n", &output_ptr_name));
                    ctx.gen_ctx.buffer.push_str(&format!("(local.get ${})\n", &field_name.0));
                    ctx.gen_ctx.buffer.push_str(&format!("(i32.store {})\n", memory_index));
                    ctx.gen_ctx.buffer.push_str(";; update output pointer\n");
                    ctx.gen_ctx.buffer.push_str(&format!("(local.get ${})\n", &output_ptr_name));
                    ctx.gen_ctx.buffer.push_str(&format!("(i32.const {})\n", 4));
                    ctx.gen_ctx.buffer.push_str(&format!("(i32.add)\n"));
                    ctx.gen_ctx.buffer.push_str(&format!("(local.set ${})\n", &output_ptr_name));
                }))?;
            }
            Node::Map(source, exprs, out_fields) => {
                ctx.buffer.push_str(";; declare map output\n");
                let field_names = out_fields.iter().enumerate().map(|(i, field)| {
                    let field_name = ctx.get_unique(field);
                    let field_type = exprs[i].value_type();
                    ctx.buffer.push_str(&format!("(local ${} i32)\n", &field_name));
                    (field_name, ValueMetadata{ value_type: ValueType::Int, nullable: false })
                }).collect::<Vec<_>>();
                let field_mapping = out_fields.iter().zip(field_names.iter()).map(|(a, b)| (a.to_string(), (b.0.to_string(), b.1.clone()))).collect::<VariableMapping>();
                source.generate(ctx, Box::new(|ctx, fields| {
                    for (i, field) in out_fields.iter().enumerate() {
                        ctx.gen_ctx.buffer.push_str(&format!(";; evaluate {}\n", field));
                        exprs[i].generate(ctx.gen_ctx, fields);
                        ctx.gen_ctx.buffer.push_str(&format!("(local.set ${})\n", &field_names[i].0));
                    }
                    produce(ctx, &field_mapping);
                }))?;
            }
            Node::Sum(source, field) => {
                ctx.buffer.push_str(";; declare sum output\n");
                let sum_name = ctx.get_unique("sum");
                ctx.buffer.push_str(&format!("(local ${} i32)\n", &sum_name));
                source.generate(ctx, Box::new(|ctx, fields| {
                    ctx.gen_ctx.buffer.push_str(";; add to sum\n");
                    let field_name = fields.get(field).unwrap();
                    ctx.gen_ctx.buffer.push_str(&format!("(local.get ${})\n", &sum_name));
                    ctx.gen_ctx.buffer.push_str(&format!("(local.get ${})\n", &field_name.0));
                    ctx.gen_ctx.buffer.push_str(&format!("(i32.add)\n"));
                    ctx.gen_ctx.buffer.push_str(&format!("(local.set ${})\n", &sum_name));
                }))?;
                let mut out_field_mapping = HashMap::new();
                out_field_mapping.insert(field.to_string() + "_sum", (sum_name, ValueMetadata{ value_type: ValueType::Int, nullable: false }));
                produce(&mut ProduceContext {
                    continue_label: "<unreachable>".to_string(), // TODO: Should also come in GenContext.
                    gen_ctx: ctx,
                }, &out_field_mapping);
            }
        }
        Ok(())
    }
}

enum Expr {
    Variable(String),
    // Constant(Value),
    Add(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn generate(&self, ctx: &mut GenContext, fields: &VariableMapping) -> Result<()> {
        match self {
            Expr::Variable(name) => {
                let field_name = fields.get(name).unwrap();
                ctx.buffer.push_str(";; load variable\n");
                ctx.buffer.push_str(&format!("(local.get ${})\n", &field_name.0));
            }
            Expr::Add(left, right) => {
                ctx.buffer.push_str(";; add\n");
                left.generate(ctx, fields)?;
                right.generate(ctx, fields)?;
                ctx.buffer.push_str("(i32.add)\n");
            }
        }
        Ok(())
    }

    fn value_type(&self) -> Result<ValueType> {
        Ok(match self {
            Expr::Variable(_) => ValueType::Int,
            Expr::Add(_, _) => ValueType::Int,
        })
    }
}

struct GenContext {
    unique_name_number: usize,
    buffer: String,
}

impl GenContext {
    fn get_unique(&mut self, name: &str) -> String {
        let out = format!("{}_{}", name, self.unique_name_number);
        self.unique_name_number += 1;
        out
    }
}

struct ProduceContext<'a> {
    continue_label: String,
    gen_ctx: &'a mut GenContext,
}

fn main() -> Result<()> {
    let plan = Node::Range(3, 1000000000);
    let plan = Node::Map(
        Box::new(plan),
        vec![
            Expr::Add(
                Box::new(Expr::Variable("i".to_string())),
                Box::new(Expr::Variable("i".to_string())),
            ),
            Expr::Add(
                Box::new(Expr::Variable("i".to_string())),
                Box::new(Expr::Variable("i".to_string())),
            ),
        ],
        vec!["a".to_string(), "b".to_string()],
    );
    let plan = Node::Sum(
        Box::new(plan),
        "a".to_string(),
    );
    let plan = Node::Output(Box::new(plan), "a_sum".to_string(), 1);

    let mut gen_ctx = GenContext {
        unique_name_number: 0,
        buffer: String::new(),
    };
    gen_ctx.buffer.push_str("(module\n");
    gen_ctx.buffer.push_str(r#"(import "env" "input" (memory 0))
(memory (export "memory") 0)
(func (export "execute")
"#);

    plan.generate(&mut gen_ctx, Box::new(|ctx, fields| {}));

    gen_ctx.buffer.push_str(")\n");
    gen_ctx.buffer.push_str(")\n");

    let mut file = File::create("out.wat")?;
    file.write_all(gen_ctx.buffer.as_bytes())?;

    // Create our `store_fn` context and then compile a module and create an
    // instance from the compiled module all in one go.
    let mut config = Config::default();
    config.wasm_multi_memory(true);
    let mut store: Store<()> = Store::new(&Engine::new(&config).unwrap(), ());
    let start = SystemTime::now();
    // let module = Module::from_binary(store.engine(), gen_ctx.buffer.as_bytes())?;
    let module = Module::from_file(store.engine(), "out.wat")?;
    println!("Module compiled in {:?}", start.elapsed()?);
    let input_memory = Memory::new(&mut store, MemoryType::new(64, None)).unwrap();
    // let start_pointer = 4 * 2;
    // let end_pointer = start_pointer + 4 * count;
    // (&mut input_memory.data_mut(&mut store)[0..4]).copy_from_slice(&(start_pointer as u32).to_le_bytes());
    // (&mut input_memory.data_mut(&mut store)[4..8]).copy_from_slice(&(end_pointer as u32).to_le_bytes());
    // for i in (0 as usize)..(count as usize) {
    //     (&mut input_memory.data_mut(&mut store)[start_pointer + i * 4..start_pointer + (i + 1) * 4]).copy_from_slice(&(i as u32).to_le_bytes());
    // }

    let instance = Instance::new(&mut store, &module, &[
        Extern::Memory(input_memory),
    ])?;
    println!("Instance created in {:?}", start.elapsed()?);

    // load_fn up our exports from the instance
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or(anyhow::format_err!("failed to find `memory` export"))?;
    memory.grow(&mut store, 4)?;

    let execute_fn = instance.get_typed_func::<(), ()>(&mut store, "execute")?;

    println!("Executing after {:?}", start.elapsed()?);
    execute_fn.call(&mut store, ())?;
    println!("Executed once after {:?}", start.elapsed()?);

    let output_memory = instance.exports(&mut store).next().unwrap().into_memory().unwrap();

    // assert_eq!(u32::from_le_bytes(memory.data(&store)[0..4].try_into().unwrap()), 4*10_000_000);
    // assert_eq!(sum, 45000000);
    // println!("Memory state: {:?}", input_memory.data(&store)[0..16].to_vec());
    println!("Memory state: {:?}", output_memory.data(&store)[0..16].to_vec());
    assert_eq!(u32::from_le_bytes(output_memory.data(&store)[4..8].try_into().unwrap()), 6);

    Ok(())
}
