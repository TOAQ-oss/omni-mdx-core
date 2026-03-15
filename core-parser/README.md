# 🦀 Omni MDX : Core Parser

Bienvenue dans le moteur de parsing officiel de l'écosystème Omni MDX.

Écrit entièrement en Rust pour des performances maximales et une sécurité mémoire absolue, ce module est chargé d'ingérer les soumissions des chercheurs (fichiers Markdown étendus avec du JSX et des mathématiques) et de les transformer en un Arbre Syntaxique Abstrait (AST) universel au format JSON.

Ce parseur garantit que le texte, les métadonnées et la structure des épisodes de podcasts soient parfaitement qualifiés avant d'être envoyés vers le front-end (React) ou le pipeline de machine learning (Python) du dataset vocal.

## ✨ Fonctionnalités Clés
* **Surgical JSX Cloaking :** Les balises personnalisées (comme `<Speaker>` ou `<DataChart>`) sont temporairement masquées au moteur Markdown standard pour éviter qu'elles ne soient accidentellement enveloppées dans des paragraphes `<p>`.

* **Sanctuarisation des Mathématiques :** Les équations LaTeX (`$` et `$$`) sont extraites avant le parsing. Un signe `>` dans une équation mathématique ne cassera jamais la structure du document.

* **Typage Dynamique des Attributs :** Le parseur identifie si une prop JSX est une chaîne de caractères (`name="Dr. Laurent"`), une expression évaluée (`data={[1, 2, 3]}`) ou un booléen.

* **Zéro Panique :** L'architecture gère les erreurs de syntaxe de manière gracieuse. Si une balise n'a pas été fermé, le moteur renvoie un nœud d'erreur propre au lieu de faire crasher l'application.

* **Multi-Cibles :** Le code est conçu pour être compilé en **WebAssembly (WASM)** pour le web, et exposé via **C-FFI** pour l'écosystème Python.

## 🏗️ Architecture Interne

Le code source (`src/`) est divisé en plusieurs modules spécialisés :

* `lexer.rs` : Analyse syntaxique primaire. Gère le masquage des blocs mathématiques et l'identification des composants JSX.

* `jsx.rs` : Moteur de parsing spécifique aux balises personnalisées et à l'extraction de leurs attributs complexes.

* `markdown.rs` : Gère l'intégration avec `pulldown-cmark` pour la spécification CommonMark standard.

* `ast.rs` : Définition des structures de données (les nœuds et les types d'attributs) qui seront sérialisées en JSON.

* `lib.rs` / `ffi.rs` : Les points d'entrée publics pour l'exportation WASM (`wasm-bindgen`) et Python (`extern "C"`).

## 🚀 Compilation & Tests
Le cycle de vie du parseur est géré par le `Makefile` situé à la racine du monorepo.

### 1. Lancer la suite de tests Rust :
Assurez-vous que le parseur gère correctement les erreurs, l'AST et les performances :

```bash
make test
```
*(Cela exécute les binaires `test_ast`, `test_errors` et `test_perf` situés dans `src/bin/`)*

### 2. Compiler pour le Web (WASM) :
Pour générer le binaire `.wasm` ultra-léger et les définitions TypeScript destinées au module `@toaq-oss/omni-mdx` :

```bash
make build-web
```
*(Les fichiers compilés seront automatiquement copiés dans `packages/mdx-next/omni-core/`)*

### 3. Nettoyer les artefacts de build :

```bash
make clean
```

## 🌳 Structure de l'AST Généré
Le parseur transforme n'importe quel script MDX en un flux JSON strict. Voici à quoi ressemble l'output standardisé :

**Entrée (MDX) :**

```mdx
<Speaker name="Dr. Dupont" time="00:15">
  L'énergie est $E = mc^2$.
</Speaker>
```
Sortie (JSON AST) :

```json
[
  {
    "node_type": "Speaker",
    "attributes": {
      "name": { "kind": "text", "value": "Dr. Dupont" },
      "time": { "kind": "text", "value": "00:15" }
    },
    "children": [
      {
        "node_type": "text",
        "content": "L'énergie est "
      },
      {
        "node_type": "InlineMath",
        "content": "E = mc^2"
      },
      {
        "node_type": "text",
        "content": "."
      }
    ]
  }
]
```

## 🛡️ Gestion des Erreurs
La philosophie du `core-parser` est de toujours fournir un retour exploitable. Si le document contient du JSX mal formé, le parser injectera un nœud d'erreur spécifique directement dans l'AST :

```
{
  "node_type": "error",
  "content": "Unexpected token near '<Speaker name=\"...'"
}
```
Cela permet aux interfaces clientes d'afficher des "Error Boundaries" localisées sans détruire le reste de la page ou du script.