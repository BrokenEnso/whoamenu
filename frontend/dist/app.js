const promptEl = document.getElementById('prompt');
const queryEl = document.getElementById('query');
const resultsEl = document.getElementById('results');

let currentState = {
  filteredItems: [],
  selectedIndex: -1,
};

function emit(event, payload) {
  if (payload === undefined) {
    window.runtime.EventsEmit(event);
    return;
  }
  window.runtime.EventsEmit(event, payload);
}

function render(state) {
  currentState = state;
  promptEl.textContent = state.promptText;
  queryEl.style.fontSize = `${state.fontSize}px`;
  resultsEl.style.fontSize = `${state.fontSize}px`;

  if (queryEl.value !== state.input) {
    queryEl.value = state.input;
  }

  resultsEl.innerHTML = '';
  state.filteredItems.forEach((item, index) => {
    const li = document.createElement('li');
    li.textContent = item;
    if (index === state.selectedIndex) {
      li.classList.add('selected');
    }
    li.addEventListener('mousedown', (e) => {
      e.preventDefault();
      emit('click-selection', index);
    });
    resultsEl.appendChild(li);
  });
}

queryEl.addEventListener('input', () => emit('query-change', queryEl.value));
queryEl.addEventListener('keydown', (e) => {
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    emit('move-selection', 1);
    return;
  }
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    emit('move-selection', -1);
    return;
  }
  if (e.key === 'Enter') {
    e.preventDefault();
    emit('accept');
    return;
  }
  if (e.key === 'Escape') {
    e.preventDefault();
    emit('cancel');
  }
});

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    e.preventDefault();
    emit('cancel');
  }
});

window.runtime.EventsOn('state', (state) => {
  render(state);
});

window.addEventListener('DOMContentLoaded', () => {
  queryEl.focus();
  emit('frontend-ready');
});
