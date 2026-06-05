const wasmUrl = "./pkg/daken_wasm_example.wasm";
const encoder = new TextEncoder();

const statusText = {
  0: "Accepted",
  1: "Completed",
  2: "Miss",
};

const { instance } = await WebAssembly.instantiateStreaming(fetch(wasmUrl), {});
const wasm = instance.exports;
const memory = wasm.memory;

const targetInput = document.querySelector("#target");
const targetView = document.querySelector("#targetView");
const typedView = document.querySelector("#typed");
const statusView = document.querySelector("#status");
const nextView = document.querySelector("#next");
const resetButton = document.querySelector("#reset");

let matcherId = 0;
let typed = "";
let misses = 0;

function createMatcher(target) {
  const bytes = encoder.encode(target);
  const ptr = wasm.alloc_bytes(bytes.length);
  new Uint8Array(memory.buffer, ptr, bytes.length).set(bytes);

  const id = wasm.matcher_new(ptr, bytes.length);
  wasm.dealloc_bytes(ptr, bytes.length);
  return id;
}

function reset() {
  matcherId = createMatcher(targetInput.value);
  typed = "";
  misses = 0;
  render("Ready");
}

function render(status) {
  targetView.textContent = targetInput.value;
  typedView.textContent = typed || " ";
  statusView.textContent = `${status} / misses ${misses}`;
  nextView.textContent = nextKeys().join(" ");
}

function nextKeys() {
  const mask = wasm.matcher_next_key_mask(matcherId);
  const keys = [];

  for (let i = 0; i < 26; i += 1) {
    if ((mask & (1 << i)) !== 0) {
      keys.push(String.fromCharCode("a".charCodeAt(0) + i));
    }
  }

  return keys;
}

window.addEventListener("keydown", (event) => {
  if (event.target === targetInput) {
    return;
  }

  if (event.ctrlKey || event.metaKey || event.altKey || event.key.length !== 1) {
    return;
  }

  event.preventDefault();

  const key = event.key.toLowerCase();
  const result = wasm.matcher_input(matcherId, key.codePointAt(0));

  if (result === 2) {
    misses += 1;
  } else {
    typed += key;
  }

  render(statusText[result]);
});

targetInput.addEventListener("input", reset);
resetButton.addEventListener("click", reset);

reset();
