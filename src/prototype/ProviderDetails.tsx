import { isEstimatedFact } from '../app/estimate';
import { groupName } from '../app/layout';
import { useNavigation } from '../app/navigation';
import { uniqueNotices } from '../app/noticeText';
import type { ViewContext } from '../app/windowModel';
import { EstimateMark } from '../components/panel/EstimateMark';
import { factLabel } from '../components/panel/Facts';
import { HistorySection } from '../components/panel/history/HistorySection';
import { Notices } from '../components/panel/Notices';
import { StaleBanner } from '../components/panel/StaleBanner';
import { hasMessage, t } from '../i18n';
import { formatAge, formatFactValue } from '../i18n/format';
import type { HarnessView, UsageWindow } from '../lib/contract';
import { ProviderActions } from './ProviderActions';
import { compactForecast, compactQuota, unofficialSource } from './quotaModel';

function DetailFacts({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  return (
    <dl className="preview-detail-facts">
      {view.facts.map((fact) => (
        <div key={fact.id}>
          <dt>{factLabel(fact, ctx.nowSecs)}</dt>
          <dd>
            {isEstimatedFact(view, fact.id) && <EstimateMark />}
            {formatFactValue(fact)}
          </dd>
        </div>
      ))}
    </dl>
  );
}

function DetailRow({
  view,
  window,
  ctx,
}: {
  view: HarnessView;
  window: UsageWindow;
  ctx: ViewContext;
}) {
  const quota = compactQuota(view, window, ctx);
  const forecast = compactForecast(view, quota.model, ctx);
  return (
    <tr data-window={window.id}>
      <th scope="row" title={quota.model.title}>
        {quota.model.cadenceGuessed && <EstimateMark />}
        {quota.name}
        {forecast && (
          <small
            className="preview-detail-forecast"
            data-urgent={forecast.urgent || undefined}
            title={forecast.text}
          >
            {forecast.estimate && <EstimateMark />}
            <span aria-hidden="true">{forecast.label}</span>
            <span className="vh">{forecast.text}</span>
          </small>
        )}
      </th>
      <td data-status={quota.status} title={quota.spoken}>
        <span className="preview-detail-meter">
          <span>{quota.reading}</span>
          <span
            className={`t-bar${quota.shown === null ? ' off' : ''}`}
            data-status={quota.status}
            aria-hidden="true"
          >
            {quota.shown !== null && <i style={{ inlineSize: `${quota.shown}%` }} />}
          </span>
        </span>
      </td>
      <td>{quota.renewal}</td>
    </tr>
  );
}

/** All quotas and renewal times in one list; secondary facts and guidance stay disclosed. */
export function ProviderDetails({ view, ctx }: { view: HarnessView; ctx: ViewContext }) {
  const navigation = useNavigation();
  const groups = new Map<string | null, UsageWindow[]>();
  for (const window of view.windows) {
    groups.set(window.group, [...(groups.get(window.group) ?? []), window]);
  }
  const warnings = uniqueNotices(view.notices).filter(
    (notice) => notice.severity !== 'info',
  ).length;
  const sourceKey = `source.${view.source?.id}`;
  const source = view.source ? (hasMessage(sourceKey) ? t(sourceKey) : view.source.id) : null;
  return (
    <div className="preview-provider-details">
      {(view.stale || view.error !== null) && <StaleBanner view={view} ctx={ctx} />}
      <ProviderActions key={view.id} view={view} ctx={ctx} />
      {view.windows.length > 0 ? (
        <table className="preview-quota-table" aria-label="Cotas e renovação">
          <thead>
            <tr>
              <th scope="col">Cota</th>
              <th scope="col">{ctx.settings.general.showUsed ? 'Uso' : 'Disponível'}</th>
              <th scope="col">Renova em</th>
            </tr>
          </thead>
          {[...groups.entries()].map(([group, windows]) => (
            <tbody key={group === null ? 'plain' : `group:${group}`}>
              {group !== null && (
                <tr className="preview-detail-group">
                  <th colSpan={3} scope="rowgroup">
                    {groupName(group)}
                  </th>
                </tr>
              )}
              {windows.map((window) => (
                <DetailRow key={window.id} view={view} window={window} ctx={ctx} />
              ))}
            </tbody>
          ))}
        </table>
      ) : view.facts.length > 0 ? (
        <DetailFacts view={view} ctx={ctx} />
      ) : (
        <p className="hint">{t('window.none')}</p>
      )}
      <div className="preview-detail-history">
        <HistorySection key={view.id} view={view} ctx={ctx} />
      </div>
      <details className="preview-additional">
        <summary>
          Informações adicionais
          {warnings > 0 && (
            <span className="preview-detail-warning">
              {' '}
              · {warnings} {warnings === 1 ? 'aviso' : 'avisos'}
            </span>
          )}
        </summary>
        <div className="preview-additional-body">
          <p>
            {view.readAt === null
              ? 'Sem horário de leitura'
              : `Lido há ${formatAge(ctx.nowSecs - view.readAt)}`}
          </p>
          {view.plan && <p>Plano: {view.plan}</p>}
          {source && (
            <p>
              Fonte: {source}
              {unofficialSource(view, ctx) ? ' · leitura não oficial' : ''}
            </p>
          )}
          {view.windows.length > 0 && view.facts.length > 0 && (
            <DetailFacts view={view} ctx={ctx} />
          )}
          <Notices notices={view.notices} harness={view.id} placement="top" />
          <Notices notices={view.notices} harness={view.id} placement="bottom" />
          {view.id === 'claude' && !ctx.settings.claude.oauthUsage && (
            <p>Cotas por modelo podem ser ativadas nas configurações.</p>
          )}
          {!view.plan && !source && view.facts.length === 0 && view.notices.length === 0 && (
            <p>Sem informações adicionais.</p>
          )}
          <button
            type="button"
            className="link-btn preview-detail-config"
            onClick={() => navigation.openSettings(view.id)}
          >
            Configurar
          </button>
        </div>
      </details>
    </div>
  );
}
