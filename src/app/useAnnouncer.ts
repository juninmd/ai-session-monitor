import { useEffect, useRef, useState } from 'react';
import type { HarnessId, HarnessView } from '../lib/contract';
import { announcementFor, type StatusKey, statusKeyOf } from './announce';
import { summaryOf } from './tabModel';
import type { ViewContext } from './windowModel';

/** Text for the polite live region: only status changes after the first snapshot are spoken. */
export function useAnnouncer(views: readonly HarnessView[], ctx: ViewContext): string {
  const [message, setMessage] = useState('');
  const previous = useRef(new Map<HarnessId, StatusKey>());

  useEffect(() => {
    const next = new Map<HarnessId, StatusKey>();
    const sentences: string[] = [];
    for (const view of views) {
      const summary = summaryOf(view, ctx);
      const key = statusKeyOf(view, summary);
      next.set(view.id, key);
      const sentence = announcementFor(view.id, previous.current.get(view.id), key, summary);
      if (sentence) sentences.push(sentence);
    }
    previous.current = next;
    if (sentences.length > 0) setMessage(sentences.join(' '));
  }, [views, ctx]);

  return message;
}
