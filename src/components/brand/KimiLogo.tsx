// A plain "K" drawn for this project in one ink. It is a neutral glyph, not Moonshot's artwork:
// original/kimi.mono.svg is the source file and docs/third-party-assets.md says why.
import { type LogoProps, LogoSvg } from './LogoSvg';

const GLYPH = 'M5 3H8V21H5ZM8 12L16.4 3H20.6L12.2 12ZM8 12H12.2L20.6 21H16.4Z';

export function KimiLogo(props: LogoProps) {
  return (
    <LogoSvg harness="kimi" tone="mono" viewBox="0 0 24 24" {...props}>
      <path d={GLYPH} fill="currentColor" />
    </LogoSvg>
  );
}
