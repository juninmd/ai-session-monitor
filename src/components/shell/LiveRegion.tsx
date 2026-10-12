/** Polite announcements for status changes; visually hidden, read by screen readers only. */
export function LiveRegion({ message }: { message: string }) {
  return (
    <output className="vh" aria-live="polite" aria-atomic="true">
      {message}
    </output>
  );
}
