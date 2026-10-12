import { harnessName, TAB_ORDER } from '../../app/harness';
import {
  INTERVAL_PRESETS,
  intervalLabel,
  numericOptions,
  percentLabel,
  WARN_PRESETS,
} from '../../app/options';
import { useSettingsApi } from '../../app/settingsContext';
import { t } from '../../i18n';
import {
  type GeneralSettings as General,
  type HarnessId,
  LIMITS,
  type Settings,
  type TimeFormat,
  type WeekFormat,
} from '../../lib/contract';
import type { Option } from '../forms/fields';
import { SettingSegmented, SettingSelect, SettingSwitch } from '../forms/SettingControls';

const patch = (settings: Settings, change: Partial<General>): Settings => ({
  ...settings,
  general: { ...settings.general, ...change },
});

const TIME_OPTIONS: readonly Option<TimeFormat>[] = [
  { value: 'hm', label: t('settings.time_hm') },
  { value: 'colon', label: t('settings.time_colon') },
  { value: 'decimal', label: t('settings.time_decimal') },
];

const WEEK_OPTIONS: readonly Option<WeekFormat>[] = [
  { value: 'hours', label: t('settings.week_hours') },
  { value: 'days_hours', label: t('settings.week_days_hours') },
];

const TRAY_OPTIONS: readonly Option<HarnessId | 'lowest'>[] = [
  { value: 'lowest', label: t('settings.tray_lowest') },
  ...TAB_ORDER.map((id) => ({ value: id, label: harnessName(id) })),
];

/** "Geral" and "Alertas": options that belong to the app, not to one source. */
export function GeneralSettings() {
  const { settings } = useSettingsApi();
  const g = settings.general;
  return (
    <>
      <section className="sg" aria-labelledby="sg-general">
        <h3 id="sg-general">{t('settings.general')}</h3>
        <SettingSelect
          id="set-interval"
          label={t('settings.interval')}
          value={g.intervalSecs}
          options={numericOptions(
            INTERVAL_PRESETS,
            LIMITS.intervalSecs,
            g.intervalSecs,
            intervalLabel,
          )}
          apply={(current, intervalSecs) => patch(current, { intervalSecs })}
        />
        <SettingSwitch
          id="set-autostart"
          label={t('settings.autostart')}
          checked={g.autostart}
          apply={(current, autostart) => patch(current, { autostart })}
        />
        <SettingSelect
          id="set-tray"
          label={t('settings.tray_source')}
          hint={t('settings.tray_source_hint')}
          value={g.traySource ?? 'lowest'}
          options={TRAY_OPTIONS}
          apply={(current, source) =>
            patch(current, { traySource: source === 'lowest' ? null : source })
          }
        />
        <SettingSegmented
          id="set-time-format"
          legend={t('settings.time_format')}
          value={g.timeFormat}
          options={TIME_OPTIONS}
          apply={(current, timeFormat) => patch(current, { timeFormat })}
        />
        <SettingSegmented
          id="set-week-format"
          legend={t('settings.week_format')}
          value={g.weekFormat}
          options={WEEK_OPTIONS}
          apply={(current, weekFormat) => patch(current, { weekFormat })}
        />
        <div className="srow">
          <div className="lbl">
            <span>{t('settings.theme')}</span>
            <small className="hint">{t('settings.theme_hint')}</small>
          </div>
        </div>
      </section>
      <section className="sg" aria-labelledby="sg-alerts">
        <h3 id="sg-alerts">{t('settings.alerts')}</h3>
        <SettingSelect
          id="set-warn"
          label={t('settings.warn_at')}
          hint={t('settings.warn_at_hint')}
          value={g.warnAtPct}
          options={numericOptions(WARN_PRESETS, LIMITS.warnAtPct, g.warnAtPct, percentLabel)}
          apply={(current, warnAtPct) => patch(current, { warnAtPct })}
        />
        <SettingSwitch
          id="set-notify-out"
          label={t('settings.notify_out')}
          checked={g.notifyOnOut}
          apply={(current, notifyOnOut) => patch(current, { notifyOnOut })}
        />
        <SettingSwitch
          id="set-notify-reset"
          label={t('settings.notify_reset')}
          checked={g.notifyOnReset}
          apply={(current, notifyOnReset) => patch(current, { notifyOnReset })}
        />
      </section>
    </>
  );
}
