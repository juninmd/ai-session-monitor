// A plain ring drawn for this project in one ink. It is a neutral glyph, not OpenRouter's artwork:
// original/openrouter.mono.svg is the source file and docs/third-party-assets.md says why.
import { type LogoProps, LogoSvg } from './LogoSvg';

const GLYPH =
  'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18Zm0 3.4a5.6 5.6 0 1 1 0 11.2 5.6 5.6 0 0 1 0-11.2Z';

export function OpenRouterLogo(props: LogoProps) {
  return (
    <LogoSvg harness="openrouter" tone="mono" viewBox="0 0 24 24" {...props}>
      <path d={GLYPH} fill="currentColor" fillRule="evenodd" />
    </LogoSvg>
  );
}
