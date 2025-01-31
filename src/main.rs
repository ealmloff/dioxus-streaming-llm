use dioxus::prelude::*;
use futures::StreamExt;

fn main() {
    launch(app)
}

fn app() -> Element {
    let mut prompt = use_signal(String::new);
    let mut response = use_signal(String::new);

    rsx! {
        div { display: "flex", flex_direction: "column", width: "100vw",
            textarea {
                value: "{prompt}",
                wrap: "soft",
                oninput: move |e| {
                    prompt.set(e.value());
                }
            }
            button {
                onclick: move |_| {
                    async move {
                        let initial_prompt = prompt();
                        response.set("Thinking...".into());
                        if let Ok(stream) = mistral(initial_prompt).await {
                            let mut stream = stream.into_inner();
                            let mut first_token = true;
                            while let Some(Ok(text)) = stream.next().await {
                                if first_token {
                                    response.write().clear();
                                    first_token = false;
                                }
                                response.write().push_str(&text);
                            }
                        }
                    }
                },
                "Respond"
            }
            div {
                white_space: "pre-wrap",
                "Response:\n{response}"
            }
        }
    }
}

#[server(output = server_fn::codec::StreamingText)]
pub async fn mistral(text: String) -> Result<server_fn::codec::TextStream, ServerFnError> {
    use kalosm::language::*;
    use once_cell::sync::OnceCell;

    static MODEL: OnceCell<Llama> = OnceCell::new();

    let model = match MODEL.get() {
        Some(model) => model,
        None => {
            let model = Llama::new_chat().await.unwrap();
            let _ = MODEL.set(model);
            MODEL.get().unwrap()
        }
    };
    let mut chat = model.chat();

    let stream = chat.into_add_message(&text);

    Ok(server_fn::codec::TextStream::new(stream.map(Ok)))
}
