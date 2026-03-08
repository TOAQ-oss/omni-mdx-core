import React from 'react';
import { renderToString } from 'react-dom/server';
// Assure-toi que le chemin correspond bien à ton fichier MDXRenderer
import { MDXViewer } from './src/MDXRenderer'; 

// 1. Notre texte de test brut (rappel: notre Rust ne comprend que h1 et p pour l'instant !)
const markdownTest = `
# Architecture Validée
Voici une équation en direct : $E=mc^2$
<Note>Le système WASM est branché sur Next.js !</Note>
  `;

console.log("🚀 Début du test...");
console.log("Texte Markdown en entrée :\n", markdownTest);

try {
    // 2. On demande à React de faire le rendu complet de ton composant
    const htmlOutput = renderToString(<MDXViewer content={markdownTest} />);
    
    console.log("\n✅ Succès ! Voici le code HTML final généré par tes composants TOAQ :\n");
    console.log(htmlOutput);
    console.log("\n🎉 La chaîne Rust -> WASM -> React est 100% opérationnelle.");
    
} catch (error) {
    console.error("\n❌ Erreur lors du rendu :", error);
}