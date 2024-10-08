#![deny(
  unsafe_code,
  // reason = "Let's just not do it"
)]
#![deny(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::unreachable,
  clippy::arithmetic_side_effects
  // reason = "We have to handle errors properly"
)]
#![deny(
  clippy::dbg_macro,
  // reason = "Use tracing instead"
)]

use anyhow::anyhow;
use candle_core::{DType, Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::mixformer::Config;
use candle_transformers::models::quantized_mixformer::MixFormerSequentialForCausalLM as QMixFormer;
use hf_hub::api::sync::Api;
use hf_hub::Repo;
use tokenizers::Tokenizer;
use tracing_subscriber::{
  layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[tokio::main]
#[tracing::instrument]
async fn main() -> anyhow::Result<()> {
  let format_layer = tracing_subscriber::fmt::layer();
  let (filter_layer, filter_handle) =
    tracing_subscriber::reload::Layer::new(build_tracing_filter("info")?);
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(format_layer)
    .try_init()?;

  // TODO: from config when loaded if needed
  let log_level = "info".to_string();
  filter_handle.modify(move |filter| {
    #[allow(clippy::unwrap_used)] // NOTE: static and env doesn't change
    let new_filter = build_tracing_filter(log_level.as_str()).unwrap();
    *filter = new_filter;
  })?;

  let device = match Device::cuda_if_available(0) {
    Ok(cuda) => {
      println!("Using CUDA");
      cuda
    }
    Err(err) => {
      println!("Using CPU because {err}");
      Device::Cpu
    }
  };

  let api = Api::new()?;
  let repo = api.repo(Repo::new(
    "lmz/candle-quantized-phi".to_string(),
    hf_hub::RepoType::Model,
  ));
  let tokenizer_filename = repo.get("tokenizer.json")?;
  let tokenizer = match Tokenizer::from_file(tokenizer_filename) {
    Ok(tokenizer) => tokenizer,
    Err(_err) => return Err(anyhow!("Failed getting tokenizer")),
  };

  let vb =
    match candle_transformers::quantized_var_builder::VarBuilder::from_gguf(
      &repo.get("model-v2-q4k.gguf")?,
      &device,
    ) {
      Ok(vars) => vars,
      Err(_err) => return Err(anyhow::anyhow!("I failed in life")),
    };

  let mut model = QMixFormer::new_v2(&Config::v2(), vb)?;

  let prompt = "Once upon a time";

  let tokenizer_output = match tokenizer.encode(prompt, true) {
    Ok(result) => result,
    Err(_err) => return Err(anyhow::anyhow!("Tokenizer output bad")),
  };
  let tokens = tokenizer_output.get_ids().to_vec();

  let input = Tensor::new(tokens, &device)?.unsqueeze(0)?;
  let logits = model.forward(&input)?;

  let mut logits_processor = LogitsProcessor::new(rand::random(), None, None);
  let casted = logits.to_dtype(DType::F32)?.squeeze(0)?;
  let next_token = logits_processor.sample(&casted)?;

  let next_word = match tokenizer.decode(&[next_token], false) {
    Ok(word) => word,
    Err(_err) => return Err(anyhow::anyhow!("Next word bad")),
  };

  println!("Generated text: {}", next_word);

  Ok(())
}

fn build_tracing_filter(level: &str) -> anyhow::Result<EnvFilter> {
  Ok(
    tracing_subscriber::EnvFilter::builder()
      .with_default_directive(tracing::level_filters::LevelFilter::WARN.into())
      .with_env_var("DOUBLE_STAR_LOG")
      .from_env()?
      .add_directive(format!("double-star={level}").parse()?),
  )
}
