use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Payload {
    data: f64,
}

#[plugin_fn]
pub fn handle(input: String) -> FnResult<String> {
    run_handle(input)
}

fn run_handle(input: String) -> FnResult<String> {
    let data: Payload = serde_json::from_str(&input).unwrap();

    let payload = Payload {
        data: data.data * 2.0,
    };

    let response = serde_json::to_string(&payload).unwrap();
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
