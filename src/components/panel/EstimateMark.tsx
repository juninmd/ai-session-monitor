import { t } from '../../i18n';

/** "~" for sighted users; screen readers hear "estimativa". */
export function EstimateMark() {
  return (
    <span className="est">
      <span aria-hidden="true">~</span>
      <span className="vh">{`${t('estimate.word')}: `}</span>
    </span>
  );
}
