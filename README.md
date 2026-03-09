# 🦀 Omni-MDX
## Un moteur de rendu MDX rapide, cross-platform et cross-langage propulsé par Rust.

Omni-MDX est conçu pour transformer des documents scientifiques complexes (LaTeX, MDX, composants interactifs) en un arbre de données (AST) universel. Initialement créé pour la plateforme TOAQ, ce moteur permet de visualiser du MDX (Markdown Extended) sur des projets du style Web, App, Software.

## 🚀 Vision du Projet
L'objectif d'Omni-MDX est de résoudre le problème de la fragmentation des parseurs. Au lieu d'avoir un parseur en JavaScript pour le site web et un autre en Python pour le traitement de données, Omni-MDX propose un **Cœur Unique en Rust** compilé pour chaque environnement.
* **Vitesse Native :** Parsing haute performance grâce à Rust.
* **Math-Ready :** Support natif de KaTeX ($E=mc^2$) et des blocs d'équations.
* **Composants Agnostiques :** Le moteur identifie les composants JSX/MDX et délègue le rendu à la plateforme hôte.
* **Multi-Cible** : Un seul code source pour le Web (WASM), Python (IA/Datasets) et bientôt Mobile (Dart/FFI).

## 🏗️ Architecture du Monorepo
```
omni-mdx/
├── core-parser/          # 🦀 Le Cœur Rust (Source de vérité)
│   ├── src/              # Logique de parsing MDX -> JSON
│   └── bindings/         # Points d'entrée spécifiques (WASM, Python, etc.)
├── packages/
│   ├── mdx-engine/       # ⚛️ Wrapper React / Next.js (WebAssembly)
│   └── mdx-python/       # 🐍 Binding Python (PyO3) - [Work in progress]
└── tests/
    └── next-sandbox/     # 🧪 Environnement de test Next.js
```

##  🛠️ Installation & Développement
### Pré-requis
* [Rust](https://rustup.rs/) (dernière version stable)
* [Node.js](https://nodejs.org/fr) (v18+)
* [wasm-pack](https://www.google.com/search?q=https://rustwasm.github.io/wasm-pack/installer/) (pour la partie Web)

### 1. Compiler le moteur (WASM)
Depuis la racine du projet :
```bash
cd core-parser
wasm-pack build --target bundler --out-dir ../packages/mdx-engine/wasm-core
```
2. Lancer le bac à sable (Next.js)
```bash
cd tests/next-sandbox
npm install
npm run dev --webpack
```
## 📦 Distribution des Paquets
Chaque module est publié de manière indépendante sur son registre respectif :

|Paquet|Plateforme|Installation|
|:--|:--|:--|
| @omni/mdx-engine | NPM / Next.js |npm install @omni/mdx-engine |
| omni-mdx | PyPI / Python | pip install omni-mdx |
| omni_mdx | Pub / Dart | pub add omni_mdx |

## 📄 Licence
Ce projet est sous licence MIT. Vous êtes libre de l'utiliser, de le modifier et de le distribuer.