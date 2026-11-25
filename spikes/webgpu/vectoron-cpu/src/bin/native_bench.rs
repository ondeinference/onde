//! Native baseline. Usage:
//!   native-bench <model.gguf> <tokenizer.json> <bench.json>
//! Prints one JSON line per run (warmup runs are marked), like cpu-baseline.
use spike_vectoron_cpu::*;
use tokenizers::Tokenizer;
use vectoron::Qwen2Model;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let gguf = std::fs::read(&a[1]).unwrap();
    let tok = Tokenizer::from_file(&a[2]).unwrap();
    let cfg: serde_json::Value = serde_json::from_slice(&std::fs::read(&a[3]).unwrap()).unwrap();
    let t = std::time::Instant::now();
    let mut model = Qwen2Model::load(gguf).expect("load");
    eprintln!("load_ms={:.1}", t.elapsed().as_secs_f64() * 1000.0);
    let prompt = chatml(
        cfg["system"].as_str().unwrap(),
        cfg["user"].as_str().unwrap(),
    );
    let max_new = cfg["max_new_tokens"].as_u64().unwrap() as usize;
    let warm = cfg["warmup_runs"].as_u64().unwrap();
    let runs = cfg["measured_runs"].as_u64().unwrap();
    let t0 = std::time::Instant::now();
    let now = move || t0.elapsed().as_secs_f64() * 1000.0;
    for i in 0..(warm + runs) {
        let r = pollster::block_on(run(
            &mut model,
            &tok,
            &prompt,
            max_new,
            &now,
            false,
            || async {},
        ))
        .expect("run");
        let mut v = serde_json::to_value(&r).unwrap();
        v["device"] = "vectoron".into();
        v["warmup"] = (i < warm).into();
        println!("{v}");
    }
}
