// Generic stroke icons on a 16 px grid. Each tool's own logo lives in components/brand.
import type { ReactNode } from 'react';

const PATHS = {
  refresh: (
    <>
      <path d="M13 8a5 5 0 1 1-1.6-3.7" />
      <path d="M13 2.6v3h-3" />
    </>
  ),
  sliders: (
    <>
      <path d="M2 5h6M12 5h2M2 11h2M8 11h6" />
      <circle cx="10" cy="5" r="1.7" />
      <circle cx="6" cy="11" r="1.7" />
    </>
  ),
  back: (
    <>
      <path d="M9.5 3.5 5 8l4.5 4.5" />
      <path d="M5.5 8H13" />
    </>
  ),
  next: <path d="m6.5 3.5 4.5 4.5-4.5 4.5" />,
  check: (
    <>
      <circle cx="8" cy="8" r="6" />
      <path d="m5.4 8.2 1.8 1.8 3.4-3.6" />
    </>
  ),
  warn: (
    <>
      <path d="M8 2.3 14.2 13H1.8z" />
      <path d="M8 6.6v3.1" />
      <circle cx="8" cy="11.4" r=".5" fill="currentColor" />
    </>
  ),
  alert: (
    <>
      <circle cx="8" cy="8" r="6" />
      <path d="M8 4.8v3.6" />
      <circle cx="8" cy="10.9" r=".5" fill="currentColor" />
    </>
  ),
  ban: (
    <>
      <circle cx="8" cy="8" r="6" />
      <path d="m3.8 12.2 8.4-8.4" />
    </>
  ),
  clock: (
    <>
      <circle cx="8" cy="8" r="6" />
      <path d="M8 4.8V8l2.2 1.4" />
    </>
  ),
  lock: (
    <>
      <rect x="3.5" y="7" width="9" height="6.5" rx="1.5" />
      <path d="M5.5 7V5.5a2.5 2.5 0 0 1 5 0V7" />
    </>
  ),
  info: (
    <>
      <circle cx="8" cy="8" r="6" />
      <path d="M8 7.4v3.4" />
      <circle cx="8" cy="5.4" r=".5" fill="currentColor" />
    </>
  ),
  plug: <path d="M6 2.5v3M10 2.5v3M4.5 5.5h7v2.2a3.5 3.5 0 0 1-7 0zM8 11.2v2.3" />,
  power: (
    <>
      <path d="M8 2.5v5" />
      <path d="M4.6 4.7a5 5 0 1 0 6.8 0" />
    </>
  ),
  external: (
    <>
      <path d="M9 3h4v4M13 3 7.5 8.5" />
      <path d="M11 9.5V12a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h2.5" />
    </>
  ),
  key: (
    <>
      <circle cx="5.5" cy="10.5" r="2.5" />
      <path d="m7.3 8.7 5.2-5.2M10.5 5.5 12 7" />
    </>
  ),
} satisfies Record<string, ReactNode>;

export type IconName = keyof typeof PATHS;

export function Icon({ name, className }: { name: IconName; className?: string }) {
  return (
    <svg
      className={className ? `ico ${className}` : 'ico'}
      viewBox="0 0 16 16"
      width={16}
      height={16}
      fill="none"
      stroke="currentColor"
      strokeWidth={1.5}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      {PATHS[name]}
    </svg>
  );
}
