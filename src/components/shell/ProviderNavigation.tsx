import { TAB_ORDER } from '../../app/harness';
import type { ViewContext } from '../../app/windowModel';
import { t } from '../../i18n';
import type { HarnessId, HarnessView } from '../../lib/contract';
import { Icon } from '../icons';
import { TabStrip } from './TabStrip';

interface Props {
  views: readonly HarnessView[];
  selected: HarnessId | null;
  ctx: ViewContext;
  onSelect: (id: HarnessId) => void;
  onCustomize: () => void;
}

export function ProviderNavigation({ views, selected, ctx, onSelect, onCustomize }: Props) {
  return (
    <div className="provider-navigation">
      <div className="provider-toolbar">
        <span>
          {t('tabs.yours')}{' '}
          <span className="provider-total">
            {views.length}/{TAB_ORDER.length}
          </span>
        </span>
        <button id="customize-tabs" type="button" className="link-btn" onClick={onCustomize}>
          <Icon name="sliders" />
          {t('tabs.customize')}
        </button>
      </div>
      {selected !== null && (
        <TabStrip views={views} selected={selected} ctx={ctx} onSelect={onSelect} />
      )}
    </div>
  );
}

export function EmptyTabs({ onCustomize }: { onCustomize: () => void }) {
  return (
    <section className="card notice empty-tabs" aria-labelledby="empty-tabs-title">
      <span className="badge">
        <Icon name="sliders" />
      </span>
      <h2 id="empty-tabs-title">{t('tabs.empty_title')}</h2>
      <p>{t('tabs.empty_body')}</p>
      <div className="actions">
        <button type="button" className="btn primary" onClick={onCustomize}>
          {t('tabs.choose')}
        </button>
      </div>
    </section>
  );
}
