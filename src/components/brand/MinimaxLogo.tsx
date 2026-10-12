// A plain bar chart drawn for this project in one ink. It is a neutral glyph, not MiniMax's artwork:
// original/minimax.mono.svg is the source file and docs/third-party-assets.md says why.
import { type LogoProps, LogoSvg } from './LogoSvg';

const GLYPH = 'M4 9h3v10H4Zm6.5-5h3v15h-3ZM17 12h3v7h-3Z';

export function MinimaxLogo(props: LogoProps) {
  return (
    <LogoSvg harness="minimax" tone="mono" viewBox="0 0 24 24" {...props}>
      <path d={GLYPH} fill="currentColor" fillRule="evenodd" />
    </LogoSvg>
  );
}
