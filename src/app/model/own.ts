/** Table lookup that skips inherited keys such as "constructor": the key can come from the wire. */
export function own<V>(
  table: Readonly<Record<string, V>>,
  key: string | null | undefined,
): V | undefined {
  if (key === null || key === undefined || !Object.hasOwn(table, key)) return undefined;
  return table[key];
}
