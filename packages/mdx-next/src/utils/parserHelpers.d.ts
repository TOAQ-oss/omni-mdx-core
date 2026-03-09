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
export declare const parseProps: (propValue: any) => any;
//# sourceMappingURL=parserHelpers.d.ts.map