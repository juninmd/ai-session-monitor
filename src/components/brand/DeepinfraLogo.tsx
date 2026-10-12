// A plain stack of bars drawn for this project in one ink. It is a neutral glyph, not DeepInfra's artwork:
// original/deepinfra.mono.svg is the source file and docs/third-party-assets.md says why.
import { type LogoProps, LogoSvg } from './LogoSvg';

const GLYPH = 'M4 4.5h16v3.6H4ZM4 10.2h16v3.6H4ZM4 15.9h10.5v3.6H4Z';

export function DeepinfraLogo(props: LogoProps) {
  return (
    <LogoSvg harness="deepinfra" tone="mono" viewBox="0 0 24 24" {...props}>
      <path d={GLYPH} fill="currentColor" fillRule="evenodd" />
    </LogoSvg>
  );
}
