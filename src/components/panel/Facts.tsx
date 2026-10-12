import { isEstimatedFact } from '../../app/estimate';
import { hasMessage, t } from '../../i18n';
import { formatFactValue } from '../../i18n/format';
import type { Fact, HarnessView } from '../../lib/contract';
import { EstimateMark } from './EstimateMark';

/**
 * A fact's id is a message key; an id without a message shows as itself instead of vanishing.
 * A timestamp already behind us reads in the past tense ("Expirou em") when the catalog has it.
 */
export function factLabel(fact: Fact, nowSecs: number): string {
  const past = `fact.${fact.id}_past`;
  const behind = fact.unit === 'epoch_s' && fact.number !== null && fact.number <= nowSecs;
  if (behind && hasMessage(past)) return t(past);
  const key = `fact.${fact.id}`;
  return hasMessage(key) ? t(key) : fact.id;
}

/** The limits belong to the level the user configured, which the view reports as its scope. */
function factsTitle({ id, meta }: HarnessView): string {
  const scoped = `facts.title.${id}.${meta.scope}`;
  if (meta.scope !== undefined && hasMessage(scoped)) return t(scoped);
  const key = `facts.title.${id}`;
  return hasMessage(key) ? t(key) : t('facts.title.default');
}

export function Facts({ view, nowSecs }: { view: HarnessView; nowSecs: number }) {
  if (view.facts.length === 0) return null;
  return (
    <section className="flat">
      <h3 className="win-title">{factsTitle(view)}</h3>
      <dl className="stats">
        {view.facts.map((fact) => (
          <div key={fact.id}>
            <dt>{factLabel(fact, nowSecs)}</dt>
            <dd>
              {isEstimatedFact(view, fact.id) && <EstimateMark />}
              {formatFactValue(fact)}
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
