use extism_pdk::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct InData {
    data: f64,
}

#[plugin_fn]
pub fn handle(input: String) -> FnResult<String> {
    run_handle(input)
}

fn run_handle(input: String) -> FnResult<String> {
    let data:InData = serde_json::from_str(&input).unwrap();
    Ok((data.data * 2.0).to_string())
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