import { harnessName, TAB_ORDER } from '../../app/harness';
import {
  INTERVAL_PRESETS,
  intervalLabel,
  numericOptions,
  parseThresholdKey,
  percentLabel,
  thresholdKey,
  thresholdOptions,
  WARN_PRESETS,
} from '../../app/options';
import { useSettingsApi } from '../../app/settingsContext';
import { t } from '../../i18n';
import {
  type GeneralSettings as General,
  type HarnessId,
  LIMITS,
  LOCAL_API_PORT,
  type Settings,
  type TimeFormat,
  type WeekFormat,
} from '../../lib/contract';
import type { Option } from '../forms/fields';
import {
  SettingSegmented,
  SettingSelect,
  SettingSwitch,
  SettingTime,
} from '../forms/SettingControls';
import { LanguageSetting } from './LanguageSetting';

const patch = (settings: Settings, change: Partial<General>): Settings => ({
  ...settings,
  general: { ...settings.general, ...change },
});

const timeOptions = (): readonly Option<TimeFormat>[] => [
  { value: 'hm', label: t('settings.time_hm') },
  { value: 'colon', label: t('settings.time_colon') },
  { value: 'decimal', label: t('settings.time_decimal') },
];

const weekOptions = (): readonly Option<WeekFormat>[] => [
  { value: 'hours', label: t('settings.week_hours') },
  { value: 'days_hours', label: t('settings.week_days_hours') },
];

const quotaOptions = (): readonly Option<'available' | 'used'>[] => [
  { value: 'available', label: t('settings.show_available') },
  { value: 'used', label: t('settings.show_used_opt') },
];

const densityOptions = (): readonly Option<'comfortable' | 'compact'>[] => [
  { value: 'comfortable', label: t('settings.density_comfortable') },
  { value: 'compact', label: t('settings.density_compact') },
];

const trayOptions = (): readonly Option<HarnessId | 'lowest'>[] => [
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
        <LanguageSetting />
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
          id="set-adaptive"
          label={t('settings.adaptive')}
          hint={t('settings.adaptive_hint')}
          checked={g.adaptive}
          apply={(current, adaptive) => patch(current, { adaptive })}
        />
        <SettingSwitch
          id="set-autostart"
          label={t('settings.autostart')}
          checked={g.autostart}
          apply={(current, autostart) => patch(current, { autostart })}
        />
        <SettingSwitch
          id="set-hide-capture"
          label={t('settings.hide_capture')}
          hint={t('settings.hide_capture_hint')}
          checked={g.hideFromCapture}
          apply={(current, hideFromCapture) => patch(current, { hideFromCapture })}
        />
        <SettingSwitch
          id="set-local-api"
          label={t('settings.local_api')}
          hint={t('settings.local_api_hint', { port: LOCAL_API_PORT })}
          checked={g.localApi}
          apply={(current, localApi) => patch(current, { localApi })}
        />
        <SettingSelect
          id="set-tray"
          label={t('settings.tray_source')}
          hint={t('settings.tray_source_hint')}
          value={g.traySource ?? 'lowest'}
          options={trayOptions()}
          apply={(current, source) =>
            patch(current, { traySource: source === 'lowest' ? null : source })
          }
        />
        <SettingSegmented
          id="set-show-used"
          legend={t('settings.show_used')}
          value={g.showUsed ? 'used' : 'available'}
          options={quotaOptions()}
          apply={(current, mode) => patch(current, { showUsed: mode === 'used' })}
        />
        <SettingSegmented
          id="set-density"
          legend={t('settings.density')}
          value={g.compact ? 'compact' : 'comfortable'}
          options={densityOptions()}
          apply={(current, mode) => patch(current, { compact: mode === 'compact' })}
        />
        <SettingSegmented
          id="set-time-format"
          legend={t('settings.time_format')}
          value={g.timeFormat}
          options={timeOptions()}
          apply={(current, timeFormat) => patch(current, { timeFormat })}
        />
        <SettingSegmented
          id="set-week-format"
          legend={t('settings.week_format')}
          value={g.weekFormat}
          options={weekOptions()}
          apply={(current, weekFormat) => patch(current, { weekFormat })}
        />
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
        <SettingSelect
          id="set-thresholds"
          label={t('settings.alert_thresholds')}
          hint={t('settings.alert_thresholds_hint')}
          value={thresholdKey(g.alertThresholds)}
          options={thresholdOptions(g.alertThresholds)}
          apply={(current, key) => patch(current, { alertThresholds: parseThresholdKey(key) })}
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
        <SettingSwitch
          id="set-notify-risk"
          label={t('settings.notify_risk')}
          checked={g.notifyOnRisk}
          apply={(current, notifyOnRisk) => patch(current, { notifyOnRisk })}
        />
        <SettingSwitch
          id="set-quiet"
          label={t('settings.quiet_hours')}
          hint={t('settings.quiet_hint')}
          checked={g.quietHoursOn}
          apply={(current, quietHoursOn) => patch(current, { quietHoursOn })}
        />
        {g.quietHoursOn && (
          <>
            <SettingTime
              id="set-quiet-from"
              label={t('settings.quiet_from')}
              minutes={g.quietFrom}
              apply={(current, quietFrom) => patch(current, { quietFrom })}
            />
            <SettingTime
              id="set-quiet-to"
              label={t('settings.quiet_to')}
              minutes={g.quietTo}
              apply={(current, quietTo) => patch(current, { quietTo })}
            />
          </>
        )}
      </section>
    </>
  );
}
