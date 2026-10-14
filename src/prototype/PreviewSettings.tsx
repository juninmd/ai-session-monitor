import { useState } from 'react';
import { useDraftGuard } from '../app/draftContext';
import type { SettingsSection } from '../app/harness';
import type { ViewContext } from '../app/windowModel';
import { SettingsView } from '../components/settings/SettingsView';
import { ProviderPicker } from '../components/shell/ProviderPicker';
import type { HarnessId, HarnessView } from '../lib/contract';

export function PreviewSettings({
  views,
  selected,
  ctx,
  section,
  onBack,
}: {
  views: readonly HarnessView[];
  selected: HarnessId;
  ctx: ViewContext;
  section: SettingsSection | null;
  onBack: () => void;
}) {
  const guard = useDraftGuard();
  const [target, setTarget] = useState(section);
  return (
    <SettingsView
      key={target ?? 'all'}
      section={target}
      onBack={onBack}
      overview={
        <ProviderPicker
          views={views}
          selected={selected}
          ctx={ctx}
          onSelect={(id) => {
            const open = () => setTarget(id);
            if (guard) guard.request(open);
            else open();
          }}
        />
      }
    />
  );
}
