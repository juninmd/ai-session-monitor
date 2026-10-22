// Informações adicionais: the facts, the notices and the way into the settings, as a labelled grid
// instead of a column of sentences. Everything here is already on the wire view; nothing is read
// from anywhere else.
import { isEstimatedFact } from '../app/estimate';
import { useNavigation } from '../app/navigation';
import { uniqueNotices } from '../app/noticeText';
import type { ViewContext } from '../app/windowModel';
import { EstimateMark } from '../components/panel/EstimateMark';
import { factLabel } from '../components/panel/Facts';
import { Notices } from '../components/panel/Notices';
import { t, tn } from '../i18n';
import { formatFactValue } from '../i18n/format';
import type { HarnessView } from '../lib/contract';

export function InfoBoard({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  const navigation = useNavigation();
  const warnings = uniqueNotices(view.notices).filter(
    (notice) => notice.severity !== 'info',
  ).length;
  const rows = view.facts.map((fact) => ({
    id: fact.id,
    label: factLabel(fact, ctx.nowSecs),
    value: formatFactValue(fact),
    estimated: isEstimatedFact(view, fact.id),
  }));
  return (
    <details className="nx-more">
      <summary>
        {t('info.title')}
        {warnings > 0 && <span className="nx-more-warn">{tn('info.warnings', warnings)}</span>}
      </summary>
      <div className="nx-more-body">
        {rows.length > 0 && (
          <dl className="nx-pairs">
            {rows.map((row) => (
              <div key={row.id}>
                <dt>{row.label}</dt>
                <dd>
                  {row.estimated && <EstimateMark />}
                  {row.value}
                </dd>
              </div>
            ))}
          </dl>
        )}
        <Notices notices={view.notices} harness={view.id} placement="top" />
        <Notices notices={view.notices} harness={view.id} placement="bottom" />
        {rows.length === 0 && view.notices.length === 0 && (
          <p className="hint">{t('info.nothing_more')}</p>
        )}
        <div className="actions">
          <button type="button" className="btn" onClick={() => navigation.openSettings(view.id)}>
            {t('info.configure')}
          </button>
        </div>
      </div>
    </details>
  );
}
