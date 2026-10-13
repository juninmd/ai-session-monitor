// A plain letter M drawn for this project in one ink. It is a neutral glyph, not Moonshot's artwork:
// original/moonshot.mono.svg is the source file and docs/third-party-assets.md says why.
import { type LogoProps, LogoSvg } from './LogoSvg';

const GLYPH = 'M3 20V4h3.3l5.7 8.6L17.7 4H21v16h-3.3v-9.4l-4.4 6.6h-2.6L6.3 10.6V20Z';

export function MoonshotLogo(props: LogoProps) {
  return (
    <LogoSvg harness="moonshot" tone="mono" viewBox="0 0 24 24" {...props}>
      <path d={GLYPH} fill="currentColor" fillRule="evenodd" />
    </LogoSvg>
  );
}
