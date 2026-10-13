(() => {
  const { invoke } = window.__TAURI__.core;
  const dialog = window.__TAURI__.dialog;

  const $ = (id) => document.getElementById(id);
  const BARS = 14;
  const TICK_MS = 60;
  // Peak level above which the person is considered to be speaking.
  const VOICE_LEVEL = 0.03;
  // Reading pace of the karaoke highlight; it only moves while there is voice.
  const WORDS_PER_SECOND = { ru: 2.4, en: 2.8 };
  const AUDIO_EXTENSIONS = ["wav", "mp3", "m4a", "ogg", "oga", "opus", "webm", "flac"];

  const STRINGS = {
    ru: {
      newProfile: "Новый профиль",
      createProfile: "Создать новый профиль",
      namePlaceholder: "Имя профиля",
      ready: "готов",
      secondsOf: (have, need) => `${have} из ${need} с`,
      readAloud: "Читайте вслух",
      micHint: "Нажмите на микрофон и читайте текст",
      enough: "Речи достаточно — можно сохранять профиль",
      recording: (seconds) => `Идёт запись · ${seconds} с`,
      processing: "Обрабатываю запись…",
      rejected: (count) => `Отброшено непохожих отрезков: ${count}`,
      or: "или",
      uploadFiles: "Загрузить файлы",
      saveProfile: "Сохранить профиль",
      reset: "Начать заново",
      delete: "Удалить профиль",
      confirmDelete: (name) => `Удалить профиль «${name}»? Это нельзя отменить.`,
      readyMeta: (count, date) => `Векторов: ${count} · создан ${date}`,
      export: "Экспортировать .qvoice.json",
      exported: "Файл сохранён",
      verifyTitle: "Проверка",
      verifyHint: "Запишите новую фразу или выберите файл — покажу, насколько голос похож на профиль.",
      verifyMicHint: "Нажмите на микрофон и скажите что-нибудь",
      verifyFile: "Проверить файл",
      similarity: (centroid, best) =>
        `Близость к профилю: ${centroid} (к ближайшему отрезку: ${best})`,
      privacy:
        "Звук обрабатывается на этом компьютере и не сохраняется. Файл профиля — биометрические данные. Голос не заменяет пароль: запись или синтез голоса могут обмануть такую проверку.",
      audioFiles: "Аудио",
      profileFiles: "Голосовой профиль",
    },
    en: {
      newProfile: "New profile",
      createProfile: "Create a new profile",
      namePlaceholder: "Profile name",
      ready: "ready",
      secondsOf: (have, need) => `${have} of ${need} s`,
      readAloud: "Read aloud",
      micHint: "Press the microphone and read the text",
      enough: "Enough speech — you can save the profile",
      recording: (seconds) => `Recording · ${seconds} s`,
      processing: "Processing the recording…",
      rejected: (count) => `Dissimilar segments dropped: ${count}`,
      or: "or",
      uploadFiles: "Upload files",
      saveProfile: "Save profile",
      reset: "Start over",
      delete: "Delete profile",
      confirmDelete: (name) => `Delete the profile “${name}”? This can't be undone.`,
      readyMeta: (count, date) => `Vectors: ${count} · created ${date}`,
      export: "Export .qvoice.json",
      exported: "File saved",
      verifyTitle: "Check",
      verifyHint: "Record a new phrase or pick a file to see how close the voice is to the profile.",
      verifyMicHint: "Press the microphone and say something",
      verifyFile: "Check a file",
      similarity: (centroid, best) =>
        `Similarity to the profile: ${centroid} (closest segment: ${best})`,
      privacy:
        "Audio is processed on this computer and is not saved. A profile file is biometric data. A voice is not a password: a recording or a synthetic voice can fool this check.",
      audioFiles: "Audio",
      profileFiles: "Voice profile",
    },
  };

  let lang = "ru";
  try {
    if (localStorage.getItem("lang") === "en") lang = "en";
  } catch {
    // Storage may be unavailable; Russian stays the default.
  }

  let profiles = [];
  let selectedId = null;
  let naming = false;
  // { purpose: "enroll" | "verify", timer, history } while the microphone is on.
  let recording = null;
  let busy = false;
  // Reading position per language: { lines, line, word }.
  const reading = {};

  function t(key, ...args) {
    const value = STRINGS[lang][key];
    return typeof value === "function" ? value(...args) : value;
  }

  const selected = () => profiles.find((p) => p.id === selectedId) ?? null;

  // Karaoke text

  // Cuts the text into short lines: sentences, and long sentences at commas.
  function buildLines(text) {
    const lines = [];
    for (const paragraph of text.split("\n")) {
      const sentences = paragraph.match(/[^.!?…]+[.!?…]+[»”"]?\s*/g) ?? [paragraph];
      for (const sentence of sentences) {
        let current = [];
        for (const piece of sentence.match(/[^,;:]+[,;:]?\s*/g) ?? [sentence]) {
          current.push(...piece.trim().split(/\s+/).filter(Boolean));
          if (current.length >= 6) {
            lines.push(current);
            current = [];
          }
        }
        if (current.length >= 3 || lines.length === 0) lines.push(current);
        else lines[lines.length - 1].push(...current);
      }
    }
    return lines.filter((line) => line.length > 0);
  }

  function position() {
    reading[lang] ??= { lines: buildLines(window.READING_TEXTS[lang]), line: 0, word: 0 };
    return reading[lang];
  }

  function renderKaraoke() {
    const { lines, line, word } = position();
    const at = (index) => lines[(index + lines.length) % lines.length].join(" ");
    $("k-prev").textContent = line > 0 ? at(line - 1) : "";
    $("k-next").textContent = at(line + 1);
    const current = $("k-cur");
    current.replaceChildren();
    lines[line].forEach((text, index) => {
      const span = document.createElement("span");
      span.textContent = text + " ";
      if (index < Math.floor(word)) span.className = "read";
      current.append(span);
    });
  }

  function advanceReading(seconds) {
    const state = position();
    const before = `${state.line}:${Math.floor(state.word)}`;
    state.word += seconds * WORDS_PER_SECOND[lang];
    // One extra beat on the fully highlighted line before moving on.
    while (state.word >= state.lines[state.line].length + 1) {
      state.word -= state.lines[state.line].length + 1;
      state.line = (state.line + 1) % state.lines.length;
    }
    if (before !== `${state.line}:${Math.floor(state.word)}`) renderKaraoke();
  }

  // Voice visualization

  function buildWaves() {
    for (const wave of document.querySelectorAll(".wave")) {
      wave.replaceChildren(...Array.from({ length: BARS }, () => document.createElement("i")));
    }
  }

  // The newest level sits next to the microphone and travels outwards.
  function drawWaves(prefix, history) {
    const set = (bar, level) => (bar.style.height = `${8 + level * 92}%`);
    [...$(`${prefix}-wave-l`).children].forEach((bar, i) => set(bar, history[BARS - 1 - i]));
    [...$(`${prefix}-wave-r`).children].forEach((bar, i) => set(bar, history[i]));
  }

  // Rendering

  function renderSidebar() {
    const list = $("profiles");
    list.replaceChildren();
    for (const profile of profiles) {
      const item = document.createElement("button");
      item.className = profile.id === selectedId ? "item on" : "item";
      const name = document.createElement("b");
      name.textContent = profile.name;
      const status = document.createElement("small");
      status.textContent = profile.ready
        ? t("ready")
        : t("secondsOf", Math.floor(profile.speechSeconds), profile.targetSeconds);
      item.append(name, status);
      item.addEventListener("click", () => select(profile.id));
      list.append(item);
    }
    if (naming) {
      const input = document.createElement("input");
      input.className = "item-input";
      input.maxLength = 200;
      input.placeholder = t("namePlaceholder");
      input.addEventListener("keydown", (event) => {
        if (event.key === "Enter") confirmName(input.value);
        if (event.key === "Escape") cancelName();
      });
      input.addEventListener("blur", () => (input.value.trim() ? confirmName(input.value) : cancelName()));
      list.append(input);
      input.focus();
    }
    for (const button of document.querySelectorAll(".lang button")) {
      button.classList.toggle("on", button.dataset.lang === lang);
    }
  }

  function renderDraft(profile) {
    const have = Math.floor(profile.speechSeconds);
    $("draft-progress").textContent = t("secondsOf", have, profile.targetSeconds);
    const share = Math.min(1, profile.speechSeconds / profile.targetSeconds);
    $("draft-bar").style.width = `${share * 100}%`;
    $("draft-finish").hidden = !profile.canFinish;
    $("draft-mic").disabled = busy;
    $("draft-mic").classList.toggle("active", recording !== null);
    $("draft-files").disabled = busy || recording !== null;
    if (recording === null) {
      let hint = profile.canFinish ? t("enough") : t("micHint");
      if (busy) hint = t("processing");
      else if (profile.rejected > 0) hint += `. ${t("rejected", profile.rejected)}`;
      $("draft-hint").textContent = hint;
    }
    renderKaraoke();
  }

  function renderReady(profile) {
    $("ready-name").textContent = profile.name;
    const date = new Date(profile.createdAt).toLocaleDateString(lang);
    $("ready-meta").textContent = t("readyMeta", profile.segments, date);
    $("ready-mic").disabled = busy;
    $("ready-mic").classList.toggle("active", recording !== null);
    $("ready-file").disabled = busy || recording !== null;
    if (recording === null) $("ready-hint").textContent = busy ? t("processing") : t("verifyMicHint");
  }

  function render() {
    document.documentElement.lang = lang;
    for (const element of document.querySelectorAll("[data-i18n]")) {
      element.textContent = t(element.dataset.i18n);
    }
    renderSidebar();
    const profile = selected();
    $("view-empty").hidden = profile !== null;
    $("view-draft").hidden = profile === null || profile.ready;
    $("view-ready").hidden = profile === null || !profile.ready;
    if (profile === null) return;
    if (profile.ready) renderReady(profile);
    else renderDraft(profile);
  }

  function showError(error) {
    $("message").textContent = String(error);
    $("message").hidden = false;
  }

  async function run(action) {
    $("message").hidden = true;
    try {
      await action();
    } catch (error) {
      showError(error);
    }
    render();
  }

  // Replaces a profile in the list; saving a draft changes its id.
  function update(view, previousId = view.id) {
    profiles = profiles.map((p) => (p.id === previousId ? view : p));
    if (selectedId === previousId) selectedId = view.id;
  }

  // Profiles

  function select(id) {
    if (recording !== null || busy) return;
    selectedId = id;
    $("ready-result").hidden = true;
    $("message").hidden = true;
    render();
  }

  function startNaming() {
    if (recording !== null || busy) return;
    naming = true;
    render();
  }

  function cancelName() {
    if (!naming) return;
    naming = false;
    render();
  }

  function confirmName(name) {
    if (!naming) return;
    naming = false;
    run(async () => {
      const view = await invoke("create_profile", { name });
      profiles.push(view);
      selectedId = view.id;
    });
  }

  async function deleteSelected() {
    const profile = selected();
    if (!profile || recording !== null || busy) return;
    const confirmed = await dialog.confirm(t("confirmDelete", profile.name), {
      title: "Quant Voice Vector",
      kind: "warning",
    });
    if (!confirmed) return;
    await invoke("delete_profile", { id: profile.id });
    profiles = profiles.filter((p) => p.id !== profile.id);
    selectedId = null;
  }

  // Recording

  function showMatch(match) {
    $("ready-result").textContent = t(
      "similarity",
      match.centroid.toFixed(2),
      match.bestSegment.toFixed(2),
    );
    $("ready-result").hidden = false;
  }

  async function tick(prefix) {
    const progress = await invoke("recording_progress");
    if (!progress || recording === null) return;
    recording.history.unshift(Math.min(1, Math.sqrt(progress.peak) * 1.3));
    recording.history.pop();
    drawWaves(prefix, recording.history);
    $(`${prefix}-hint`).textContent = t("recording", Math.floor(progress.seconds));
    if (recording.purpose === "enroll" && progress.peak > VOICE_LEVEL) {
      advanceReading(TICK_MS / 1000);
    }
  }

  async function toggleRecording(purpose, prefix) {
    if (busy) return;
    if (recording === null) {
      await invoke("start_recording");
      recording = {
        purpose,
        history: new Array(BARS).fill(0),
        timer: setInterval(() => tick(prefix).catch(showError), TICK_MS),
      };
      return;
    }

    clearInterval(recording.timer);
    recording = null;
    busy = true;
    drawWaves(prefix, new Array(BARS).fill(0));
    render();
    try {
      const id = selectedId;
      if (purpose === "enroll") update(await invoke("stop_recording_enroll", { id }));
      else showMatch(await invoke("stop_recording_verify", { id }));
    } finally {
      busy = false;
    }
  }

  async function withBusy(action) {
    busy = true;
    render();
    try {
      await action();
    } finally {
      busy = false;
    }
  }

  // Events

  $("new-profile").addEventListener("click", startNaming);
  $("empty-create").addEventListener("click", startNaming);

  for (const button of document.querySelectorAll(".lang button")) {
    button.addEventListener("click", () => {
      lang = button.dataset.lang;
      try {
        localStorage.setItem("lang", lang);
      } catch {
        // The choice just won't survive a restart.
      }
      render();
    });
  }

  $("draft-mic").addEventListener("click", () => run(() => toggleRecording("enroll", "draft")));
  $("ready-mic").addEventListener("click", () => run(() => toggleRecording("verify", "ready")));

  $("draft-files").addEventListener("click", () =>
    run(async () => {
      const paths = await dialog.open({
        multiple: true,
        filters: [{ name: t("audioFiles"), extensions: AUDIO_EXTENSIONS }],
      });
      if (!paths) return;
      await withBusy(async () => update(await invoke("add_files", { id: selectedId, paths })));
    }),
  );

  $("draft-finish").addEventListener("click", () =>
    run(async () => {
      const previousId = selectedId;
      update(await invoke("finish_profile", { id: previousId }), previousId);
    }),
  );

  $("draft-reset").addEventListener("click", () =>
    run(async () => {
      if (recording !== null || busy) return;
      update(await invoke("reset_profile", { id: selectedId }));
    }),
  );

  $("draft-delete").addEventListener("click", () => run(deleteSelected));
  $("ready-delete").addEventListener("click", () => run(deleteSelected));

  $("ready-export").addEventListener("click", () =>
    run(async () => {
      const profile = selected();
      const path = await dialog.save({
        defaultPath: `${profile.name}.qvoice.json`,
        filters: [{ name: t("profileFiles"), extensions: ["json"] }],
      });
      if (!path) return;
      await invoke("export_profile", { id: profile.id, path });
      $("ready-result").textContent = t("exported");
      $("ready-result").hidden = false;
    }),
  );

  $("ready-file").addEventListener("click", () =>
    run(async () => {
      const path = await dialog.open({
        multiple: false,
        filters: [{ name: t("audioFiles"), extensions: AUDIO_EXTENSIONS }],
      });
      if (!path) return;
      await withBusy(async () => showMatch(await invoke("verify_file", { id: selectedId, path })));
    }),
  );

  buildWaves();
  run(async () => {
    profiles = await invoke("list_profiles");
    selectedId = profiles[0]?.id ?? null;
  });
})();
