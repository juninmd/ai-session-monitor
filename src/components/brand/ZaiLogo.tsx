// A plain "Z" drawn for this project in one ink. It is a neutral glyph, not Z.ai's artwork:
// original/zai.mono.svg is the source file and docs/third-party-assets.md says why.
import { type LogoProps, LogoSvg } from './LogoSvg';

const GLYPH = 'M4 3H20V6.4L9 17.6H20V21H4V17.6L15 6.4H4Z';

export function ZaiLogo(props: LogoProps) {
  return (
    <LogoSvg harness="zai" tone="mono" viewBox="0 0 24 24" {...props}>
      <path d={GLYPH} fill="currentColor" />
    </LogoSvg>
  );
}
