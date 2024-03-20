use dioxus::prelude::*;
use futures::StreamExt;
use server_fn::codec::{StreamingText, TextStream};

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

#[server(output = StreamingText)]
pub async fn mistral(text: String) -> Result<TextStream, ServerFnError> {
    use kalosm_llama::prelude::*;
    use once_cell::sync::OnceCell;

    static MISTRAL: OnceCell<Llama> = OnceCell::new();

    let model = match MISTRAL.get() {
        Some(model) => model,
        None => {
            let model = Llama::new_chat().await.unwrap();
            let _ = MISTRAL.set(model);
            MISTRAL.get().unwrap()
        }
    };
    let markers = model.chat_markers().unwrap();
    let message = markers.system_prompt_marker.to_string()
        + "You are a helpful assistant who responds to user input with concise, helpful answers."
        + markers.end_system_prompt_marker
        + markers.user_marker
        + &text
        + markers.end_user_marker
        + markers.assistant_marker;

    let stream = model
        .stream_text(&message)
        .with_max_length(1000)
        .with_stop_on(markers.end_assistant_marker.to_string())
        .await
        .unwrap();

    Ok(TextStream::new(stream.map(Ok)))
}
