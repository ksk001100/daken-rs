use daken_rs::{KeyResult, RomajiInput};
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let target = use_state(|| "タイピング".to_string());
    let matcher = use_state(|| RomajiInput::new("タイピング"));
    let typed = use_state(String::new);
    let misses = use_state(|| 0usize);
    let status = use_state(|| "Ready".to_string());

    let reset = {
        let target = target.clone();
        let matcher = matcher.clone();
        let typed = typed.clone();
        let misses = misses.clone();
        let status = status.clone();

        Callback::from(move |_| {
            matcher.set(RomajiInput::new((*target).clone()));
            typed.set(String::new());
            misses.set(0);
            status.set("Ready".to_string());
        })
    };

    let on_target_input = {
        let target = target.clone();
        let matcher = matcher.clone();
        let typed = typed.clone();
        let misses = misses.clone();
        let status = status.clone();

        Callback::from(move |event: InputEvent| {
            let input = event.target_unchecked_into::<HtmlInputElement>();
            let next_target = input.value();

            target.set(next_target.clone());
            matcher.set(RomajiInput::new(next_target));
            typed.set(String::new());
            misses.set(0);
            status.set("Ready".to_string());
        })
    };

    let on_target_keydown = Callback::from(|event: KeyboardEvent| {
        event.stop_propagation();
    });

    let on_keydown = {
        let matcher = matcher.clone();
        let typed = typed.clone();
        let misses = misses.clone();
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

            let mut next_matcher = (*matcher).clone();
            match next_matcher.input(ch) {
                KeyResult::Accepted => {
                    matcher.set(next_matcher);
                    typed.set(format!("{}{}", *typed, ch));
                    status.set("Accepted".to_string());
                }
                KeyResult::Completed => {
                    matcher.set(next_matcher);
                    typed.set(format!("{}{}", *typed, ch));
                    status.set("Completed".to_string());
                }
                KeyResult::Rejected => {
                    misses.set(*misses + 1);
                    status.set("Miss".to_string());
                }
            }
        })
    };

    let next_keys = matcher
        .next_keys()
        .into_iter()
        .map(|key| key.to_string())
        .collect::<Vec<_>>()
        .join(" ");

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

                <div class="target">{(*target).clone()}</div>
                <div class="typed">{if typed.is_empty() { " ".to_string() } else { (*typed).clone() }}</div>
                <div class="status">{format!("{} / misses {}", *status, *misses)}</div>
                <div class="next">{next_keys}</div>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
