import type { ViewContext } from '../../app/windowModel';
import { hasMessage, t } from '../../i18n';
import { formatAge } from '../../i18n/format';
import type { HarnessView } from '../../lib/contract';
import { Icon } from '../icons';

const MAX_EXTRA_META = 3;
const OAUTH_SOURCE = 'oauth_usage';

function sourceName(id: string): string {
  const key = `source.${id}`;
  return hasMessage(key) ? t(key) : id;
}

function scopeLabel(scope: string): string {
  const key = `meta.scope.${scope}`;
  return hasMessage(key) ? t(key) : scope;
}

/** Meta with a chip of its own; any other key, or an unexpected value, is listed as it came. */
function isNamed([key, value]: [string, string], scope: string | undefined): boolean {
  switch (key) {
    case 'alias':
    case 'host':
    case 'scope':
      return true;
    case 'level':
      return value === scope || hasMessage(`meta.level.${value}`);
    case 'local_estimate':
      return value === 'true';
    case 'extra_source':
      return value === OAUTH_SOURCE;
    default:
      return false;
  }
}

/** Where the budget came from, only when it is not the level the user configured. */
function levelChip(level: string | undefined, scope: string | undefined): string | null {
  if (level === undefined || level === scope) return null;
  const key = `meta.level.${level}`;
  return hasMessage(key) ? t(key) : null;
}

/** Plan, where the numbers come from, how fresh they are, and the official/unofficial badge. */
export function MetaChips({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  const { meta, source } = view;
  const identity = [meta.scope ? scopeLabel(meta.scope) : null, meta.alias ?? null]
    .filter((part): part is string => part !== null)
    .join(': ');
  const level = levelChip(meta.level, meta.scope);
  const extras = Object.entries(meta)
    .filter((entry) => !isNamed(entry, meta.scope))
    .slice(0, MAX_EXTRA_META);
  const insecure = view.id === 'litellm' && ctx.settings.litellm.tls.mode === 'insecure';
  // Windows from the usage API are on screen whatever the setting says by now.
  const oauth =
    view.id === 'claude' && (ctx.settings.claude.oauthUsage || meta.extra_source === OAUTH_SOURCE);

  return (
    <div className="meta">
      {view.plan && <span className="chip plain">{view.plan}</span>}
      {identity && <span className="chip plain">{identity}</span>}
      {meta.host && <span className="chip plain">{meta.host}</span>}
      {level && (
        <span className="chip plain" title={t('meta.level_tip')}>
          {level}
        </span>
      )}
      {meta.local_estimate === 'true' && (
        <span className="chip plain">{t('meta.local_estimate')}</span>
      )}
      {extras.map(([key, value]) => (
        <span className="chip plain" key={key}>{`${key}: ${value}`}</span>
      ))}
      {view.readAt !== null && (
        <span className="chip">
          <Icon name="clock" />
          {t('meta.read_ago', { age: formatAge(ctx.nowSecs - view.readAt) })}
        </span>
      )}
      {source?.official === true && (
        <span className="chip" title={t('meta.official_tip')}>
          <Icon name="check" />
          {`${t('meta.official')} · ${sourceName(source.id)}`}
        </span>
      )}
      {source?.official === false && (
        <span className="chip warnchip" title={t('meta.unofficial_tip')}>
          <Icon name="warn" />
          {`${t('meta.unofficial')} · ${sourceName(source.id)}`}
        </span>
      )}
      {oauth && (
        <span className="chip warnchip">
          <Icon name="warn" />
          {t('claude.oauth.chip')}
        </span>
      )}
      {insecure && (
        <span className="chip warnchip">
          <Icon name="warn" />
          {t('meta.tls_insecure')}
        </span>
      )}
    </div>
  );
}
