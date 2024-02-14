/**
 * Gets the keys of an Object as an union type.
 */
export type Keys<T> = T extends T ? keyof T : never;

/**
 * Extracts an Type from a Type Map.
 *
 * Eg.
 * ```
 * type A = { X: string } | { Y: number}
 *
 * Extract<A, 'X'> => number
 * ```
 */
export type Select<T, K extends Keys<T>> = T extends { [P in K]: infer V }
  ? V
  : never;
