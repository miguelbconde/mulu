// Mulu's window. Plain JavaScript, no framework and no npm package (decision D17).
// Everything shown comes from the application through a few commands; text from the
// books is always inserted as text, never as HTML.
"use strict";

// ---------------------------------------------------------------- Texts

// The language follows the operating system; English is the fallback. A translation is
// marked ready only after review by a native speaker (project rule).
const TEXTS = {
  en: {
    searchLabel: "Search the library",
    searchPlaceholder: "Search your books",
    searchButton: "Search",
    helpTitle: "Help",
    welcome: "Welcome to Mulu.",
    purpose: "Choose the folder where you keep your books. Mulu reads them and lets you search every word inside, on this computer only.",
    chooseFolder: "Choose the folder with your books",
    chooseNote: "Mulu only reads your files: it never changes, moves or deletes them.",
    promisesTitle: "Promises",
    offline: "Works entirely offline. No account, no ads, no data collection.",
    readOnly: "Only reads your files. Nothing is changed without your confirmation.",
    free: "Free software, under the GNU GPL, version 3 or later.",
    libraryLabel: "Library",
    hint1: "Type a word, a name or a concept and press Enter.",
    hint2: "Use quotes for an exact phrase (\"olhos de ressaca\") and a star for the beginning of a word (melancol*).",
    version: "Version",
    keys: "Ctrl+F: search · Enter or double click: open the file",
    problemsTitle: "Files that could not be read",
    problemsNote: "These files can still be found by their name.",
    close: "Close",
    finding: "Looking for books in the folder…",
    indexing: (done, total) => `Reading the books: ${n(done)} of ${n(total)}. You can already search what has been read.`,
    ready: (total) => total === 1 ? "1 file in the library." : `${n(total)} files in the library.`,
    problems: (count) => count === 1 ? "1 file could not be read" : `${n(count)} files could not be read`,
    summary: (hits, files, seconds) => `${n(hits)} ${hits === 1 ? "occurrence" : "occurrences"} in ${n(files)} ${files === 1 ? "file" : "files"} (${seconds} s)`,
    noResults: (q) => `Nothing found for ${q}.`,
    byName: "Found by the file name",
    showPassages: (count) => `Show the ${n(count)} passages`,
    hidePassages: "Hide the passages",
    more: (count) => `Show ${n(count)} more`,
    open: "Open",
    reveal: "Show in folder",
    page: (label) => `p. ${label}`,
    part: (number) => `part ${number}`,
    reasons: {
      Unsupported: "format not supported",
      Protected: "protected (password or copy protection)",
      Corrupted: "damaged file",
      TooLarge: "too large to read",
      NoText: "no text inside (perhaps scanned pages: text recognition comes in version 0.5)",
      Unreadable: "could not be opened",
      InCloud: "in the cloud, not downloaded to this computer",
      TimedOut: "took too long to read",
      ReaderStopped: "the reader stopped on this file",
    },
    help: [
      ["Searching", [
        "Type one or more words and press Enter. Files that contain all of them are listed, those with most occurrences first.",
        "Accents and capital letters do not matter: acao finds Ação.",
        "Between quotes, an exact phrase: \"olhos de ressaca\".",
        "A star at the end finds every word that begins that way: melancol* finds melancolia, melancólico, melancholy.",
        "Chinese and Japanese: type the characters; Mulu finds exactly that sequence.",
      ]],
      ["Results", [
        "Each file appears once, with the number of occurrences. The arrow shows the passages, with 25 words before and after and the page or chapter.",
        "Enter or double click opens the file in the program you usually use for it.",
        "Files whose name matches are listed after them, even when their text could not be read.",
      ]],
      ["Formats", [
        "PDF, EPUB, plain text, Markdown and HTML. More formats come in version 0.4; scanned pages in version 0.5.",
        "Files protected against copying (DRM), with a password, or kept only in the cloud are listed but not read.",
      ]],
      ["Your library and your privacy", [
        "Mulu only reads your files: it never changes, moves or deletes them.",
        "Mulu works without the internet and sends nothing anywhere. Its index is kept in its own folder on this computer.",
        "Files are read by a separate process, with time and memory limits, so a damaged file cannot freeze Mulu.",
      ]],
    ],
  },
  "pt-BR": {
    searchLabel: "Pesquisar na biblioteca",
    searchPlaceholder: "Pesquise nos seus livros",
    searchButton: "Pesquisar",
    helpTitle: "Ajuda",
    welcome: "Bem-vindo ao Mulu.",
    purpose: "Escolha a pasta onde você guarda seus livros. O Mulu os lê e permite pesquisar cada palavra que há neles, só neste computador.",
    chooseFolder: "Escolha a pasta dos seus livros",
    chooseNote: "O Mulu só lê os seus arquivos: nunca os altera, move ou apaga.",
    promisesTitle: "Compromissos",
    offline: "Funciona inteiramente sem internet. Sem conta, sem anúncios, sem coleta de dados.",
    readOnly: "Só lê os seus arquivos. Nada é alterado sem a sua confirmação.",
    free: "Software livre, sob a GPL da GNU, versão 3 ou posterior.",
    libraryLabel: "Biblioteca",
    hint1: "Digite uma palavra, um nome ou um conceito e tecle Enter.",
    hint2: "Use aspas para uma frase exata (\"olhos de ressaca\") e um asterisco para o começo de uma palavra (melancol*).",
    version: "Versão",
    keys: "Ctrl+F: pesquisar · Enter ou duplo clique: abrir o arquivo",
    problemsTitle: "Arquivos que não puderam ser lidos",
    problemsNote: "Esses arquivos ainda podem ser achados pelo nome.",
    close: "Fechar",
    finding: "Procurando livros na pasta…",
    indexing: (done, total) => `Lendo os livros: ${n(done)} de ${n(total)}. Já dá para pesquisar no que foi lido.`,
    ready: (total) => total === 1 ? "1 arquivo na biblioteca." : `${n(total)} arquivos na biblioteca.`,
    problems: (count) => count === 1 ? "1 arquivo não pôde ser lido" : `${n(count)} arquivos não puderam ser lidos`,
    summary: (hits, files, seconds) => `${n(hits)} ${hits === 1 ? "ocorrência" : "ocorrências"} em ${n(files)} ${files === 1 ? "arquivo" : "arquivos"} (${seconds} s)`,
    noResults: (q) => `Nada encontrado para ${q}.`,
    byName: "Encontrados pelo nome do arquivo",
    showPassages: (count) => `Mostrar os ${n(count)} trechos`,
    hidePassages: "Esconder os trechos",
    more: (count) => `Mostrar mais ${n(count)}`,
    open: "Abrir",
    reveal: "Mostrar na pasta",
    page: (label) => `p. ${label}`,
    part: (number) => `parte ${number}`,
    reasons: {
      Unsupported: "formato não suportado",
      Protected: "protegido (senha ou proteção contra cópia)",
      Corrupted: "arquivo danificado",
      TooLarge: "grande demais para ser lido",
      NoText: "sem texto (talvez páginas digitalizadas: o reconhecimento de texto chega na versão 0.5)",
      Unreadable: "não pôde ser aberto",
      InCloud: "na nuvem, não baixado neste computador",
      TimedOut: "demorou demais para ser lido",
      ReaderStopped: "o leitor parou neste arquivo",
    },
    help: [
      ["Pesquisar", [
        "Digite uma ou mais palavras e tecle Enter. Aparecem os arquivos que contêm todas elas, primeiro os que têm mais ocorrências.",
        "Acentos e maiúsculas não importam: acao encontra Ação.",
        "Entre aspas, uma frase exata: \"olhos de ressaca\".",
        "Um asterisco no fim encontra todas as palavras que começam assim: melancol* encontra melancolia, melancólico, melancholy.",
        "Chinês e japonês: digite os caracteres; o Mulu encontra exatamente essa sequência.",
      ]],
      ["Resultados", [
        "Cada arquivo aparece uma vez, com o número de ocorrências. A seta mostra os trechos, com 25 palavras antes e depois e a página ou o capítulo.",
        "Enter ou duplo clique abre o arquivo no programa que você costuma usar para ele.",
        "Os arquivos cujo nome combina com a pesquisa aparecem depois, mesmo quando o texto deles não pôde ser lido.",
      ]],
      ["Formatos", [
        "PDF, EPUB, texto simples, Markdown e HTML. Outros formatos chegam na versão 0.4; páginas digitalizadas, na versão 0.5.",
        "Arquivos protegidos contra cópia (DRM), com senha ou guardados só na nuvem são listados, mas não lidos.",
      ]],
      ["Sua biblioteca e sua privacidade", [
        "O Mulu só lê os seus arquivos: nunca os altera, move ou apaga.",
        "O Mulu funciona sem internet e não envia nada a lugar nenhum. O índice fica numa pasta própria, neste computador.",
        "Os arquivos são lidos por um processo separado, com limites de tempo e de memória, para que um arquivo danificado não trave o Mulu.",
      ]],
    ],
  },
  es: {
    searchLabel: "Buscar en la biblioteca",
    searchPlaceholder: "Busque en sus libros",
    searchButton: "Buscar",
    helpTitle: "Ayuda",
    welcome: "Bienvenido a Mulu.",
    purpose: "Elija la carpeta donde guarda sus libros. Mulu los lee y le permite buscar cada palabra que contienen, solo en esta computadora.",
    chooseFolder: "Elija la carpeta de sus libros",
    chooseNote: "Mulu solo lee sus archivos: nunca los modifica, mueve ni borra.",
    promisesTitle: "Compromisos",
    offline: "Funciona completamente sin internet. Sin cuenta, sin anuncios, sin recopilación de datos.",
    readOnly: "Solo lee sus archivos. Nada se modifica sin su confirmación.",
    free: "Software libre, bajo la GPL de GNU, versión 3 o posterior.",
    libraryLabel: "Biblioteca",
    hint1: "Escriba una palabra, un nombre o un concepto y pulse Intro.",
    hint2: "Use comillas para una frase exacta (\"olhos de ressaca\") y un asterisco para el comienzo de una palabra (melancol*).",
    version: "Versión",
    keys: "Ctrl+F: buscar · Intro o doble clic: abrir el archivo",
    problemsTitle: "Archivos que no se pudieron leer",
    problemsNote: "Estos archivos todavía se pueden encontrar por su nombre.",
    close: "Cerrar",
    finding: "Buscando libros en la carpeta…",
    indexing: (done, total) => `Leyendo los libros: ${n(done)} de ${n(total)}. Ya puede buscar en lo que se ha leído.`,
    ready: (total) => total === 1 ? "1 archivo en la biblioteca." : `${n(total)} archivos en la biblioteca.`,
    problems: (count) => count === 1 ? "1 archivo no se pudo leer" : `${n(count)} archivos no se pudieron leer`,
    summary: (hits, files, seconds) => `${n(hits)} ${hits === 1 ? "aparición" : "apariciones"} en ${n(files)} ${files === 1 ? "archivo" : "archivos"} (${seconds} s)`,
    noResults: (q) => `No se encontró nada para ${q}.`,
    byName: "Encontrados por el nombre del archivo",
    showPassages: (count) => `Mostrar los ${n(count)} fragmentos`,
    hidePassages: "Ocultar los fragmentos",
    more: (count) => `Mostrar ${n(count)} más`,
    open: "Abrir",
    reveal: "Mostrar en la carpeta",
    page: (label) => `p. ${label}`,
    part: (number) => `parte ${number}`,
    reasons: {
      Unsupported: "formato no compatible",
      Protected: "protegido (contraseña o protección contra copia)",
      Corrupted: "archivo dañado",
      TooLarge: "demasiado grande para leerlo",
      NoText: "sin texto (quizás páginas escaneadas: el reconocimiento de texto llega en la versión 0.5)",
      Unreadable: "no se pudo abrir",
      InCloud: "en la nube, no descargado en esta computadora",
      TimedOut: "tardó demasiado en leerse",
      ReaderStopped: "el lector se detuvo en este archivo",
    },
    help: [
      ["Buscar", [
        "Escriba una o más palabras y pulse Intro. Aparecen los archivos que contienen todas, primero los que tienen más apariciones.",
        "Los acentos y las mayúsculas no importan: accion encuentra Acción.",
        "Entre comillas, una frase exacta: \"olhos de ressaca\".",
        "Un asterisco al final encuentra todas las palabras que empiezan así: melancol* encuentra melancolía, melancólico, melancholy.",
        "Chino y japonés: escriba los caracteres; Mulu encuentra exactamente esa secuencia.",
      ]],
      ["Resultados", [
        "Cada archivo aparece una vez, con el número de apariciones. La flecha muestra los fragmentos, con 25 palabras antes y después y la página o el capítulo.",
        "Intro o doble clic abre el archivo en el programa que usa habitualmente para él.",
        "Los archivos cuyo nombre coincide aparecen después, incluso si su texto no se pudo leer.",
      ]],
      ["Formatos", [
        "PDF, EPUB, texto plano, Markdown y HTML. Más formatos en la versión 0.4; páginas escaneadas en la versión 0.5.",
        "Los archivos protegidos contra copia (DRM), con contraseña o guardados solo en la nube se listan, pero no se leen.",
      ]],
      ["Su biblioteca y su privacidad", [
        "Mulu solo lee sus archivos: nunca los modifica, mueve ni borra.",
        "Mulu funciona sin internet y no envía nada a ninguna parte. Su índice se guarda en una carpeta propia, en esta computadora.",
        "Los archivos se leen en un proceso separado, con límites de tiempo y de memoria, para que un archivo dañado no bloquee Mulu.",
      ]],
    ],
  },
};

function pickLanguage(preferred) {
  for (const tag of preferred) {
    const lower = tag.toLowerCase();
    if (lower.startsWith("pt")) return "pt-BR";
    if (lower.startsWith("es")) return "es";
    if (lower.startsWith("en")) return "en";
  }
  return "en";
}

const language = pickLanguage(navigator.languages || [navigator.language || "en"]);
const T = TEXTS[language];
const numberFormat = new Intl.NumberFormat(language);
function n(value) { return numberFormat.format(value); }

document.documentElement.lang = language;
for (const element of document.querySelectorAll("[data-i18n]")) {
  const text = T[element.dataset.i18n];
  if (typeof text === "string") element.textContent = text;
}
for (const element of document.querySelectorAll("[data-i18n-placeholder]")) {
  element.placeholder = T[element.dataset.i18nPlaceholder];
}
for (const element of document.querySelectorAll("[data-i18n-title]")) {
  element.title = T[element.dataset.i18nTitle];
  element.setAttribute("aria-label", T[element.dataset.i18nTitle]);
}

// ---------------------------------------------------------------- Helpers

const $ = (id) => document.getElementById(id);

/** Calls a command of the application. */
function call(command, args) {
  return window.__TAURI_INTERNALS__.invoke(command, args || {});
}

/** Creates an element with text (never HTML) and optional class. */
function el(tag, text, className) {
  const element = document.createElement(tag);
  if (text !== undefined && text !== null) element.textContent = text;
  if (className) element.className = className;
  return element;
}

function reason(problem) {
  if (!problem) return "";
  const kind = typeof problem === "string" ? problem : Object.keys(problem)[0];
  return T.reasons[kind] || kind;
}

/** The place of a passage, from its code: "p:12", "s:Chapter", "n:3" or "". */
function place(code) {
  if (!code) return "";
  const value = code.slice(2);
  if (code.startsWith("p:")) return T.page(value);
  if (code.startsWith("n:")) return T.part(value);
  return value;
}

// ---------------------------------------------------------------- Library and indexing

let polling = null;

async function refreshStatus() {
  let status;
  try {
    status = await call("status");
  } catch (error) {
    console.error(error);
    return;
  }
  const hasLibrary = Boolean(status.library);
  $("welcome").hidden = hasLibrary;
  $("library").hidden = !hasLibrary;
  $("search-form").hidden = !hasLibrary;
  if (!hasLibrary) return;
  $("library-path").textContent = status.library;

  const progress = status.progress;
  $("indexing").hidden = !progress;
  $("ready-text").hidden = Boolean(progress);
  if (progress) {
    if (progress.total === 0) {
      $("indexing-text").textContent = T.finding;
      $("indexing-bar").removeAttribute("value");
    } else {
      $("indexing-text").textContent = T.indexing(progress.done, progress.total);
      $("indexing-bar").max = progress.total;
      $("indexing-bar").value = progress.done;
    }
    if (!polling) polling = setInterval(refreshStatus, 1000);
  } else {
    clearInterval(polling);
    polling = null;
    $("ready-text").textContent = T.ready(status.files);
  }
  $("problems-button").hidden = status.problems === 0;
  $("problems-button").textContent = T.problems(status.problems);
}

$("choose").addEventListener("click", async () => {
  $("choose").disabled = true;
  try {
    const chosen = await call("choose_folder");
    if (chosen) {
      await refreshStatus();
      $("query").focus();
    }
  } finally {
    $("choose").disabled = false;
  }
});

// ---------------------------------------------------------------- Search

const PASSAGES_FIRST = 10;
const PASSAGES_MORE = 20;
let currentQuery = "";

$("search-form").addEventListener("submit", (event) => {
  event.preventDefault();
  search($("query").value);
});

async function search(query) {
  query = query.trim();
  currentQuery = query;
  const results = $("results");
  if (!query) {
    results.replaceChildren();
    $("start-hint").hidden = false;
    return;
  }
  const answer = await call("search", { query });
  if (!answer || query !== currentQuery) return;
  $("start-hint").hidden = true;
  results.replaceChildren();

  if (answer.files.length === 0 && answer.by_name.length === 0) {
    results.append(el("p", T.noResults(`“${query}”`), "summary"));
    return;
  }
  if (answer.files.length > 0) {
    const seconds = (answer.elapsed_ms / 1000).toLocaleString(language, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
    results.append(el("p", T.summary(answer.total, answer.files.length, seconds), "summary"));
    const list = el("ol", null, "files");
    for (const file of answer.files) list.append(fileItem(file, query));
    results.append(list);
  }
  if (answer.by_name.length > 0) {
    results.append(el("h2", T.byName, "label by-name"));
    const list = el("ol", null, "files");
    for (const file of answer.by_name) list.append(fileItem(file, null));
    results.append(list);
  }
}

/** One file of the results: name, folder, count, and its passages. */
function fileItem(file, query) {
  const item = el("li", null, "file");
  const row = el("div", null, "file-row");
  row.tabIndex = 0;
  row.title = T.open;
  row.addEventListener("dblclick", () => openFile(file.id, false));
  row.addEventListener("keydown", (event) => {
    if (event.key === "Enter") openFile(file.id, false);
  });

  const names = el("div", null, "file-names");
  names.append(el("span", file.name, "file-name"));
  const details = el("span", file.folder, "file-folder");
  if (file.problem) details.append(el("span", ` · ${reason(file.problem)}`, "file-problem"));
  names.append(details);
  row.append(names);
  if (file.count > 0) row.append(el("span", n(file.count), "count"));

  const reveal = el("button", T.reveal, "link small");
  reveal.type = "button";
  reveal.addEventListener("click", (event) => {
    event.stopPropagation();
    openFile(file.id, true);
  });
  row.append(reveal);
  item.append(row);

  if (query && file.count > 0) {
    const passages = el("ol", null, "passages");
    if (file.count === 1) {
      item.append(passages);
      loadPassages(passages, query, file, 0, 1);
    } else {
      // Several occurrences: an arrow unfolds the list (project instructions).
      const toggle = el("button", null, "toggle");
      toggle.type = "button";
      toggle.setAttribute("aria-expanded", "false");
      toggle.append(el("span", "▸", "arrow"), el("span", T.showPassages(file.count)));
      passages.hidden = true;
      toggle.addEventListener("click", () => {
        const open = toggle.getAttribute("aria-expanded") !== "true";
        toggle.setAttribute("aria-expanded", String(open));
        toggle.firstChild.textContent = open ? "▾" : "▸";
        toggle.lastChild.textContent = open ? T.hidePassages : T.showPassages(file.count);
        passages.hidden = !open;
        if (open && passages.childElementCount === 0) {
          loadPassages(passages, query, file, 0, PASSAGES_FIRST);
        }
      });
      item.append(toggle, passages);
    }
  }
  return item;
}

async function loadPassages(list, query, file, skip, limit) {
  const passages = await call("passages", { query, file: file.id, skip, limit });
  list.querySelector(".more")?.remove();
  for (const p of passages) {
    const item = el("li", null, "passage");
    const where = place(p.place);
    if (where) item.append(el("span", where, "place label"));
    const text = el("span", null, "passage-text");
    // The match is marked for screen readers too, not only by its color.
    text.append(p.before, el("mark", p.matched), p.after);
    item.append(text);
    list.append(item);
  }
  const shown = skip + passages.length;
  if (shown < file.count && passages.length > 0) {
    const more = el("li", null, "more");
    const button = el("button", T.more(Math.min(PASSAGES_MORE, file.count - shown)), "link");
    button.type = "button";
    button.addEventListener("click", () => loadPassages(list, query, file, shown, PASSAGES_MORE));
    more.append(button);
    list.append(more);
  }
}

function openFile(id, reveal) {
  call("open_file", { file: id, reveal }).catch((error) => console.error(error));
}

// ---------------------------------------------------------------- Dialogs and keys

$("problems-button").addEventListener("click", async () => {
  const files = await call("problems");
  const list = $("problems-list");
  list.replaceChildren();
  for (const file of files) {
    const item = el("li");
    item.append(el("span", file.name, "file-name"), el("span", ` — ${reason(file.problem)}`, "file-problem"));
    item.append(el("div", file.folder, "file-folder"));
    list.append(item);
  }
  showDialog($("problems-dialog"));
});

(function buildHelp() {
  const body = $("help-body");
  for (const [title, paragraphs] of T.help) {
    body.append(el("h3", title, "label"));
    const list = el("ul", null, "plain-list");
    for (const p of paragraphs) list.append(el("li", p));
    body.append(list);
  }
})();

/** Opens a dialog at its top, with the focus on its title. */
function showDialog(dialog) {
  dialog.showModal();
  dialog.scrollTop = 0;
  const title = dialog.querySelector("h2");
  title.tabIndex = -1;
  title.focus();
}

$("help-button").addEventListener("click", () => showDialog($("help-dialog")));
for (const button of document.querySelectorAll("dialog .close")) {
  button.addEventListener("click", () => button.closest("dialog").close());
}

document.addEventListener("keydown", (event) => {
  // Ctrl+F (Cmd+F on a Mac) goes back to the search box (D21).
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "f") {
    event.preventDefault();
    if (!$("search-form").hidden) {
      $("query").focus();
      $("query").select();
    }
  }
});

refreshStatus();
