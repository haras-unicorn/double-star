#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![deny(clippy::todo)]
#![deny(clippy::unreachable)]
#![deny(clippy::allow_attributes_without_reason)]

pub mod config;

use candle_core::{DType, Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::mixformer::Config;
use candle_transformers::models::quantized_mixformer::MixFormerSequentialForCausalLM as QMixFormer;
use hf_hub::api::sync::Api;
use hf_hub::Repo;
use tokenizers::Tokenizer;

#[tokio::main]
pub async fn run(
  tx: flume::Sender<gravity::DoubleStarMessage>,
  rx: flume::Receiver<gravity::OrbitusMessage>,
  _config: config::Config,
  _config_rx: flume::Receiver<gravity::config::ConfigUpdate<config::Config>>,
) -> anyhow::Result<()> {
  let device = match Device::cuda_if_available(0) {
    Ok(cuda) => {
      tracing::info!("Using CUDA");
      cuda
    }
    Err(err) => {
      tracing::warn!("Using CPU because {err}");
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
    Err(_err) => return Err(anyhow::anyhow!("Failed getting tokenizer")),
  };

  let vb =
    match candle_transformers::quantized_var_builder::VarBuilder::from_gguf(
      &repo.get("model-v2-q4k.gguf")?,
      &device,
    ) {
      Ok(vars) => vars,
      Err(_err) => return Err(anyhow::anyhow!("I failed in life")),
    };

  let model_config = Config::v2();
  let mut model = QMixFormer::new_v2(&model_config, vb)?;

  let mut logits_processor = LogitsProcessor::new(rand::random(), None, None);

  loop {
    let prompt = match rx.recv_async().await? {
      gravity::OrbitusMessage::Submit(prompt) => prompt,
      gravity::OrbitusMessage::Exited => {
        break;
      }
    };

    let tokenizer_output = match tokenizer.encode(prompt, true) {
      Ok(result) => result,
      Err(_err) => return Err(anyhow::anyhow!("Tokenizer output bad")),
    };
    let mut tokens = tokenizer_output.get_ids().to_vec();

    loop {
      let input = Tensor::new(tokens.clone(), &device)?.unsqueeze(0)?;
      tracing::debug!("input {}", input);

      model.clear_kv_cache();
      let logits = model.forward(&input)?;
      tracing::debug!("logits {}", logits);

      let processed = logits.to_dtype(DType::F32)?.squeeze(0)?;
      tracing::debug!("processed logits {}", processed);

      let next_token = logits_processor.sample(&processed)?;

      let next_word = match tokenizer.decode(&[next_token], false) {
        Ok(word) => word,
        Err(_err) => return Err(anyhow::anyhow!("Next word bad")),
      };
      tracing::info!("Generated text: {}", next_word);

      tx.send_async(gravity::DoubleStarMessage::Generated(next_word.clone()))
        .await?;

      if next_word == "." {
        tx.send_async(gravity::DoubleStarMessage::Break).await?;
        break;
      }

      let tokenizer_output = match tokenizer.encode(next_word, true) {
        Ok(result) => result,
        Err(_err) => return Err(anyhow::anyhow!("Tokenizer output bad")),
      };
      tokens.extend(tokenizer_output.get_ids());
    }
  }

  Ok(())
}
