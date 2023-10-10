use extism_pdk::*;
use serde::Deserialize;

// The host function
extern "C" {
    fn emit(data: i64) -> i64;
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
    let response = (data.data * 2.0).to_string();

    unsafe {
        emit(data.data as i64);
    }

    Ok(response)
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
