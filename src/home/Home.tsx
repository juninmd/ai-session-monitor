// The home as the flyout renders it: the contexts the surfaces read, the frame itself and the
// live region. App owns the state and the backend wiring; this is only the composition.
import { BackendContext } from '../app/backendContext';
import { type Navigation, NavigationContext } from '../app/navigation';
import { SettingsContext } from '../app/settingsContext';
import type { SettingsApi } from '../app/useSettingsState';
import { type UpdateApi, UpdateContext } from '../app/useUpdateState';
import { LiveRegion } from '../components/shell/LiveRegion';
import type { Backend } from '../lib/backend';
import { Fly, type FlyProps } from './Fly';

export interface HomeProps {
  backend: Backend;
  settings: SettingsApi;
  update: UpdateApi;
  navigation: Navigation;
  fly: FlyProps;
  /** The announcer's line and the one-off notices, read out as a whole. */
  message: string;
}

export function Home({ backend, settings, update, navigation, fly, message }: HomeProps) {
  return (
    <BackendContext value={backend}>
      <UpdateContext value={update}>
        <SettingsContext value={settings}>
          <NavigationContext value={navigation}>
            <Fly {...fly} />
            <LiveRegion message={message} />
          </NavigationContext>
        </SettingsContext>
      </UpdateContext>
    </BackendContext>
  );
}
