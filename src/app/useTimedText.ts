import { useEffect, useState } from 'react';

/** A one-off line (a notice, a refusal) that clears itself: set it, and it is gone after `ms`. */
export function useTimedText(ms: number): readonly [string, (text: string) => void] {
  const [text, setText] = useState('');
  useEffect(() => {
    if (text === '') return;
    const timer = setTimeout(() => setText(''), ms);
    return () => clearTimeout(timer);
  }, [text, ms]);
  return [text, setText];
}
