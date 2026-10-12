// A plain letter N drawn for this project in one ink. It is a neutral glyph, not Novita's artwork:
// original/novita.mono.svg is the source file and docs/third-party-assets.md says why.
import { type LogoProps, LogoSvg } from './LogoSvg';

const GLYPH = 'M4 20V4h3.2l9.6 11.4V4H20v16h-3.2L7.2 8.6V20Z';

export function NovitaLogo(props: LogoProps) {
  return (
    <LogoSvg harness="novita" tone="mono" viewBox="0 0 24 24" {...props}>
      <path d={GLYPH} fill="currentColor" fillRule="evenodd" />
    </LogoSvg>
  );
}
