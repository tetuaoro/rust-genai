use crate::adapter::{Adapter, AdapterDispatcher, AdapterKind, ServiceType, WebRequestData};
use crate::chat::{ChatRequest, ChatRequestOptions, ChatRequestOptionsSet, ChatResponse, ChatStreamResponse};
use crate::client::Client;
use crate::{ConfigSet, Result};

/// Public AI Functions
impl Client {
	pub async fn list_models(&self, adapter_kind: AdapterKind) -> Result<Vec<String>> {
		AdapterDispatcher::list_models(adapter_kind).await
	}

	/// Resolve the adapter kind for a given model name.
	pub fn resolve_adapter_kind(&self, model: &str) -> Result<AdapterKind> {
		let adapter_kind_from_resolver = self
			.config()
			.adapter_kind_resolver()
			.map(|r| r.resolve(model))
			.transpose()?
			.flatten();

		match adapter_kind_from_resolver {
			Some(adapter_kind) => Ok(adapter_kind),
			None => AdapterKind::from_model(model),
		}
	}

	pub async fn exec_chat(
		&self,
		model: &str,
		chat_req: ChatRequest,
		// options not implemented yet
		options: Option<&ChatRequestOptions>,
	) -> Result<ChatResponse> {
		let adapter_kind = self.resolve_adapter_kind(model)?;

		let adapter_config = self
			.custom_adapter_config(adapter_kind)
			.unwrap_or_else(|| AdapterDispatcher::default_adapter_config(adapter_kind));

		let config_set = ConfigSet::new(self.config(), adapter_config);

		let options_set = ChatRequestOptionsSet::default()
			.with_chat_options(options)
			.with_client_options(self.config().default_chat_request_options());

		let WebRequestData { headers, payload, url } = AdapterDispatcher::to_web_request_data(
			adapter_kind,
			&config_set,
			ServiceType::Chat,
			model,
			chat_req,
			options_set,
		)?;

		let web_res = self.web_client().do_post(&url, &headers, payload).await?;

		let chat_res = AdapterDispatcher::to_chat_response(adapter_kind, web_res)?;

		Ok(chat_res)
	}

	pub async fn exec_chat_stream(
		&self,
		model: &str,
		chat_req: ChatRequest, // options not implemented yet
		options: Option<&ChatRequestOptions>,
	) -> Result<ChatStreamResponse> {
		let adapter_kind = self.resolve_adapter_kind(model)?;

		let adapter_config = self
			.custom_adapter_config(adapter_kind)
			.unwrap_or_else(|| AdapterDispatcher::default_adapter_config(adapter_kind));

		let config_set = ConfigSet::new(self.config(), adapter_config);

		let options_set = ChatRequestOptionsSet::default()
			.with_chat_options(options)
			.with_client_options(self.config().default_chat_request_options());

		let WebRequestData { url, headers, payload } = AdapterDispatcher::to_web_request_data(
			adapter_kind,
			&config_set,
			ServiceType::ChatStream,
			model,
			chat_req,
			options_set,
		)?;

		let reqwest_builder = self.web_client().new_req_builder(&url, &headers, payload)?;

		AdapterDispatcher::to_chat_stream(adapter_kind, reqwest_builder)
	}
}
