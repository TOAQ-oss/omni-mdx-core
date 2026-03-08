export const analyzeMDXError = (rawMessage: string, content: string, line: number | undefined): string => {
    if (!line || line < 1) return "Erreur de syntaxe générale.";

    const lines = content.split('\n');
    const errorLineIndex = Math.min(line - 1, lines.length - 1);
    
    if (errorLineIndex < 0) return "Erreur de syntaxe (fichier vide ?)";

    const errorLine = lines[errorLineIndex] || "";
    let componentName = "le code";
    const lookBackIndex = Math.max(0, errorLineIndex - 10);

    for (let i = errorLineIndex; i >= lookBackIndex; i--) {
        const currentLine = lines[i]?.trim(); 
        
        if (!currentLine) continue;

        if (currentLine.startsWith('<Table')) { componentName = "le Tableau"; break; }
        if (currentLine.startsWith('<Chart')) { componentName = "le Graphique"; break; }
        if (currentLine.startsWith('<Note')) { componentName = "la Note"; break; }
        if (currentLine.startsWith('<SplitLayout')) { componentName = "la Mise en page"; break; }
    }

    const quoteCount = (errorLine.match(/"/g) || []).length;
    const simpleQuoteCount = (errorLine.match(/'/g) || []).length;

    if (quoteCount % 2 !== 0) {
        return `Erreur sur ${componentName} : Il manque probablement un guillemet (") sur cette ligne.`;
    }
    
    if (simpleQuoteCount % 2 !== 0 && (errorLine.includes('=') || errorLine.includes('['))) {
        return `Erreur sur ${componentName} : Il manque probablement une apostrophe (') pour fermer une valeur.`;
    }

    if (errorLine.includes('[') && !errorLine.includes(']')) {
        return `Erreur sur ${componentName} : Vous avez ouvert un crochet '[' sans le fermer sur la même ligne.`;
    }
    
    if (errorLine.includes('{') && !errorLine.includes('}')) {
        return `Erreur sur ${componentName} : Vous avez ouvert une accolade '{' sans la fermer.`;
    }

    if (rawMessage.toLowerCase().includes('unexpected') || rawMessage.toLowerCase().includes('parse')) {
        const trimmed = errorLine.trim();
        if ((trimmed.endsWith('"') || trimmed.endsWith(']') || /[\d]$/.test(trimmed)) && !trimmed.endsWith(',')) {
             if (componentName === "le Tableau" || componentName === "le Graphique") {
                return `Erreur sur ${componentName} : Il manque peut-être une virgule (,) à la fin de la ligne.`;
             }
        }
        return `Erreur de syntaxe dans ${componentName}. Vérifiez les virgules et les fermetures (] ou }).`;
    }

    return `Erreur détectée dans ${componentName} (Ligne ${line}).`;
};