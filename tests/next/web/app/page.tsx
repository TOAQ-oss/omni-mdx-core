import { MDXViewer } from '@omni/mdx-engine';

export default function Home() {
  const testArticle = `
<Note type="info" title="Open Source">
  Le modèle est, comme le souhaite les valeurs internes, disponible sur le [Hugging Face](https://huggingface.co/TOAQ/Leon-7B-Research-Radio-v1)
</Note>

## 1. Le Défi de la Génération Longue

L'objectif de **TOAQ Research Radio** est de démocratiser l'accès à la recherche scientifique en transformant des publications académiques complexes en podcasts. 

La conversion d'une publication scientifique complète nécessite un découpage séquentiel (Chunking). Lors de la phase de prototypage (v0), le modèle traitait chaque section comme un nouvel épisode de podcast.

<SplitLayout 
  ratio="60-40"
  stickyRight={true}
  left={
    <div className="prose">
      <h2>L'innovation : Le Temporal Positioning</h2>
      Pour contrer les hallucinations de format, le dataset de la V1 (450 paires issues de 41 documents) a été enrichi avec des balises de contexte utilisateur.
      
      Ce conditionnement explicite permet d'éliminer totalement le besoin de scripts Python de post-traitement pour nettoyer les redondances de présentation.
    </div>
  }
  right={
    <Note type="warning" title="Le syndrome V0">
      Sans balisage, un LLM générant un podcast en 5 morceaux générait 5 fois la phrase : "Bonjour et bienvenue dans ce nouvel épisode".
    </Note>
  }
/>

<Timeline items={[
  { time: "Section 1", title: "[Position: START] - Force une accroche et la présentation de l'hôte.", active: true },
  { time: "Section 2 à N", title: "[Position: MIDDLE] - Interdit les salutations, se concentre sur le flux technique.", active: false },
  { time: "Section Finale", title: "[Position: END] - Déclenche la conclusion et la signature sonore.", active: false }
]} />

## 2. Infrastructure et Paramètres d'Entraînement

L'entraînement a été réalisé avec la technique **QLoRA** via la librairie **Unsloth**.

<Table 
  caption="Configuration matérielle et hyperparamètres (SFTTrainer)"
  headers={["Paramètre", "Configuration", "Impact"]}
  data={[
    ["Base Model", "Mistral 7B v0.3", "Vocabulaire étendu"],
    ["Hardware", "NVIDIA A100 (40GB)", "Permet un batch size plus grand"],
    ["Global Batch Size", "16", "Stabilité des gradients"],
    ["Learning Rate", "1e-4", "Apprentissage doux sur 150 steps"],
    ["Précision", "BF16", "Stabilité numérique native A100"]
  ]}
/>

<Details title="Voir les métriques d'utilisation des ressources">
  Grâce à la quantification, nous avons pu constater ces métriques :
  * **GPU VRAM :** 7.6 GB utilisés sur 40.0 GB disponibles.
  * **System RAM :** 6.1 GB utilisés sur 83.5 GB disponibles.
  * **Débit :** 5.26 échantillons par seconde.
</Details>

## 3. Dynamique d'Apprentissage et Convergence

L'augmentation du volume de données couplée à un apprentissage sur 150 étapes démontre une convergence sans aucun overfitting.

<Chart
  caption="Évolution de la Training Loss sur la A100"
  xAxis={{ label: "Steps d'entraînement", domain: [0, 150] }}
  yAxis={{ label: "Loss", domain: [0.8, 2.5] }}
  curves={[
    { 
      id: "loss_curve", 
      label: "Léon-v1 Training Loss", 
      color: "#10b981", 
      fn: "(x) => 1.4 * Math.exp(-0.02 * x) + 1.0" 
    }
  ]}
/>

*Note : La fonction représentée ci-dessus simule la courbe d'apprentissage réelle, partant d'une perte de 2.43 pour se stabiliser à un minimum global de 0.93 (Step 149).*

Le point d'équilibre autour de 1.00 indique que le réseau a parfaitement assimilé la structure narrative (SSML, transitions fluides) tout en conservant sa flexibilité sémantique pour vulgariser les équations et concepts LaTeX <AnnotationRef index={1} />.

## 4. Perspectives

La validation de l'architecture Léon-v1 marque un tournant pour l'automatisation de la plateforme. La prochaine étape visera l'intégration multimodale (Vision-to-Text) pour ingérer les graphiques des papiers de recherche, ainsi que le déploiement d'une pipeline ArXiv 100% autonome.

<Glossary items={[
  { term: "QLoRA", def: "Quantized Low-Rank Adaptation. Méthode permettant d'entraîner des modèles massifs sur un seul GPU en réduisant la précision des poids." },
  { term: "SSML", def: "Speech Synthesis Markup Language. Balises (ex: <break time='0.5s'/>) permettant de dicter le rythme de la synthèse vocale." }
]} />

<Annotation index={1}>
  Pour des raisons de fluidité audio, le modèle est entraîné à convertir la notation mathématique en langage naturel descriptif plutôt qu'à la lire de manière brute.
</Annotation>
  `;

  return (
    <main className="min-h-screen p-12 bg-gray-50 text-gray-900 font-sans">
      <div className="max-w-3xl mx-auto bg-white p-8 rounded-xl shadow-lg">
        <p className="text-sm text-blue-500 font-bold mb-4 uppercase tracking-widest">Aperçu Visuel</p>
        <MDXViewer content={testArticle} />
      </div>
    </main>
  );
}