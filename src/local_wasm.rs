use crate::{Error as OpaError, OpenPolicyAgentClient};
use async_trait::async_trait;
use policy_evaluator::burrego::Evaluator;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

pub struct OpenPolicyAgentWasmClient<'a> {
    entry_point: &'a str,
    evaluator: Evaluator,
}

impl<'a> OpenPolicyAgentWasmClient<'a> {
    pub fn new(wasm: &'a [u8], entry_point: &'a str) -> Self {
        Self {
            entry_point,
            evaluator: Evaluator::new(wasm, Default::default()).unwrap(),
        }
    }
}

#[async_trait(?Send)]
impl<'a> OpenPolicyAgentClient for OpenPolicyAgentWasmClient<'a> {
    async fn query<I, D, O>(&mut self, input: &I, data: &D) -> Result<Option<O>, OpaError>
    where
        I: Serialize,
        D: Serialize,
        O: DeserializeOwned,
    {
        println!("Evaluating:");

        let entrypoint_id = self.evaluator.entrypoint_id(&self.entry_point).unwrap();

        let r: serde_json::Value = self
            .evaluator
            .evaluate(
                entrypoint_id,
                &serde_json::value::to_value(input).unwrap(),
                &serde_json::value::to_value(data).unwrap(),
            )
            .unwrap();

        println!("value: {:?}", r);
        let val = r[0].get("result").unwrap();
        Ok(Some(<O as Deserialize>::deserialize(val).unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;
    use std::env;
    use std::fs;
    use std::path::Path;

    #[tokio::test(flavor = "multi_thread")]
    async fn local_wasm_query_test() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let example_path = Path::new(&manifest_dir).join("example");
        let wasm_path = Path::new(&example_path).join("license.wasm");
        let wasm: [u8; 131650] = to_array(fs::read(wasm_path).unwrap());
        let mut client = OpenPolicyAgentWasmClient::new(&wasm, "license/allow");

        let input_str =
            fs::read_to_string(Path::new(&example_path).join("licenses-input.txt")).unwrap();
        let input: serde_json::Value = serde_json::from_str(&input_str).unwrap();
        println!("Input: {}", input);
        let data: serde_json::Value =
            serde_json::from_str("{}").expect("data json does not have correct format.");

        let result: Result<Option<bool>, OpaError> = client.query(&input, &data).await;
        println!("Result: {:?}", result);

        assert_eq!(1, 1);
    }

    fn to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
        v.try_into().unwrap_or_else(|v: Vec<T>| {
            panic!("Incorrect vector length: {}, expected: {}", N, v.len())
        })
    }
}
