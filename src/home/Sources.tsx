// The evidence surface: every quota, budget and reset channel the fourteen sources expose, split
// by whether the app reads it, could read it with the credential it already has, or would have to
// break its own rules. Reading, not claims: each row names the endpoint and the fields.
import type { ReactNode } from 'react';
import { Icon } from '../components/icons';
import { t } from '../i18n';
import { CANDIDATES } from './candidates';
import {
  type Channel,
  channels,
  confidenceLevel,
  REACH_ICON,
  REACH_ORDER,
  reachLabel,
  reachNote,
} from './channels';

// A sentence that bolds two of its words gets them wrapped in these marks by the caller; the
// catalog text itself stays plain, so a language can put the words anywhere.
const OPEN = '\u0001';
const CLOSE = '\u0002';
const marked = (word: string) => `${OPEN}${word}${CLOSE}`;

function boldMarked(text: string): ReactNode[] {
  let offset = 0;
  return text.split(new RegExp(`${OPEN}(.*?)${CLOSE}`)).map((part, index) => {
    const key = offset;
    offset += part.length + 1;
    return index % 2 === 1 ? <b key={key}>{part}</b> : part;
  });
}

function ChannelCard({ channel }: { channel: Channel }) {
  return (
    <li className="nx-chan" data-reach={channel.reach}>
      <details>
        <summary className="nx-chan-head">
          <strong>{channel.harness}</strong>
          <span className="nx-chan-title">{channel.title}</span>
          <span className="nx-tag" data-shape={channel.shape}>
            {channel.shape === 'window' ? t('channels.form.window') : t('channels.form.fact')}
          </span>
        </summary>
        <p className="nx-chan-how">{channel.how}</p>
        {channel.caveat !== undefined && <p className="nx-chan-caveat">{channel.caveat}</p>}
        <p className="nx-meta">
          {t('channels.meta', {
            level: confidenceLevel(channel.confidence),
            source: channel.source,
          })}
        </p>
      </details>
    </li>
  );
}

export function Sources() {
  const all = channels();
  const count = (reach: Channel['reach']) =>
    all.filter((channel) => channel.reach === reach).length;
  return (
    <div className="nx-sources">
      <section className="nx-note">
        <h3>{t('channels.page.title')}</h3>
        <p>
          {boldMarked(
            t('channels.page.intro', {
              window: marked(t('channels.form.window')),
              fact: marked(t('channels.form.fact')),
            }),
          )}
        </p>
        <p className="nx-meta">
          {t('channels.page.counts', {
            now: count('now'),
            already: count('already'),
            forbidden: count('forbidden'),
          })}
        </p>
      </section>
      {REACH_ORDER.map((reach) => {
        const items = all.filter((channel) => channel.reach === reach);
        if (items.length === 0) return null;
        return (
          <details className="nx-section" key={reach} open={reach === 'now'}>
            <summary>
              <h3 className="nx-section-head">
                <Icon name={REACH_ICON[reach]} />
                {reachLabel(reach)}
                <span className="nx-count">{items.length}</span>
              </h3>
            </summary>
            <p className="nx-meta nx-note-line">{reachNote(reach)}</p>
            <ul className="nx-chan-list">
              {items.map((channel) => (
                <ChannelCard key={channel.id} channel={channel} />
              ))}
            </ul>
          </details>
        );
      })}
      <details className="nx-section">
        <summary>
          <h3 className="nx-section-head">
            {t('channels.candidates.title')}
            <span className="nx-count">{CANDIDATES.length}</span>
          </h3>
        </summary>
        <p className="nx-meta nx-note-line">{t('channels.candidates.note')}</p>
        <ul className="nx-chan-list">
          {CANDIDATES.map((candidate) => (
            <li className="nx-chan" key={candidate.id}>
              <p className="nx-chan-head">
                <strong>{candidate.name}</strong>
                <span className="nx-tag">
                  {t('channels.confidence', {
                    level: confidenceLevel(candidate.confidence),
                  })}
                </span>
              </p>
              <p className="nx-chan-how">{candidate.offer}</p>
              <p className="nx-meta">
                {t('channels.candidates.credential', { credential: candidate.credential })}
              </p>
            </li>
          ))}
        </ul>
      </details>
    </div>
  );
}
