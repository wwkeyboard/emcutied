use extism_pdk::*;

#[plugin_fn]
pub fn double(input: String) -> FnResult<String> {
    if let Ok(num) = input.parse::<f64>() {
        Ok(format!("{}", num * 2.0))
    } else {
        // YOLO
        Ok("couldn't parse input".to_string())
    }
}
