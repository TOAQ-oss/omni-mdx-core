/**
 * Parses a stringified JSX property value into its corresponding native JavaScript type.
 *
 * In MDX, complex props are often passed as strings (e.g., `data="[1, 2, 3]"`
 * or `config={{ a: 1 }}`). This utility evaluates those strings back into objects,
 * arrays, booleans, or numbers so React components can consume them natively.
 *
 * @param propValue - The raw property value, typically extracted from the Rust AST.
 * @returns The parsed JavaScript object, array, boolean, number, or the original string if parsing fails.
 */
export const parseProps = (propValue) => {
    // 1. If it's already a parsed object/boolean (or null/undefined), return it as-is.
    if (typeof propValue !== 'string')
        return propValue;
    let cleanVal = propValue.trim();
    // 2. Handle primitive string values (booleans and numbers)
    if (cleanVal === 'true')
        return true;
    if (cleanVal === 'false')
        return false;
    if (cleanVal !== '' && !isNaN(Number(cleanVal)))
        return Number(cleanVal);
    // 3. Remove outer JSX expression braces ONLY IF it wraps an inner object/array.
    // E.g., `{{ a: 1 }}` becomes `{ a: 1 }`.
    // If it's a single brace like `{ a: 1 }`, we DO NOT strip it, otherwise `new Function` will fail.
    if (cleanVal.startsWith('{') && cleanVal.endsWith('}')) {
        const unbraced = cleanVal.slice(1, -1).trim();
        if ((unbraced.startsWith('[') && unbraced.endsWith(']')) ||
            (unbraced.startsWith('{') && unbraced.endsWith('}'))) {
            cleanVal = unbraced;
        }
    }
    // 4. Evaluate Array [...] or Object {...} string representations.
    if ((cleanVal.startsWith('[') && cleanVal.endsWith(']')) ||
        (cleanVal.startsWith('{') && cleanVal.endsWith('}'))) {
        try {
            // Note: We use `new Function` instead of `JSON.parse` because MDX props 
            // often use relaxed JS syntax with unquoted keys (e.g., { id: "test" }).
            return new Function(`return ${cleanVal}`)();
        }
        catch (e) {
            console.error("Failed to parse JSX property:", cleanVal, e);
            return propValue; // Fallback to raw string on failure
        }
    }
    // 5. Fallback for standard strings (e.g., "hello world")
    return propValue;
};
//# sourceMappingURL=parserHelpers.js.map