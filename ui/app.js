// Interface texts. The language follows the operating system; English is the fallback.
// A translation is marked ready only after review by a native speaker (project rule).
"use strict";

const TEXTS = {
  en: {
    rehearsal: "Rehearsal version",
    welcome: "Welcome to Mulu.",
    purpose: "This rehearsal version only checks that Mulu downloads, installs and opens correctly on your computer. Search arrives in version 0.1.",
    firstStep: "First step",
    chooseFolder: "Choose the folder with your books",
    soon: "Available in version 0.1.",
    promisesTitle: "Promises",
    offline: "Works entirely offline. No account, no ads, no data collection.",
    readOnly: "Only reads your files. Nothing is changed without your confirmation.",
    free: "Free software, under the GNU GPL, version 3 or later.",
    version: "Version",
  },
  "pt-BR": {
    rehearsal: "Versão de ensaio",
    welcome: "Bem-vindo ao Mulu.",
    purpose: "Esta versão de ensaio serve só para conferir se o Mulu é baixado, instalado e aberto corretamente no seu computador. A busca chega na versão 0.1.",
    firstStep: "Primeiro passo",
    chooseFolder: "Escolha a pasta dos seus livros",
    soon: "Disponível na versão 0.1.",
    promisesTitle: "Compromissos",
    offline: "Funciona inteiramente sem internet. Sem conta, sem anúncios, sem coleta de dados.",
    readOnly: "Só lê os seus arquivos. Nada é alterado sem a sua confirmação.",
    free: "Software livre, sob a GPL da GNU, versão 3 ou posterior.",
    version: "Versão",
  },
  es: {
    rehearsal: "Versión de ensayo",
    welcome: "Bienvenido a Mulu.",
    purpose: "Esta versión de ensayo solo sirve para comprobar que Mulu se descarga, se instala y se abre correctamente en su computadora. La búsqueda llega en la versión 0.1.",
    firstStep: "Primer paso",
    chooseFolder: "Elija la carpeta de sus libros",
    soon: "Disponible en la versión 0.1.",
    promisesTitle: "Compromisos",
    offline: "Funciona completamente sin internet. Sin cuenta, sin anuncios, sin recopilación de datos.",
    readOnly: "Solo lee sus archivos. Nada se modifica sin su confirmación.",
    free: "Software libre, bajo la GPL de GNU, versión 3 o posterior.",
    version: "Versión",
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
const texts = TEXTS[language];
document.documentElement.lang = language;
for (const element of document.querySelectorAll("[data-i18n]")) {
  const text = texts[element.dataset.i18n];
  if (text) element.textContent = text;
}
