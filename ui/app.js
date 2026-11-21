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
    basesTitle: "Libraries",
    addLibrary: "+ Add a library",
    removeLibrary: "Take out of Mulu",
    removeTitle: "Take this library out of Mulu?",
    removeText: (name) => `“${name}” will no longer be searched. Its folder and its files are not touched: only Mulu's index of it is deleted. You can add it again later; it will be read again.`,
    removeConfirm: "Take it out",
    cancel: "Cancel",
    missing: "Folder not found (an external disk not plugged in?). Its books stay in the index.",
    readingBase: (done, total) => total === 0 ? "Checking for changes…" : `Reading: ${n(done)} of ${n(total)}`,
    overlap: (o) => o.kind === "same" ? "This folder is already a library."
      : o.kind === "inside" ? `This folder is already part of the library “${o.library}”.`
      : `This folder holds the library “${o.library}”. Take that library out first, or choose another folder.`,
    refresh: "Check for changes",
    checking: "Checking for changes…",
    checked: (when) => `Checked on ${when}`,
    neverChecked: "Not checked yet",
    noChanges: "No changes since the last check.",
    changes: (c) => {
      const parts = [];
      if (c.added) parts.push(`${n(c.added)} new`);
      if (c.changed) parts.push(`${n(c.changed)} changed`);
      if (c.moved) parts.push(`${n(c.moved)} renamed or moved`);
      if (c.removed) parts.push(`${n(c.removed)} removed from the index`);
      return parts.join(", ") + ".";
    },
    scopeNote: "Untick a folder to leave it out of the search.",
    showFolders: (name) => `Show the folders in ${name}`,
    hideFolders: (name) => `Hide the folders in ${name}`,
    filesIn: (count) => count === 1 ? "1 file" : `${n(count)} files`,
    scoped: "in part of the library",
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
    copyReport: "Copy a reading report for the developer",
    faster: "Read faster",
    fasterNote: "Uses all of the computer's power; other programs may be slow until the reading ends.",
    slower: "Back to background reading",
    slowerNote: "Reading at full speed. The computer may be slow until it ends.",
    copied: "Copied. Paste it in your message.",
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
      ["Libraries and scope", [
        "The library appears on the left with its folders. Untick a folder to leave it out of the search; tick it again to bring it back. The index stays whole, so this is instant.",
        "Mulu checks the library for changes each time it opens. The ↻ button checks again now. Only new and changed files are read; renamed or moved files keep their place in the index.",
        "Files that left the folder only leave Mulu's index: Mulu never deletes anything.",
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
    basesTitle: "Bases",
    addLibrary: "+ Acrescentar base",
    removeLibrary: "Tirar do Mulu",
    removeTitle: "Tirar esta base do Mulu?",
    removeText: (name) => `“${name}” deixará de ser pesquisada. A pasta e os arquivos dela não são tocados: só o índice do Mulu é apagado. Você pode acrescentá-la de novo depois; ela será lida outra vez.`,
    removeConfirm: "Tirar",
    cancel: "Cancelar",
    missing: "Pasta não encontrada (disco externo desligado?). Os livros dela continuam no índice.",
    readingBase: (done, total) => total === 0 ? "Conferindo as mudanças…" : `Lendo: ${n(done)} de ${n(total)}`,
    overlap: (o) => o.kind === "same" ? "Esta pasta já é uma base."
      : o.kind === "inside" ? `Esta pasta já faz parte da base “${o.library}”.`
      : `Esta pasta contém a base “${o.library}”. Tire essa base antes, ou escolha outra pasta.`,
    refresh: "Conferir as mudanças",
    checking: "Conferindo as mudanças…",
    checked: (when) => `Conferida em ${when}`,
    neverChecked: "Ainda não conferida",
    noChanges: "Nenhuma mudança desde a última conferência.",
    changes: (c) => {
      const parts = [];
      if (c.added) parts.push(`${n(c.added)} ${c.added === 1 ? "novo" : "novos"}`);
      if (c.changed) parts.push(`${n(c.changed)} ${c.changed === 1 ? "alterado" : "alterados"}`);
      if (c.moved) parts.push(`${n(c.moved)} ${c.moved === 1 ? "renomeado ou movido" : "renomeados ou movidos"}`);
      if (c.removed) parts.push(`${n(c.removed)} ${c.removed === 1 ? "removido" : "removidos"} do índice`);
      return parts.join(", ") + ".";
    },
    scopeNote: "Desmarque uma pasta para deixá-la fora da pesquisa.",
    showFolders: (name) => `Mostrar as pastas de ${name}`,
    hideFolders: (name) => `Esconder as pastas de ${name}`,
    filesIn: (count) => count === 1 ? "1 arquivo" : `${n(count)} arquivos`,
    scoped: "em parte da base",
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
    copyReport: "Copiar um relatório da leitura para o desenvolvedor",
    faster: "Ler mais depressa",
    fasterNote: "Usa toda a força do computador; outros programas podem ficar lentos até a leitura terminar.",
    slower: "Voltar à leitura em segundo plano",
    slowerNote: "Lendo em velocidade máxima. O computador pode ficar lento até terminar.",
    copied: "Copiado. Cole na sua mensagem.",
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
      ["Bases e escopo", [
        "A base aparece à esquerda, com as suas pastas. Desmarque uma pasta para deixá-la fora da pesquisa; marque de novo para trazê-la de volta. O índice continua inteiro, por isso a mudança é instantânea.",
        "O Mulu confere as mudanças da base cada vez que é aberto. O botão ↻ confere de novo na hora. Só os arquivos novos e alterados são lidos; os renomeados ou movidos mantêm o lugar no índice.",
        "Arquivos que saíram da pasta saem só do índice do Mulu: o Mulu nunca apaga nada.",
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
    basesTitle: "Bibliotecas",
    addLibrary: "+ Añadir biblioteca",
    removeLibrary: "Quitar de Mulu",
    removeTitle: "¿Quitar esta biblioteca de Mulu?",
    removeText: (name) => `“${name}” dejará de buscarse. Su carpeta y sus archivos no se tocan: solo se borra el índice de Mulu. Puede añadirla de nuevo más tarde; se leerá otra vez.`,
    removeConfirm: "Quitar",
    cancel: "Cancelar",
    missing: "Carpeta no encontrada (¿disco externo desconectado?). Sus libros siguen en el índice.",
    readingBase: (done, total) => total === 0 ? "Comprobando los cambios…" : `Leyendo: ${n(done)} de ${n(total)}`,
    overlap: (o) => o.kind === "same" ? "Esta carpeta ya es una biblioteca."
      : o.kind === "inside" ? `Esta carpeta ya forma parte de la biblioteca “${o.library}”.`
      : `Esta carpeta contiene la biblioteca “${o.library}”. Quite antes esa biblioteca, o elija otra carpeta.`,
    refresh: "Comprobar los cambios",
    checking: "Comprobando los cambios…",
    checked: (when) => `Comprobada el ${when}`,
    neverChecked: "Todavía no comprobada",
    noChanges: "Ningún cambio desde la última comprobación.",
    changes: (c) => {
      const parts = [];
      if (c.added) parts.push(`${n(c.added)} ${c.added === 1 ? "nuevo" : "nuevos"}`);
      if (c.changed) parts.push(`${n(c.changed)} ${c.changed === 1 ? "modificado" : "modificados"}`);
      if (c.moved) parts.push(`${n(c.moved)} ${c.moved === 1 ? "renombrado o movido" : "renombrados o movidos"}`);
      if (c.removed) parts.push(`${n(c.removed)} ${c.removed === 1 ? "quitado" : "quitados"} del índice`);
      return parts.join(", ") + ".";
    },
    scopeNote: "Desmarque una carpeta para dejarla fuera de la búsqueda.",
    showFolders: (name) => `Mostrar las carpetas de ${name}`,
    hideFolders: (name) => `Ocultar las carpetas de ${name}`,
    filesIn: (count) => count === 1 ? "1 archivo" : `${n(count)} archivos`,
    scoped: "en parte de la biblioteca",
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
    copyReport: "Copiar un informe de la lectura para el desarrollador",
    faster: "Leer más rápido",
    fasterNote: "Usa toda la potencia de la computadora; otros programas pueden ir lentos hasta que termine la lectura.",
    slower: "Volver a la lectura en segundo plano",
    slowerNote: "Leyendo a máxima velocidad. La computadora puede ir lenta hasta que termine.",
    copied: "Copiado. Péguelo en su mensaje.",
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
      ["Bibliotecas y alcance", [
        "La biblioteca aparece a la izquierda, con sus carpetas. Desmarque una carpeta para dejarla fuera de la búsqueda; márquela de nuevo para recuperarla. El índice sigue entero, por eso el cambio es instantáneo.",
        "Mulu comprueba los cambios de la biblioteca cada vez que se abre. El botón ↻ los comprueba de nuevo en el momento. Solo se leen los archivos nuevos y modificados; los renombrados o movidos conservan su lugar en el índice.",
        "Los archivos que salieron de la carpeta solo salen del índice de Mulu: Mulu nunca borra nada.",
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
  const hasLibrary = status.bases.length > 0;
  $("welcome").hidden = hasLibrary;
  $("library").hidden = !hasLibrary;
  $("bases").hidden = !hasLibrary;
  $("search-form").hidden = !hasLibrary;
  if (!hasLibrary) return;
  showBases(status);

  const reading = status.reading;
  const files = status.bases.reduce((sum, b) => sum + b.files, 0);
  status.problems = status.bases.reduce((sum, b) => sum + b.problems, 0);
  $("indexing").hidden = !reading;
  $("ready-text").hidden = Boolean(reading);
  if (reading) {
    if (reading.total === 0) {
      $("indexing-text").textContent = T.finding;
      $("indexing-bar").removeAttribute("value");
    } else {
      $("indexing-text").textContent = T.indexing(reading.done, reading.total);
      $("indexing-bar").max = reading.total;
      $("indexing-bar").value = reading.done;
    }
    $("pace-button").textContent = status.fast ? T.slower : T.faster;
    $("pace-note").textContent = status.fast ? T.slowerNote : T.fasterNote;
    $("pace-button").dataset.fast = String(status.fast);
    if (!polling) polling = setInterval(refreshStatus, 1000);
  } else {
    clearInterval(polling);
    polling = null;
    $("ready-text").textContent = T.ready(files);
  }
  $("problems-button").hidden = status.problems === 0;
  $("problems-button").textContent = T.problems(status.problems);
}

$("choose").addEventListener("click", async () => {
  $("choose").disabled = true;
  try {
    const chosen = await call("add_library");
    if (chosen.Err) {
      $("welcome-error").textContent = T.overlap(chosen.Err);
    } else if (chosen.Ok) {
      await refreshStatus();
      $("query").focus();
    }
  } finally {
    $("choose").disabled = false;
  }
});

// Background reading (the default) or full speed (D29).
$("pace-button").addEventListener("click", async () => {
  const fast = $("pace-button").dataset.fast !== "true";
  await call("set_fast", { fast });
  await refreshStatus();
});

// ---------------------------------------------------------------- Bases and scope

const dateFormat = new Intl.DateTimeFormat(language, { dateStyle: "short", timeStyle: "short" });

/** The scope of each library, by its folder: the folders left out of the search ("" is
 *  the top folder). Each folder counts for the files directly in it; ticking or
 *  unticking a folder applies to all the folders below it too (D32). */
const scopes = new Map();

function scopeOf(root) {
  if (!scopes.has(root)) {
    scopes.set(root, { unticked: new Set(), folders: [], previous: new Set(), expanded: new Set(), key: "", block: null });
  }
  return scopes.get(root);
}

function baseName(root) {
  const parts = root.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] || root;
}

function showBases(status) {
  const list = $("base-list");
  const roots = status.bases.map((b) => b.root);
  for (const root of [...scopes.keys()]) {
    if (!roots.includes(root)) {
      scopes.get(root).block?.remove();
      scopes.delete(root);
    }
  }
  for (const base of status.bases) {
    const scope = scopeOf(base.root);
    if (!scope.block) scope.block = baseBlock(base.root);
    list.append(scope.block); // keeps the order of the list
    const reading = status.reading && status.reading.base === base.root ? status.reading : null;
    const q = (name) => scope.block.querySelector(name);
    q(".refresh").disabled = Boolean(reading);
    q(".refresh").setAttribute("aria-busy", String(Boolean(reading)));
    if (!base.present) {
      q(".checked").textContent = T.missing;
      q(".checked").classList.add("base-missing");
    } else {
      q(".checked").classList.remove("base-missing");
      q(".checked").textContent = reading ? T.readingBase(reading.done, reading.total)
        : base.checked ? T.checked(dateFormat.format(new Date(base.checked * 1000)))
        : T.neverChecked;
    }
    const c = base.changes;
    const anything = c && (c.added || c.changed || c.moved || c.removed);
    q(".changes").textContent = !c || reading ? "" : anything ? T.changes(c) : T.noChanges;
    // The folders change when files are found, moved or removed.
    const key = `${base.files}|${base.checked}|${Boolean(reading)}`;
    if (key !== scope.key) {
      scope.key = key;
      loadFolders(base.root);
    }
  }
}

let boxNumber = 0;

function baseBlock(root) {
  const block = el("div", null, "base");
  const head = el("div", null, "base-head");
  const box = document.createElement("input");
  box.type = "checkbox";
  box.id = `base-${boxNumber++}`;
  box.className = "base-check";
  box.checked = true;
  box.addEventListener("change", () => toggleScope(root, ""));
  const label = el("label", baseName(root), "base-name");
  label.htmlFor = box.id;
  const refresh = el("button", "↻", "icon-button refresh");
  refresh.type = "button";
  refresh.title = T.refresh;
  refresh.setAttribute("aria-label", `${T.refresh}: ${baseName(root)}`);
  refresh.addEventListener("click", async () => {
    refresh.disabled = true;
    await call("refresh", { base: root });
    await refreshStatus();
  });
  head.append(box, label, refresh);
  const remove = el("button", T.removeLibrary, "link small remove-library");
  remove.type = "button";
  remove.addEventListener("click", () => askRemove(root));
  block.append(
    head,
    el("p", root, "path base-path"),
    el("p", null, "note small checked"),
    el("p", null, "note small changes"),
    el("ul", null, "tree"),
    remove,
  );
  block.querySelector(".checked").setAttribute("aria-live", "polite");
  return block;
}

let removing = null;

function askRemove(root) {
  removing = root;
  $("remove-text").textContent = T.removeText(baseName(root));
  showDialog($("remove-dialog"));
}

$("remove-confirm").addEventListener("click", async () => {
  $("remove-dialog").close();
  if (!removing) return;
  await call("remove_library", { base: removing });
  removing = null;
  await refreshStatus();
  if (currentQuery) search(currentQuery);
});

async function loadFolders(root) {
  const scope = scopeOf(root);
  try {
    scope.folders = await call("folders", { base: root });
  } catch (error) {
    console.error(error);
    return;
  }
  const known = new Set(scope.folders.map((f) => f.path));
  for (const path of [...scope.unticked]) if (path !== "" && !known.has(path)) scope.unticked.delete(path);
  // A folder that appeared in a check follows its parent: inside an unticked folder,
  // it starts unticked. Parents come before their subfolders in the list.
  if (scope.previous.size > 0) {
    for (const folder of scope.folders) {
      if (!scope.previous.has(folder.path)) {
        const cut = folder.path.lastIndexOf("/");
        if (scope.unticked.has(cut < 0 ? "" : folder.path.slice(0, cut))) scope.unticked.add(folder.path);
      }
    }
  }
  scope.previous = known;
  const tree = scope.block.querySelector(".tree");
  tree.replaceChildren(...childrenOf(scope, "").map((f) => folderItem(root, f)));
  updateChecks(root);
}

function childrenOf(scope, parent) {
  return scope.folders.filter((f) => {
    const cut = f.path.lastIndexOf("/");
    return (cut < 0 ? "" : f.path.slice(0, cut)) === parent;
  }).sort((a, b) => a.path.localeCompare(b.path, language, { sensitivity: "base", numeric: true }));
}

function isUnder(path, folder) {
  return folder === "" || path === folder || path.startsWith(folder + "/");
}

/** A click on a box: a ticked box unticks its folder and all below it; an unticked or
 *  half-ticked box ticks them all (the usual convention). */
function toggleScope(root, folder) {
  const scope = scopeOf(root);
  const all = ["", ...scope.folders.map((f) => f.path)];
  const out = all.some((p) => isUnder(p, folder) && scope.unticked.has(p));
  for (const path of all) {
    if (isUnder(path, folder)) {
      if (out) scope.unticked.delete(path); else scope.unticked.add(path);
    }
  }
  updateChecks(root);
  if (currentQuery) search(currentQuery);
}

/** Shows each box ticked, unticked, or half (some folders below unticked). */
function updateChecks(root) {
  const scope = scopeOf(root);
  const all = ["", ...scope.folders.map((f) => f.path)];
  const show = (box, folder) => {
    const below = all.filter((p) => isUnder(p, folder));
    const out = below.filter((p) => scope.unticked.has(p)).length;
    box.checked = out < below.length;
    box.indeterminate = out > 0 && out < below.length;
  };
  show(scope.block.querySelector(".base-check"), "");
  for (const box of scope.block.querySelectorAll(".tree input[type=checkbox]")) show(box, box.dataset.path);
}

function folderItem(root, folder) {
  const scope = scopeOf(root);
  const name = folder.path.slice(folder.path.lastIndexOf("/") + 1);
  const item = el("li");
  const row = el("div", null, "tree-row");
  const box = document.createElement("input");
  box.type = "checkbox";
  box.id = `folder-${boxNumber++}`;
  box.dataset.path = folder.path;
  box.addEventListener("change", () => toggleScope(root, folder.path));
  const label = el("label", name);
  label.htmlFor = box.id;
  label.title = folder.path;
  const count = el("span", n(folder.files), "tree-count");
  count.setAttribute("aria-label", T.filesIn(folder.files));
  const children = childrenOf(scope, folder.path);
  if (children.length > 0) {
    const toggle = el("button", "▸", "tree-toggle");
    toggle.type = "button";
    toggle.setAttribute("aria-expanded", "false");
    toggle.setAttribute("aria-label", T.showFolders(name));
    let list = null;
    toggle.addEventListener("click", () => {
      const open = toggle.getAttribute("aria-expanded") !== "true";
      toggle.setAttribute("aria-expanded", String(open));
      toggle.textContent = open ? "▾" : "▸";
      toggle.setAttribute("aria-label", open ? T.hideFolders(name) : T.showFolders(name));
      if (open) scope.expanded.add(folder.path); else scope.expanded.delete(folder.path);
      if (open && !list) {
        // Subfolders are built when first opened: a large library has thousands.
        list = el("ul");
        list.append(...children.map((f) => folderItem(root, f)));
        item.append(list);
        updateChecks(root);
      }
      if (list) list.hidden = !open;
    });
    row.append(toggle);
    if (scope.expanded.has(folder.path)) queueMicrotask(() => toggle.click());
  } else {
    row.append(el("span", null, "tree-spacer"));
  }
  row.append(box, label, count);
  item.append(row);
  return item;
}

/** Where a file is: its folder, preceded by the name of its library when there are
 *  several. */
function whereIs(file) {
  if (scopes.size < 2) return file.folder;
  return file.folder ? `${baseName(file.base)} › ${file.folder}` : baseName(file.base);
}

/** The folders left out of the search, library by library, as the search command
 *  wants them. */
function excludedScopes() {
  return [...scopes.entries()]
    .filter(([, scope]) => scope.unticked.size > 0)
    .map(([base, scope]) => ({ base, folders: [...scope.unticked] }));
}

$("add-library").addEventListener("click", async () => {
  $("add-error").textContent = "";
  $("add-library").disabled = true;
  try {
    const chosen = await call("add_library");
    if (chosen.Err) $("add-error").textContent = T.overlap(chosen.Err);
    else if (chosen.Ok) await refreshStatus();
  } finally {
    $("add-library").disabled = false;
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
  const answer = await call("search", { query, excluded: excludedScopes() });
  if (!answer || query !== currentQuery) return;
  $("start-hint").hidden = true;
  results.replaceChildren();

  if (answer.files.length === 0 && answer.by_name.length === 0) {
    results.append(el("p", T.noResults(`“${query}”`), "summary"));
    return;
  }
  if (answer.files.length > 0) {
    const seconds = (answer.elapsed_ms / 1000).toLocaleString(language, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
    const summary = el("p", T.summary(answer.total, answer.files.length, seconds), "summary");
    if (excludedScopes().length > 0) summary.append(" · ", el("span", T.scoped, "scoped"));
    results.append(summary);
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
  row.addEventListener("dblclick", () => openFile(file, false));
  row.addEventListener("keydown", (event) => {
    if (event.key === "Enter") openFile(file, false);
  });

  const names = el("div", null, "file-names");
  names.append(el("span", file.name, "file-name"));
  const details = el("span", whereIs(file), "file-folder");
  if (file.problem) details.append(el("span", ` · ${reason(file.problem)}`, "file-problem"));
  names.append(details);
  row.append(names);
  if (file.count > 0) row.append(el("span", n(file.count), "count"));

  const reveal = el("button", T.reveal, "link small");
  reveal.type = "button";
  reveal.addEventListener("click", (event) => {
    event.stopPropagation();
    openFile(file, true);
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
  const passages = await call("passages", { base: file.base, query, file: file.id, skip, limit });
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

function openFile(file, reveal) {
  call("open_file", { base: file.base, file: file.id, reveal }).catch((error) => console.error(error));
}

// ---------------------------------------------------------------- Dialogs and keys

$("problems-button").addEventListener("click", async () => {
  const files = await call("problems");
  // Grouped by reason, the most frequent first.
  const groups = new Map();
  for (const file of files) {
    const why = reason(file.problem);
    if (!groups.has(why)) groups.set(why, []);
    groups.get(why).push(file);
  }
  const ordered = [...groups.entries()].sort((a, b) => b[1].length - a[1].length);
  const summary = $("problems-summary");
  summary.replaceChildren();
  const list = $("problems-list");
  list.replaceChildren();
  for (const [why, group] of ordered) {
    summary.append(el("li", `${why}: ${n(group.length)}`));
    const heading = el("h3", `${why} (${n(group.length)})`, "label");
    const items = el("ul", null, "plain-list");
    for (const file of group) {
      const item = el("li");
      item.append(el("span", file.name, "file-name"));
      item.append(el("div", whereIs(file), "file-folder"));
      items.append(item);
    }
    list.append(heading, items);
  }
  $("copied").textContent = "";
  showDialog($("problems-dialog"));
});

// A plain-text report that the user can paste in a message, to find what makes the
// reading slow or fail. Only file names, sizes and times: never their content.
$("copy-report").addEventListener("click", async () => {
  const [report, files] = await Promise.all([call("report"), call("problems")]);
  if (!report) return;
  const lines = [];
  lines.push(`Mulu ${$("version").textContent} — ${navigator.userAgent}`);
  if (report.last_run !== null && report.last_run !== undefined) lines.push(`Last reading: ${report.last_run} s`);
  lines.push(`Processors: ${report.processors} (cores: ${report.cores ?? "?"}); files read at once: ${report.background} in the background, ${report.full_speed} at full speed (${report.full_speed_used ? "used" : "not used"})`);
  const kindOf = (problem) => problem ? (typeof problem === "string" ? problem : Object.keys(problem)[0]) : "";
  for (const base of report.bases) {
    const failed = files.filter((f) => f.base === base.base);
    lines.push("");
    lines.push(`Library ${base.base}`);
    lines.push(`Files: ${base.files}; read with text: ${base.indexed}; could not be read: ${failed.length}; ${Math.round(base.megabytes)} MB`);
    lines.push(`Time in the reading processes: ${Math.round(base.read_ms / 1000)} s; adding to the index: ${Math.round(base.index_ms / 1000)} s`);
    const counts = new Map();
    for (const file of failed) counts.set(kindOf(file.problem) || "?", (counts.get(kindOf(file.problem) || "?") || 0) + 1);
    lines.push("Reasons: " + [...counts.entries()].map(([k, c]) => `${k} ${c}`).join(", "));
    lines.push("Slowest files (read ms, index ms, MB, sections, path):");
    for (const f of base.slowest) {
      const kind = kindOf(f.problem);
      lines.push(`${f.read_ms}\t${f.index_ms}\t${f.megabytes.toFixed(1)}\t${f.sections}\t${f.path}${kind ? "\t" + kind : ""}`);
    }
  }
  try {
    await navigator.clipboard.writeText(lines.join("\n"));
    $("copied").textContent = T.copied;
  } catch (error) {
    console.error(error);
  }
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
