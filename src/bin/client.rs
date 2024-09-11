use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use opentelemetry::{
    global,
    trace::{Tracer, TracerProvider as _},
};
use opentelemetry_sdk::trace::TracerProvider;
use pyo3::{
    prelude::{Py, PyAny, PyModule, PyResult, Python},
    types::{PyAnyMethods, PyTuple},
    Bound,
};
use std::error::Error;
use std::time::Duration;
use tokio::time;
use tonic::transport::Channel;
use tonic::Request;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

async fn say_hello(
    client: &mut GreeterClient<Channel>,
    llm: Bound<'_, PyAny>,
    sp: Bound<'_, PyAny>,
    generate: Bound<'_, PyAny>,
) -> Result<(), Box<dyn Error>> {
    let start = time::Instant::now();

    let outbound = async_stream::stream! {
        let mut interval = time::interval(Duration::from_secs(30));

        while let time = interval.tick().await {
            let elapsed = time.duration_since(start);
            if elapsed > Duration::from_secs(60) {
                break;
            }
            let req = HelloRequest {
                name: format!("Adam {:?}", elapsed),
            };
            yield req;
        }
    };

    let response = client.say_hello(Request::new(outbound)).await?;
    let mut inbound = response.into_inner();

    while let Some(output) = inbound.message().await? {
        let joke: String = generate.call((llm.clone(), sp.clone()), None)?.extract()?;
        println!("OUTPUT = {:?}", output);
        println!("JOKE =\n{}", joke);
    }

    Ok(())
}

// #[tokio::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    let tracer = provider.tracer("draft-sniffle");

    let pycode = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/py/generate.py"));

    Python::with_gil(|py| -> PyResult<()> {
        py.run_bound(
            r#"
import sys
print(f'executable: {sys.executable}')
print(f'path: {sys.path}')
"#,
            None,
            None,
        )
        .unwrap();
        println!("=============================");

        let pymodule = PyModule::from_code_bound(py, pycode, "draft_sniffle", "draft_sniffle")?;
        let initialize: Bound<PyAny> = pymodule.getattr("initialize")?;
        let get_sampling_params: Bound<PyAny> = pymodule.getattr("get_sampling_params")?.into();
        let generate: Bound<PyAny> = pymodule.getattr("generate")?;

        let llm: Bound<PyAny> = initialize.call0()?;
        let sp: Bound<PyAny> = get_sampling_params.call0()?;
        // let output: String = generate.call((llm, sp), None)?.extract()?;

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut client = GreeterClient::connect("http://[::1]:50051").await.unwrap();
            say_hello(&mut client, llm, sp, generate).await.unwrap();
        });

        Ok(())
    })?;

    global::shutdown_tracer_provider();
    Ok(())
}
