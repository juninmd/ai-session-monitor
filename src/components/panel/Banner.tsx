import type { ReactNode } from 'react';
import { Icon, type IconName } from '../icons';

export type BannerTone = 'info' | 'warn' | 'error';

const DEFAULT_ICON: Record<BannerTone, IconName> = {
  info: 'info',
  warn: 'warn',
  error: 'alert',
};

interface BannerProps {
  tone?: BannerTone;
  icon?: IconName;
  /** Set when the banner appears after load, so screen readers announce it. */
  role?: 'status' | 'alert';
  children: ReactNode;
}

/** Tinted callout. The icon and the text carry the meaning, the tint only reinforces it. */
export function Banner({ tone = 'warn', icon, role, children }: BannerProps) {
  return (
    <div className={tone === 'warn' ? 'banner' : `banner ${tone}`} role={role}>
      <Icon name={icon ?? DEFAULT_ICON[tone]} />
      <div className="banner-body">{children}</div>
    </div>
  );
}

/** Muted one-line aside. */
export function Note({ icon = 'info', children }: { icon?: IconName; children: ReactNode }) {
  return (
    <p className="note">
      <Icon name={icon} />
      <span>{children}</span>
    </p>
  );
}
