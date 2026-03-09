# 🧪 Omni MDX : Next.js Sandbox
Bienvenue dans le bac à sable (Sandbox) officiel du projet Omni MDX.

Il s'agit d'une application Next.js (App Router) allégée, conçue spécifiquement pour tester, déboguer et expérimenter le moteur de rendu `@omni/mdx-engine` et le cœur Rust en temps réel.

C'est ici que l'on valide l'affichage des scripts interactifs pour les 3 séries de podcasts et que l'on vérifie l'extraction visuelle des métadonnées liées au dataset vocal avant de déployer en production.

## 🎯 Objectifs de cet environnement
1. **Validation en Temps Réel :** Taper du Markdown enrichi de composants JSX et voir l'Arbre Syntaxique Abstrait (AST) généré par Rust se transformer instantanément en interface React.

2. **Tests de Résilience :** Éprouver le système de `MDXErrorBoundary`. Insérez des erreurs volontaires dans les props des composants pour vérifier que le reste de la page continue de s'afficher correctement.

3. **Prototypage de Composants :** Développer et tester de nouvelles balises personnalisées (ex: des composants de lecteurs audio spécifiques aux podcasts) isolément de l'application principale.

## 🚀 Démarrage Rapide
Assurez-vous d'abord d'avoir compilé le cœur Rust depuis la racine du monorepo (via `make build-web` ou `make setup`).

Ensuite, naviguez dans ce dossier et lancez le serveur de développement :

```bash
cd tests/next-sandbox
npm run dev
```
Ouvrez http://localhost:3000 dans votre navigateur. Vous devriez y voir l'interface de test interactif.

## 📂 Architecture du Sandbox
Le projet suit la structure standard de Next.js (`App Router`) :

* **`app/page.tsx`** : La page principale. Elle contient le champ de texte et utilise `useMemo` pour appeler le parseur WebAssembly de manière optimisé.

* **`app/components/`** : Contient les composants "Templates" injectés dans le moteur MDX.

* **`Speaker.tsx` (Template)** : Pour simuler et formater les dialogues des séries vocales.

* **`Note.tsx` (Template)**: Pour les encarts d'avertissement et d'information.

* **`DataChart.tsx` (Template)** : Pour tester l'évaluation sécurisée des props JavaScript complexes (tableaux, objets).

* **`app/layout.tsx`** : Injecte le `MdxConfigProvider` pour rendre les composants disponibles globalement dans l'application de test.

## 🛠️ Comment tester un nouveau composant ?
1. Créez votre composant React dans `app/components/MonNouveauComposant.tsx`.

2. Allez dans le fichier où votre configuration est définie (généralement dans `app/page.tsx` ou un fichier de configuration dédié).

3. Ajoutez-le au dictionnaire `components` du provider :

```tsx
import { MonNouveauComposant } from './components/MonNouveauComposant';

const mdxConfig = {
  components: {
    // ... autres composants
    MonNouveauComposant
  }
};
```

4. Mettez dans votre texte envoyé au moteur MDX le nouveau composant `<MonNouveauComposant maProp="test" />` et observez le résultat immédiat !

## 💡 Astuces de Débogage
* **Console du Navigateur :** Si le parseur Rust rencontre une syntaxe fatale (très rare), l'erreur `parseError` s'affichera directement en rouge sur l'interface de la page.

* **Trace d'Erreur React :** Si vous passez une donnée invalide à un composant (ex: `data="TexteAuLieuDUnTableau"` dans le `DataChart`), observez l'encadré rouge généré par le `MDXErrorBoundary` local, tout en remarquant que le reste du contenu Markdown autour reste intact.