use daken_rs::{KeyResult, TypingSession};
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let target = use_state(|| "タイピング".to_string());
    let session = use_state(|| TypingSession::new("タイピング"));
    let status = use_state(|| "Ready".to_string());

    let reset = {
        let target = target.clone();
        let session = session.clone();
        let status = status.clone();

        Callback::from(move |_| {
            session.set(TypingSession::new((*target).clone()));
            status.set("Ready".to_string());
        })
    };

    let on_target_input = {
        let target = target.clone();
        let session = session.clone();
        let status = status.clone();

        Callback::from(move |event: InputEvent| {
            let input = event.target_unchecked_into::<HtmlInputElement>();
            let next_target = input.value();

            target.set(next_target.clone());
            session.set(TypingSession::new(next_target));
            status.set("Ready".to_string());
        })
    };

    let on_target_keydown = Callback::from(|event: KeyboardEvent| {
        event.stop_propagation();
    });

    let on_keydown = {
        let session = session.clone();
        let status = status.clone();

        Callback::from(move |event: KeyboardEvent| {
            if event.ctrl_key() || event.meta_key() || event.alt_key() {
                return;
            }

            let key = event.key().to_ascii_lowercase();
            let Some(ch) = key.chars().next() else {
                return;
            };

            if key.chars().count() != 1 {
                return;
            }

            event.prevent_default();

            let mut next_session = (*session).clone();
            match next_session.input(ch) {
                KeyResult::Accepted => {
                    session.set(next_session);
                    status.set("Accepted".to_string());
                }
                KeyResult::Completed => {
                    session.set(next_session);
                    status.set("Completed".to_string());
                }
                KeyResult::Rejected => {
                    session.set(next_session);
                    status.set("Miss".to_string());
                }
            }
        })
    };

    let matcher = session.matcher();
    let progress = matcher.progress();
    let (confirmed, unconfirmed) = matcher.target_parts();
    let next_keys = matcher
        .next_keys()
        .into_iter()
        .map(|key| key.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let remaining = matcher
        .remaining_romaji_candidates()
        .into_iter()
        .take(6)
        .collect::<Vec<_>>()
        .join(" / ");

    html! {
        <main class="app" tabindex="0" onkeydown={on_keydown}>
            <section class="play">
                <div class="meta">
                    <label>
                        {"Target"}
                        <input
                            id="target"
                            value={(*target).clone()}
                            oninput={on_target_input}
                            onkeydown={on_target_keydown}
                        />
                    </label>
                    <button id="reset" type="button" onclick={reset}>{"Reset"}</button>
                </div>

                <div class="target">
                    <span class="confirmed">{confirmed.to_string()}</span>
                    <span>{unconfirmed.to_string()}</span>
                </div>
                <div class="typed">{if matcher.typed().is_empty() { " ".to_string() } else { matcher.typed().to_string() }}</div>
                <div class="status">{format!(
                    "{} / misses {} / progress {}/{}",
                    *status,
                    session.misses(),
                    progress.confirmed_target_chars,
                    progress.total_target_chars
                )}</div>
                <div class="next">{next_keys}</div>
                <div class="next">{remaining}</div>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
