import { useEffect, useRef, useState } from 'react';
import type { HarnessId, HarnessView } from '../../lib/contract';
import { showsUsed } from '../model/quota';
import { summaryOf } from '../model/tabModel';
import type { ViewContext } from '../model/windowModel';
import { announcementFor, type StatusKey, statusKeyOf } from '../text/announce';

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
      const sentence = announcementFor(
        view.id,
        previous.current.get(view.id),
        key,
        summary,
        showsUsed(ctx.settings),
      );
      if (sentence) sentences.push(sentence);
    }
    previous.current = next;
    if (sentences.length > 0) setMessage(sentences.join(' '));
  }, [views, ctx]);

  return message;
}
