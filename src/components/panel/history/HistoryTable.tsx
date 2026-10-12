import type { Burndown } from '../../../app/burndown';
import { tableRows } from '../../../app/burndownText';
import { t } from '../../../i18n';

/** The same numbers as the chart, as a table: the accessible way to read them. */
export function HistoryTable({ burndown, nowSecs }: { burndown: Burndown; nowSecs: number }) {
  return (
    <details className="bd-table">
      <summary>{t('history.table_toggle')}</summary>
      <table>
        <caption className="vh">{t('history.table_caption')}</caption>
        <thead>
          <tr>
            <th scope="col">{t('history.col_reset')}</th>
            <th scope="col">{t('history.col_peak')}</th>
            <th scope="col">{t('history.col_readings')}</th>
          </tr>
        </thead>
        <tbody>
          {tableRows(burndown, nowSecs).map((row) => (
            <tr key={row.key}>
              <th scope="row">{row.reset}</th>
              <td>{row.peak}</td>
              <td>{row.readings}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </details>
  );
}
