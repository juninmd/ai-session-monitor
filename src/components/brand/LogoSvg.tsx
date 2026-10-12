// Shared root of every vendored logo: decorative (the tab or row already carries the name).
import { type ReactNode, useId } from 'react';
import type { HarnessId } from '../../lib/contract';

export interface LogoProps {
  size: number;
  className?: string;
}

interface LogoSvgProps extends LogoProps {
  harness: HarnessId;
  /** Colour marks keep their brand colours; mono marks paint currentColor (see styles/brand.css). */
  tone: 'color' | 'mono';
  viewBox: string;
  children: ReactNode;
}

/** An id unique per rendered logo and safe inside url(#...): React's own ids carry punctuation. */
export function useLogoId(): string {
  return `logo-${useId().replace(/\W/g, '')}`;
}

export function LogoSvg({ harness, tone, viewBox, size, className, children }: LogoSvgProps) {
  return (
    <svg
      className={className ? `logo ${className}` : 'logo'}
      data-logo={harness}
      data-tone={tone}
      viewBox={viewBox}
      width={size}
      height={size}
      aria-hidden="true"
      focusable="false"
    >
      {children}
    </svg>
  );
}
