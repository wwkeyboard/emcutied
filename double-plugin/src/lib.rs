use extism_pdk::*;
use serde::Deserialize;

// The host function
extern "C" {
    fn emit(data: u64) -> ();
}

#[derive(Deserialize)]
struct InData {
    data: f64,
}

#[plugin_fn]
pub fn handle(input: String) -> FnResult<String> {
    run_handle(input)
}

fn run_handle(input: String) -> FnResult<String> {
    let data: InData = serde_json::from_str(&input).unwrap();

    let payload = format!("{{\"data\": {}}}", data.data * 2.0);
    let memory = Memory::from_bytes(&payload);

    unsafe {
        emit(memory.offset as u64);
    }

    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let test = run_handle(r#"{"data": 2 }"#.to_owned()).unwrap();
        assert_eq!("4", test);
    }
}
