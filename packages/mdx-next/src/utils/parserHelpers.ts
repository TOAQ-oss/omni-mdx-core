/**
 * Parses a stringified JSX property value into its native JavaScript type.
 *
 * Useful in custom components that receive props from the MDX AST —
 * especially for expression attributes like `data={[1, 2, 3]}` or `config={{ a: 1 }}`.
 *
 * @param propValue - Raw property value from the Rust AST (may be a string representation)
 * @returns Parsed JS value — object, array, boolean, number, or original string as fallback
 */
export const parseProps = (propValue: any): any => {
  if (typeof propValue !== "string") return propValue;
 
  const cleanVal = propValue.trim();
 
  // Primitives
  if (cleanVal === "true")  return true;
  if (cleanVal === "false") return false;
  if (cleanVal !== "" && !isNaN(Number(cleanVal))) return Number(cleanVal);
 
  // Unwrap double JSX braces: {{ a: 1 }} → { a: 1 }
  let val = cleanVal;
  if (val.startsWith("{") && val.endsWith("}")) {
    const inner = val.slice(1, -1).trim();
    if (
      (inner.startsWith("[") && inner.endsWith("]")) ||
      (inner.startsWith("{") && inner.endsWith("}"))
    ) {
      val = inner;
    }
  }
 
  // Evaluate arrays and objects (relaxed JS syntax, not strict JSON)
  if (
    (val.startsWith("[") && val.endsWith("]")) ||
    (val.startsWith("{") && val.endsWith("}"))
  ) {
    try {
      return JSON.parse(val);
    } catch {
      return propValue;
    }
  }
 
  return propValue;
};
 